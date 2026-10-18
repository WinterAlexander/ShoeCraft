use crate::command::{Command, CommandSupport};
use crate::customer::{AnimatedCustomer, CustomerCollider};
use crate::shop::LeatherSheet;
use crate::title_screen::GameState;
use avian3d::parry::glamx::Vec3;
use avian3d::prelude::{SpatialQuery, SpatialQueryFilter};
use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::{AssetServer, Assets, Handle};
use bevy::camera::{Camera, Camera3d};
use bevy::color::Color;
use bevy::input::ButtonInput;
use bevy::material::AlphaMode;
use bevy::math::{Affine3A, Quat};
use bevy::pbr::decal::{ForwardDecal, ForwardDecalMaterial, ForwardDecalMaterialExt};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{
    Children, Commands, Component, DelayedCommandsExt, EaseFunction, Entity, GlobalTransform,
    IntoScheduleConfigs, MouseButton, Name, NextState, Query, Res, ResMut, Resource, Single,
    Touches, Transform, Window, With, Without, default, in_state,
};
use bevy::window::PrimaryWindow;
use std::f32::consts::PI;

pub struct DrawingPlugin;

#[derive(Component)]
pub struct Pencil {
    draw_normal: Vec3,
    on_sheet: bool,
    on_customer: bool,
}

#[derive(Component)]
pub struct ValidDrawing {
    sheet_offset: Transform,
}

#[derive(Resource, Default)]
pub struct DrawnOnCustomerTracker {
    draw_count: i32,
}

impl Pencil {
    pub fn default() -> Self {
        Pencil {
            draw_normal: Vec3::ZERO,
            on_sheet: false,
            on_customer: false,
        }
    }
}

impl Plugin for DrawingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_spray_material);
        app.add_systems(
            Update,
            (draw, move_pencil).run_if(in_state(GameState::Drawing)),
        );
        app.add_systems(Update, keep_drawings_on_sheet);
        app.insert_resource(DrawnOnCustomerTracker::default());
    }
}

#[derive(Resource)]
pub struct SprayMaterial {
    material: Handle<ForwardDecalMaterial<StandardMaterial>>,
}

fn init_spray_material(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut decal_materials: ResMut<Assets<ForwardDecalMaterial<StandardMaterial>>>,
) {
    commands.insert_resource(SprayMaterial {
        material: decal_materials.add(ForwardDecalMaterial {
            base: StandardMaterial {
                base_color: Color::srgb(0.1, 0.1, 0.1),
                base_color_texture: Some(assets.load("pencil_mark.png").clone()),
                alpha_mode: AlphaMode::Mask(0.5),
                ..default()
            },
            extension: ForwardDecalMaterialExt {
                depth_fade_factor: 1.0,
            },
        }),
    })
}

fn move_pencil(
    mut pencil: Single<(Entity, &mut Transform, &mut Pencil)>,
    touches: Res<Touches>,
    window_query: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    spatial_query: SpatialQuery,
    children: Query<&Children>,
    sheet: Single<Entity, With<LeatherSheet>>,
    customer: Single<Entity, (With<CustomerCollider>, Without<LeatherSheet>)>,
) {
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

    let Some(hit) = spatial_query.cast_ray_predicate(
        ray.origin,
        ray.direction,
        50.0,
        false,
        &SpatialQueryFilter::default(),
        &|entity| {
            // If entity is pencil itself or a descendant of pencil, skip
            entity != pencil.0 && !children.iter_descendants(pencil.0).any(|e| e == entity)
        },
    ) else {
        return;
    };

    let hit_point = ray.origin + ray.direction * hit.distance;

    pencil.1.translation = hit_point;
    pencil.1.rotation = Quat::from_axis_angle(Vec3::Z, PI * 5.0 / 4.0);
    pencil.2.draw_normal = hit.normal;
    pencil.2.on_sheet =
        hit.entity == *sheet || children.iter_descendants(*sheet).any(|e| e == hit.entity);
    pencil.2.on_customer = hit.entity == *customer
        || children
            .iter_descendants(*customer)
            .any(|e| e == hit.entity);
}

fn draw(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    spray_material: Res<SprayMaterial>,
    pencil: Single<(&Transform, &Pencil)>,
    sheet: Single<&Transform, (With<LeatherSheet>, Without<Pencil>)>,
    mut drawn_on_customer_tracker: ResMut<DrawnOnCustomerTracker>,
    mut camera: Single<&mut CommandSupport, With<Camera3d>>,
    mut customer: Single<&mut AnimatedCustomer>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        customer.anger_cancel_click = false;
    }

    if buttons.pressed(MouseButton::Left) && !customer.anger_cancel_click {
        let mut transform = Transform::from_translation(pencil.0.translation)
            .with_scale(Vec3::ONE * 0.03)
            .with_rotation(Quat::from_rotation_arc(Vec3::Y, pencil.1.draw_normal));
        transform.rotate_local_y(rand::random::<f32>() * PI * 2.0);
        let mut entity_commands = commands.spawn((
            ForwardDecal,
            MeshMaterial3d(spray_material.material.clone()),
            transform,
        ));

        if pencil.1.on_sheet {
            entity_commands.insert(ValidDrawing {
                sheet_offset: Transform::from_matrix(sheet.compute_affine().inverse().into())
                    * transform,
            });
        }

        if pencil.1.on_customer {
            drawn_on_customer_tracker.draw_count += 1;
            if drawn_on_customer_tracker.draw_count > 2 {
                customer.anger_cancel_click = true;
                if camera.command_queue.len() == 0 {
                    customer.not_angry_timer = 1.5;
                    //customer.angry_timer = 2.5;
                    camera.command_queue.extend([
                        Command::new_transition(
                            Transform::from_xyz(-1.5, 1.0, -0.75)
                                .looking_at(Vec3::new(-1.3, 1.0, -0.75), Vec3::Y),
                            1.0,
                            EaseFunction::SmoothStep,
                        ),
                        Command::Wait(2.0),
                        Command::new_transition(
                            Transform::from_xyz(-1.5, 1.0, -0.75)
                                .looking_at(Vec3::new(-1.3, 0.0, -0.75), Vec3::Y),
                            1.0,
                            EaseFunction::SmoothStep,
                        ),
                    ]);
                }
                drawn_on_customer_tracker.draw_count = 0;
            }
        } else {
            drawn_on_customer_tracker.draw_count = 0;
        }
    }
}

fn keep_drawings_on_sheet(
    buttons: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut sheet_query: Single<&mut Transform, With<LeatherSheet>>,
    drawing_query: Query<(&mut Transform, &ValidDrawing), (Without<LeatherSheet>)>,
) {
    if buttons.just_pressed(MouseButton::Right) {
        next_state.set(GameState::DrawingToCutting);
        sheet_query.translation += Vec3::new(0.0, 0.6148, -0.6148);
        for (mut trans, drawing_spot) in drawing_query {
            *trans = **sheet_query * drawing_spot.sheet_offset;
        }
    }
}
