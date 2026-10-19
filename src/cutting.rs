use crate::command::{Command, CommandSupport};
use crate::shop::LeatherSheet;
use crate::title_screen::GameState;
use avian3d::parry::glamx::{Quat, Vec2Swizzles, Vec3};
use avian3d::prelude::{SpatialQuery, SpatialQueryFilter};
use bevy::app::{App, Plugin, Update};
use bevy::asset::{AssetServer, Assets, Handle, RenderAssetUsages};
use bevy::camera::Camera;
use bevy::color::Color;
use bevy::input::ButtonInput;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::math::{Dir3, Vec2, Vec3Swizzles};
use bevy::mesh::{Indices, Mesh, Mesh3d, PrimitiveTopology};
use bevy::pbr::decal::ForwardDecal;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{
    Children, Commands, Component, EaseFunction, Entity, GlobalTransform, InfinitePlane3d,
    IntoScheduleConfigs, KeyCode, MouseButton, NextState, Query, Res, ResMut, Single, Touches,
    Transform, Visibility, Window, With, Without, default, in_state,
};
use bevy::window::PrimaryWindow;
use std::collections::VecDeque;
use std::f32::consts::PI;

const TABLE_HEIGHT: f32 = 0.617;
const MAX_CUT_DISTANCE: f32 = 0.135;
const CUT_THICKNESS: f32 = 0.002;
const SHEET_HEIGHT: f32 = 0.02;

pub struct KnifePlugin;

impl Plugin for KnifePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                pick_up_knife,
                move_knife,
                rotate_knife,
                use_knife,
                cut_leather_sheet,
            )
                .run_if(in_state(GameState::Cutting)),
        );

        app.add_systems(
            Update,
            render_cuts
                .after(clear_cuts)
                .run_if(in_state(GameState::Cutting)),
        );
        app.add_systems(Update, clear_cuts);
    }
}

#[derive(Component)]
pub struct Knife {
    in_hand: bool,
    pressed: bool,
    first_click: bool,
    angle: f32,
    on_sheet: bool,
}

#[derive(Component)]
pub struct Sole {
    left: bool,
}

pub struct Cut {
    center: Vec2,
    angle: f32,
    size_positive: f32,
    size_negative: f32,
}

impl Cut {
    pub fn intersect(&self, other: &Self) -> Option<Vec2> {
        let a_start = self.center + Vec2::from_angle(self.angle) * self.size_positive;
        let a_end = self.center - Vec2::from_angle(self.angle) * self.size_negative;

        let b_start = other.center + Vec2::from_angle(other.angle) * other.size_positive;
        let b_end = other.center - Vec2::from_angle(other.angle) * other.size_negative;

        let d_a = a_end - a_start;
        let d_b = b_end - b_start;

        let denom = d_a.perp_dot(d_b);
        if denom.abs() < f32::EPSILON {
            return None;
        }

        let diff = b_start - a_start;
        let t = diff.perp_dot(d_b) / denom;
        let s = diff.perp_dot(d_a) / denom;

        if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&s) {
            Some(a_start + t * d_a)
        } else {
            None
        }
    }
}

#[derive(Component)]
pub struct CutRenderMesh {
    mesh_handle: Handle<Mesh>,
    material_handle: Handle<StandardMaterial>,
}

#[derive(Component, Default)]
pub struct Cuttable {
    cuts: Vec<Cut>,
}

impl Default for Knife {
    fn default() -> Self {
        Knife {
            in_hand: false,
            pressed: false,
            first_click: true,
            angle: 0.0,
            on_sheet: false,
        }
    }
}

fn pick_up_knife(
    mut knife: Single<(Entity, &mut Transform, &mut Knife)>,
    buttons: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    window_query: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    spatial_query: SpatialQuery,
    children: Query<&Children>,
) {
    if knife.2.in_hand {
        return;
    }
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let position = if let Some(touch) = touches.iter().next() {
        touch.position()
    } else if let Some(cursor_pos) = window_query.cursor_position() {
        cursor_pos
    } else {
        return;
    };
    let Ok(ray) = camera_query.0.viewport_to_world(&camera_query.1, position) else {
        return;
    };

    let Some(_hit) = spatial_query.cast_ray_predicate(
        ray.origin,
        ray.direction,
        50.0,
        false,
        &SpatialQueryFilter::default(),
        &|entity| {
            // If entity is not the knife, skip
            entity == knife.0 || !children.iter_descendants(knife.0).any(|e| e == entity)
        },
    ) else {
        return;
    };

    knife.2.in_hand = true;
    knife.1.rotation = Quat::from_axis_angle(Vec3::X, PI);
}

fn move_knife(
    mut knife: Single<(Entity, &mut Transform, &mut Knife)>,
    touches: Res<Touches>,
    window_query: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    spatial_query: SpatialQuery,
    children: Query<&Children>,
    mut sheet: Single<(Entity, &mut Cuttable), (With<LeatherSheet>, Without<Knife>)>,
) {
    if !knife.2.in_hand {
        return;
    }

    let position = if let Some(touch) = touches.iter().next() {
        touch.position()
    } else if let Some(cursor_pos) = window_query.cursor_position() {
        cursor_pos
    } else {
        return;
    };
    let Ok(ray) = camera_query.0.viewport_to_world(&camera_query.1, position) else {
        return;
    };

    if knife.2.pressed {
        let Some(t) = ray.intersect_plane(
            Vec3::new(0.0, TABLE_HEIGHT, 0.0),
            InfinitePlane3d { normal: Dir3::Y },
        ) else {
            return;
        };

        let hit_point = ray.origin + ray.direction * t;

        knife.1.translation.y = TABLE_HEIGHT;
        let offset = knife.1.translation.xz() - hit_point.xz();
        let angle = knife.2.angle;
        let direction_of_interest = Vec2::from_angle(angle);
        let offset = -(offset.dot(direction_of_interest) * 2.0).clamp(-PI / 5.0, PI / 5.0);

        knife.1.rotation = Quat::default();
        knife.1.rotate_local_x(PI);
        knife.1.rotate_local_y(angle);
        knife.1.rotate_local_z(offset);

        let progress = offset / (PI / 5.0);
        let Some(cut) = sheet.1.cuts.last_mut() else {
            return;
        };
        cut.size_positive = cut.size_positive.max(progress * MAX_CUT_DISTANCE);
        cut.size_negative = cut.size_negative.max(-progress * MAX_CUT_DISTANCE);
        return;
    } else {
        let Some(hit) = spatial_query.cast_ray_predicate(
            ray.origin,
            ray.direction,
            50.0,
            false,
            &SpatialQueryFilter::default(),
            &|entity| {
                // If entity is the knife itself or a descendant of the knife, skip
                entity != knife.0 && !children.iter_descendants(knife.0).any(|e| e == entity)
            },
        ) else {
            return;
        };

        let hit_point = ray.origin + ray.direction * hit.distance;

        knife.1.translation = hit_point;
        knife.1.rotation = Quat::default();
        knife.1.rotate_local_x(PI);
        let angle = knife.2.angle;
        knife.1.rotate_local_y(angle);
        knife.2.on_sheet =
            hit.entity == sheet.0 || children.iter_descendants(sheet.0).any(|e| e == hit.entity);
    }
}

fn rotate_knife(mut knife: Single<&mut Knife>, mouse_scroll: Res<AccumulatedMouseScroll>) {
    if !knife.in_hand || knife.pressed {
        return;
    }
    knife.angle += mouse_scroll.delta.y * PI / 32.0;
}

fn use_knife(
    mut knife: Single<(Entity, &mut Transform, &mut Knife)>,
    mut sheet: Single<(&mut Cuttable, &Transform), (With<LeatherSheet>, Without<Knife>)>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if !knife.2.in_hand {
        return;
    }

    if !buttons.pressed(MouseButton::Left) {
        knife.2.first_click = false;
    }

    if knife.2.first_click || knife.2.pressed == buttons.pressed(MouseButton::Left) {
        return;
    }

    knife.2.pressed = buttons.pressed(MouseButton::Left);

    if knife.2.pressed && knife.2.on_sheet {
        let center = (knife.1.translation - sheet.1.translation).xz();
        sheet.0.cuts.push(Cut {
            center,
            angle: knife.2.angle,
            size_positive: 0.0,
            size_negative: 0.0,
        });
    }
}

pub fn clear_cuts(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cut_render_query: Query<(Entity, &CutRenderMesh)>,
) {
    for (
        entity,
        CutRenderMesh {
            mesh_handle: cut_mesh_handle,
            material_handle: cut_material_handle,
        },
    ) in cut_render_query.iter()
    {
        meshes.remove(cut_mesh_handle);
        materials.remove(cut_material_handle);
        commands.entity(entity).despawn();
    }
}

pub fn render_cuts(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sheet_query: Single<(&Cuttable, &Transform)>,
) {
    if sheet_query.0.cuts.len() == 0 {
        return;
    }

    let mesh = create_cutting_mesh(&sheet_query.0, &sheet_query.1);
    let mesh_handle: Handle<Mesh> = meshes.add(mesh);

    let material_handle: Handle<StandardMaterial> = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        unlit: true,
        ..default()
    });

    commands.spawn((
        Mesh3d(mesh_handle.clone()),
        MeshMaterial3d(material_handle.clone()),
        CutRenderMesh {
            mesh_handle: mesh_handle.clone(),
            material_handle: material_handle.clone(),
        },
    ));
}

fn create_cutting_mesh(cuts: &Cuttable, sheet_transform: &Transform) -> Mesh {
    let mut positions: Vec<Vec3> = Vec::new();
    let mut normals: Vec<Vec3> = Vec::new();
    let mut uvs: Vec<Vec2> = Vec::new();

    let mut indices: Vec<u32> = Vec::new();

    for cut in &cuts.cuts {
        let first_position = sheet_transform.translation
            + Vec3::Y * SHEET_HEIGHT
            + (cut.center + Vec2::from_angle(cut.angle) * cut.size_positive)
                .xxy()
                .with_y(0.0);
        let second_position = sheet_transform.translation
            + Vec3::Y * SHEET_HEIGHT
            + (cut.center - Vec2::from_angle(cut.angle) * cut.size_negative)
                .xxy()
                .with_y(0.0);

        let segment_as_vec = second_position - first_position;

        let to_camera = Vec3::Y;
        let perp = segment_as_vec.cross(to_camera).normalize();
        let top_left = first_position + perp * CUT_THICKNESS / 2.0;
        let top_right = first_position - perp * CUT_THICKNESS / 2.0;

        let bottom_left = second_position + perp * CUT_THICKNESS / 2.0;
        let bottom_right = second_position - perp * CUT_THICKNESS / 2.0;

        let top_left_index = positions.len();
        let top_right_index = top_left_index + 1;
        let bottom_left_index = top_left_index + 2;
        let bottom_right_index = top_left_index + 3;

        positions.push(top_left);
        positions.push(top_right);
        positions.push(bottom_left);
        positions.push(bottom_right);

        normals.push(to_camera);
        normals.push(to_camera);
        normals.push(to_camera);
        normals.push(to_camera);

        uvs.push(Vec2::new(0.0, 0.0));
        uvs.push(Vec2::new(1.0, 0.0));
        uvs.push(Vec2::new(0.0, 1.0));
        uvs.push(Vec2::new(1.0, 1.0));

        // triangle 1
        indices.push(bottom_left_index.try_into().unwrap());
        indices.push(top_right_index.try_into().unwrap());
        indices.push(top_left_index.try_into().unwrap());

        // triangle 2
        indices.push(bottom_left_index.try_into().unwrap());
        indices.push(bottom_right_index.try_into().unwrap());
        indices.push(top_right_index.try_into().unwrap());
    }

    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        positions
            .iter()
            .map(|position| position.to_array())
            .collect::<Vec<_>>(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        normals
            .iter()
            .map(|normal| normal.to_array())
            .collect::<Vec<_>>(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_UV_0,
        uvs.iter().map(|uv| uv.to_array()).collect::<Vec<_>>(),
    )
    .with_inserted_indices(Indices::U32(indices));

    mesh
}

fn find_best_loop(cuts: &Cuttable) -> Option<Vec<Vec2>> {
    let mut connected_cuts_per_cut: Vec<Vec<(usize, Vec2)>> = Vec::new();

    let n = cuts.cuts.len();
    for i in 0..n {
        connected_cuts_per_cut.push(Vec::new());
    }
    for i in 0..n {
        for j in 0..n {
            let cut_a = &cuts.cuts[i];
            let cut_b = &cuts.cuts[j];
            if let Some(intersection) = cut_a.intersect(&cut_b) {
                connected_cuts_per_cut[i].push((j, intersection));
                connected_cuts_per_cut[j].push((i, intersection));
            }
        }
    }

    let mut best = usize::MAX;
    let mut best_loop: Option<Vec<Vec2>> = None;

    for start in 0..n {
        let mut dist: Vec<i32> = Vec::new();
        let mut parent: Vec<i32> = Vec::new();
        let mut parent_edge: Vec<Vec2> = vec![Vec2::ZERO; n];
        let mut queue = VecDeque::new();

        dist.resize(n, -1);
        parent.resize(n, -1);
        dist[start] = 0;
        queue.push_back(start);

        while let Some(u) = queue.pop_front() {
            for &(v, edge_pos) in &connected_cuts_per_cut[u] {
                if dist[v] == -1 {
                    dist[v] = dist[u] + 1;
                    parent[v] = u as i32;
                    parent_edge[v] = edge_pos;
                    queue.push_back(v);
                } else if parent[u] != v as i32 && parent[v] != u as i32 {
                    let cycle_len = dist[u] + dist[v] + 1;
                    if (cycle_len as usize) < best {
                        let mut left: Vec<Vec2> = Vec::new();
                        let mut right: Vec<Vec2> = Vec::new();

                        let (mut a, mut b) = (u, v);
                        let (mut da, mut db) = (dist[u], dist[v]);

                        while da > db {
                            left.push(parent_edge[a]);
                            a = parent[a] as usize;
                            da -= 1;
                        }
                        while db > da {
                            right.push(parent_edge[b]);
                            b = parent[b] as usize;
                            db -= 1;
                        }

                        while a != b {
                            left.push(parent_edge[a]);
                            a = parent[a] as usize;
                            right.push(parent_edge[b]);
                            b = parent[b] as usize;
                        }

                        let mut loop_positions: Vec<Vec2> =
                            Vec::with_capacity(left.len() + right.len() + 1);
                        loop_positions.extend(left.iter().rev().copied());
                        loop_positions.push(edge_pos); // the non-tree edge u-v
                        loop_positions.extend(right.iter().copied());
                        if signed_area(&loop_positions) < 0.0 {
                            loop_positions.reverse();
                        }

                        if signed_area(&loop_positions) > f32::EPSILON
                            && contains_point(&loop_positions, Vec2::ZERO)
                        {
                            best_loop = Some(loop_positions);
                            best = cycle_len as usize;
                        }
                    }
                }
            }
        }
    }

    best_loop
}

fn signed_area(points: &[Vec2]) -> f32 {
    let mut area = 0.0;
    for i in 0..points.len() {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        area += a.x * b.y - b.x * a.y;
    }
    area / 2.0
}

fn contains_point(polygon: &[Vec2], point: Vec2) -> bool {
    let mut winding = 0;
    for i in 0..polygon.len() {
        let a = polygon[i];
        let b = polygon[(i + 1) % polygon.len()];
        if (a.y > point.y) != (b.y > point.y) {
            let x = a.x + (point.y - a.y) / (b.y - a.y) * (b.x - a.x);
            if x > point.x {
                winding += if b.y > a.y { 1 } else { -1 };
            }
        }
    }
    winding != 0
}

fn triangulate(polygon: &[Vec2]) -> Option<Vec<[u32; 3]>> {
    let n = polygon.len();
    if n < 3 {
        return None;
    }

    let convex_corner = |a: Vec2, b: Vec2, c: Vec2| (b - a).perp_dot(c - b) > f32::EPSILON;
    let in_triangle = |p: Vec2, tri: [Vec2; 3]| {
        (tri[1] - tri[0]).perp_dot(p - tri[0]) >= -f32::EPSILON
            && (tri[2] - tri[1]).perp_dot(p - tri[1]) >= -f32::EPSILON
            && (tri[0] - tri[2]).perp_dot(p - tri[2]) >= -f32::EPSILON
    };

    let mut triangles = Vec::with_capacity((n - 2) * 3);
    let mut remaining: Vec<usize> = (0..n).collect();

    while remaining.len() > 3 {
        let m = remaining.len();

        let mut ear = None;
        for r in 0..m {
            let prev = remaining[(r + m - 1) % m];
            let cur = remaining[r];
            let next = remaining[(r + 1) % m];
            if !convex_corner(polygon[prev], polygon[cur], polygon[next]) {
                continue;
            }
            let tri = [polygon[prev], polygon[cur], polygon[next]];
            if remaining
                .iter()
                .all(|&j| (j == prev || j == cur || j == next) || !in_triangle(polygon[j], tri))
            {
                ear = Some((r, prev, cur, next));
                break;
            }
        }

        if ear.is_none() {
            for r in 0..m {
                let prev = remaining[(r + m - 1) % m];
                let cur = remaining[r];
                let next = remaining[(r + 1) % m];
                if convex_corner(polygon[prev], polygon[cur], polygon[next]) {
                    ear = Some((r, prev, cur, next));
                    break;
                }
            }
        }

        let (r, prev, cur, next) = ear?;
        triangles.push([prev as u32, cur as u32, next as u32]);
        remaining.remove(r);
    }

    triangles.push([
        remaining[0] as u32,
        remaining[1] as u32,
        remaining[2] as u32,
    ]);
    Some(triangles)
}

fn create_extrusion_mesh(polygon: &[Vec2], depth: f32) -> Option<Mesh> {
    let half_depth = depth / 2.0;
    let cap_triangles = triangulate(polygon)?;

    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    for vertex in polygon {
        min = min.min(*vertex);
        max = max.max(*vertex);
    }
    let size = (max - min).max(Vec2::splat(f32::EPSILON));

    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(polygon.len() * 4);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(polygon.len() * 4);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(polygon.len() * 4);
    let mut indices: Vec<u32> = Vec::with_capacity(cap_triangles.len() * 6);

    for vertex in polygon {
        positions.push([vertex.x, vertex.y, half_depth]);
        normals.push([0.0, 0.0, 1.0]);
        let uv = (*vertex - min) / size * 0.5;
        uvs.push([uv.x, uv.y]);
    }
    for [a, b, c] in &cap_triangles {
        indices.extend_from_slice(&[*a, *b, *c]);
    }

    let back_cap_offset = positions.len() as u32;
    for vertex in polygon {
        positions.push([vertex.x, vertex.y, -half_depth]);
        normals.push([0.0, 0.0, -1.0]);
        let uv = (*vertex - min) / size * 0.5;
        uvs.push([uv.x + 0.5, uv.y]);
    }
    for [a, b, c] in &cap_triangles {
        indices.extend_from_slice(&[
            back_cap_offset + b,
            back_cap_offset + a,
            back_cap_offset + c,
        ]);
    }

    for r in 0..polygon.len() {
        let a = polygon[r];
        let b = polygon[(r + 1) % polygon.len()];
        let base = positions.len() as u32;
        let u_a = r as f32 / polygon.len() as f32;
        let u_b = (r + 1) as f32 / polygon.len() as f32;
        let normal = Vec3::new(b.y - a.y, a.x - b.x, 0.0)
            .normalize_or_zero()
            .to_array();

        for (layer, z) in [(0.0, half_depth), (1.0, -half_depth)] {
            positions.push([a.x, a.y, z]);
            positions.push([b.x, b.y, z]);
            let v = 0.5 + 0.5 * layer;
            uvs.push([u_a, v]);
            uvs.push([u_b, v]);
            normals.push(normal);
            normals.push(normal);
        }

        indices.extend_from_slice(&[base, base + 2, base + 1, base + 1, base + 2, base + 3]);
    }

    Some(
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_indices(Indices::U32(indices))
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs),
    )
}

fn cut_leather_sheet(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    input: Res<ButtonInput<KeyCode>>,
    sheet_query: Single<(&Cuttable, &Transform, Entity)>,
    decals: Query<Entity, With<ForwardDecal>>,
    assets: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
    mut knife: Single<&mut CommandSupport, With<Knife>>,
) {
    if !input.just_pressed(KeyCode::Enter) {
        return;
    }

    if let Some(polygon) = find_best_loop(sheet_query.0) {
        if let Some(mesh) = create_extrusion_mesh(&polygon, SHEET_HEIGHT / 2.0) {
            let mesh = meshes.add(mesh);

            let leather_tex = assets.load("leather_tinted.png");
            let material_handle: Handle<StandardMaterial> = materials.add(StandardMaterial {
                base_color_texture: Some(leather_tex.clone()),
                ..default()
            });
            let mut transform = sheet_query.1.clone();
            transform.rotate_z(PI / 2.0);
            transform.rotate_y(PI / 2.0);
            transform.translation.y += SHEET_HEIGHT / 4.0;

            let mut left_destination = transform;
            left_destination.translation.y -= SHEET_HEIGHT / 2.0;
            left_destination.translation.x -= 0.15;

            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material_handle.clone()),
                transform,
                Sole { left: true },
                CommandSupport {
                    command_queue: VecDeque::from([Command::new_transition(
                        left_destination,
                        1.0,
                        EaseFunction::SmoothStep,
                    )]),
                },
            ));

            transform.translation.y -= SHEET_HEIGHT / 2.0;
            let mut right_destination = transform;
            right_destination.translation.x += 0.15;
            right_destination.rotate_z(PI);

            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material_handle),
                transform,
                Sole { left: false },
                CommandSupport {
                    command_queue: VecDeque::from([Command::new_transition(
                        right_destination,
                        1.0,
                        EaseFunction::SmoothStep,
                    )]),
                },
            ));
            commands.entity(sheet_query.2).insert(Visibility::Hidden);

            for decal in decals {
                commands.entity(decal).despawn();
            }

            let mut knife_destination = Transform::from_xyz(-0.55, TABLE_HEIGHT, -1.2);
            knife_destination.rotate_local_x(PI / 2.0);
            knife.queue(Command::new_transition(
                knife_destination,
                2.0,
                EaseFunction::SmoothStep,
            ));
            next_state.set(GameState::Assembly);
        }
    }
}
