use crate::shop::LeatherSheet;
use crate::title_screen::GameState;
use avian3d::parry::glamx::{Quat, Vec3};
use avian3d::prelude::{SpatialQuery, SpatialQueryFilter};
use bevy::app::{App, Plugin, Update};
use bevy::asset::{Assets, Handle, RenderAssetUsages};
use bevy::camera::Camera;
use bevy::color::Color;
use bevy::input::ButtonInput;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::math::{Dir3, Vec2, Vec3Swizzles};
use bevy::mesh::{Indices, Mesh, Mesh3d, PrimitiveTopology};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{
    Children, Commands, Component, Entity, GlobalTransform, InfinitePlane3d, IntoScheduleConfigs,
    MouseButton, Query, Res, ResMut, Single, Time, Touches, Transform, Window, With, Without,
    default, in_state,
};
use bevy::window::PrimaryWindow;
use std::f32::consts::PI;

const TABLE_HEIGHT: f32 = 0.617;
const MAX_CUT_DISTANCE: f32 = 0.135;
pub const CUT_PRISM_BASE: i32 = 4;
pub const CUT_THICKNESS: f32 = 0.01;

pub struct KnifePlugin;

impl Plugin for KnifePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (pick_up_knife, move_knife, rotate_knife, use_knife)
                .run_if(in_state(GameState::Cutting)),
        );

        app.add_systems(Update, clear_cuts);
        app.add_systems(Update, render_cuts.after(clear_cuts));
    }
}

#[derive(Component)]
pub struct Knife {
    in_hand: bool,
    pressed: bool,
    first_click: bool,
    angle: f32,
}

pub struct Cut {
    center: Vec2,
    angle: f32,
    size_positive: f32,
    size_negative: f32,
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
        let offset = -offset.dot(direction_of_interest) * 2.0;

        knife.1.rotation = Quat::default();
        knife.1.rotate_local_x(PI);
        knife.1.rotate_local_y(angle);
        knife.1.rotate_local_z(offset.clamp(-PI / 5.0, PI / 5.0));
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
    }
}

fn rotate_knife(mut knife: Single<&mut Knife>, mouse_scroll: Res<AccumulatedMouseScroll>) {
    if !knife.in_hand || knife.pressed {
        return;
    }
    knife.angle += mouse_scroll.delta.y * PI / 8.0;
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

    if knife.2.pressed {
        let center = (knife.1.translation - sheet.1.translation).xz();
        sheet.0.cuts.push(Cut {
            center,
            angle: knife.2.angle,
            size_positive: MAX_CUT_DISTANCE,
            size_negative: MAX_CUT_DISTANCE,
        });
    }
}

pub fn clear_cuts(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    web_render_segments_query: Query<(Entity, &CutRenderMesh)>,
) {
    for (
        web_render_segment_entity,
        CutRenderMesh {
            mesh_handle: web_segment_mesh_handle,
            material_handle: web_segment_material_handle,
        },
    ) in web_render_segments_query.iter()
    {
        meshes.remove(web_segment_mesh_handle);
        materials.remove(web_segment_material_handle);
        commands.entity(web_render_segment_entity).despawn();
    }
}

pub fn render_cuts(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sheet_query: Single<(&Cuttable, &Transform)>,
) {
    let mesh = create_web_mesh(&sheet_query.0, &sheet_query.1);
    let mesh_handle: Handle<Mesh> = meshes.add(mesh);

    let material_handle: Handle<StandardMaterial> = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.8),
        // unlit: true,
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

fn create_web_mesh(cuts: &Cuttable, sheet_transform: &Transform) -> Mesh {
    let mut positions: Vec<Vec3> = Vec::new();
    let mut normals: Vec<Vec3> = Vec::new();
    let mut uvs: Vec<Vec2> = Vec::new();

    let mut indices: Vec<u32> = Vec::new();

    for cut in &cuts.cuts {
        let first_position = sheet_transform.translation
            + (cut.center + Vec2::from_angle(cut.angle) * cut.size_positive).extend(0.0);
        let second_position = sheet_transform.translation
            + (cut.center - Vec2::from_angle(cut.angle) * cut.size_negative).extend(0.0);

        let segment_as_vec = second_position - first_position;

        for i in 0..CUT_PRISM_BASE {
            let quat = Quat::from_axis_angle(
                segment_as_vec.normalize(),
                i as f32 / CUT_PRISM_BASE as f32 * 2.0 * PI,
            );
            let normal = quat.mul_vec3(Vec3::new(0.0, 0.0, 1.0));

            let top_left_index = positions.len();
            let top_right_index = top_left_index + 1;
            let bottom_left_index = top_left_index + 2;
            let bottom_right_index = top_left_index + 3;

            let perp = segment_as_vec.cross(normal).normalize();
            let top_left =
                first_position + perp * CUT_THICKNESS / 2.0 + normal * CUT_THICKNESS / 2.0;
            let top_right =
                first_position - perp * CUT_THICKNESS / 2.0 + normal * CUT_THICKNESS / 2.0;

            let bottom_left =
                second_position + perp * CUT_THICKNESS / 2.0 + normal * CUT_THICKNESS / 2.0;
            let bottom_right =
                second_position - perp * CUT_THICKNESS / 2.0 + normal * CUT_THICKNESS / 2.0;

            positions.push(top_left);
            positions.push(top_right);
            positions.push(bottom_left);
            positions.push(bottom_right);

            normals.push(normal);
            normals.push(normal);
            normals.push(normal);
            normals.push(normal);

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
