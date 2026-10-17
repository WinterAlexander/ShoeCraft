use crate::decal::{Decal, Decalable, spray_decal};
use crate::title_screen::GameState;
use avian3d::parry::glamx::Vec3;
use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::{AssetServer, Assets, Handle};
use bevy::camera::{Camera, Camera3d};
use bevy::color::Color;
use bevy::input::ButtonInput;
use bevy::material::AlphaMode;
use bevy::math::Quat;
use bevy::mesh::Mesh3d;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{
    Commands, Entity, GlobalTransform, IntoScheduleConfigs, MouseButton, Query, Res, ResMut,
    Resource, Single, Touches, Transform, Window, With, Without, default, in_state,
};
use bevy::window::PrimaryWindow;
use std::f32::consts::PI;

pub struct DrawingPlugin;

impl Plugin for DrawingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_spray_material);
        app.add_systems(
            Update,
            (make_all_decalable, draw).run_if(in_state(GameState::Drawing)),
        );
    }
}

#[derive(Resource)]
pub struct SprayMaterial {
    material: Handle<StandardMaterial>,
}

fn init_spray_material(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(SprayMaterial {
        material: standard_materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.1),
            base_color_texture: Some(assets.load("pencil_mark.png").clone()),
            alpha_mode: AlphaMode::Mask(0.5),
            ..default()
        }),
    })
}

fn make_all_decalable(
    mut commands: Commands,
    entities: Query<Entity, (With<Mesh3d>, Without<Decal>, Without<Decalable>)>,
) {
    for entity in entities.iter() {
        commands.entity(entity).insert(Decalable::default());
    }
}

fn draw(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    window_query: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    spray_material: Res<SprayMaterial>,
) {
    if buttons.pressed(MouseButton::Left) {
        let position = if let Some(touch) = touches.iter().next() {
            touch.position()
        } else if let Some(cursor_pos) = window_query.cursor_position() {
            cursor_pos
        } else {
            return;
        };

        if let Ok(ray) = camera_query.0.viewport_to_world(&camera_query.1, position) {
            let mut spray_transform = Transform::from_translation(ray.origin)
                .looking_to(ray.direction, camera_query.1.up())
                .with_scale(Vec3::ONE * 0.01 + Vec3::Z * 50.0);
            spray_transform.rotate_local_z(rand::random::<f32>() * PI * 2.0);

            spray_decal(
                &mut commands,
                spray_material.material.clone(),
                spray_transform,
            );
        }
    }
}
