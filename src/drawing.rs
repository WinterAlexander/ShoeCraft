use crate::decal::{Decal, Decalable, spray_decal};
use avian3d::parry::glamx::Vec3;
use bevy::app::{App, Plugin, Update};
use bevy::asset::{AssetServer, Assets};
use bevy::camera::{Camera, Camera3d};
use bevy::color::Color;
use bevy::input::ButtonInput;
use bevy::material::AlphaMode;
use bevy::mesh::Mesh3d;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{
    Commands, Entity, GlobalTransform, MouseButton, Query, Res, ResMut, Single, Touches, Transform,
    Window, With, Without, default,
};
use bevy::window::PrimaryWindow;

pub struct DrawingPlugin;

impl Plugin for DrawingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (make_all_decalable, draw));
    }
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
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
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
            let spray_transform = Transform::from_translation(ray.origin)
                .looking_to(ray.direction, camera_query.1.up())
                .with_scale(Vec3::ONE * 0.01 + Vec3::Z * 50.0);

            let spray_material = standard_materials.add(StandardMaterial {
                base_color: Color::srgb(0.1, 0.1, 0.1),
                base_color_texture: Some(assets.load("pencil_mark.png").clone()),
                alpha_mode: AlphaMode::Mask(0.5),
                ..default()
            });
            spray_decal(&mut commands, spray_material.clone(), spray_transform);
        }
    }
}
