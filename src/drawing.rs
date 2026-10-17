use crate::title_screen::GameState;
use avian3d::parry::glamx::Vec3;
use avian3d::prelude::{SpatialQuery, SpatialQueryFilter};
use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::{AssetServer, Assets, Handle};
use bevy::camera::Camera;
use bevy::color::Color;
use bevy::input::ButtonInput;
use bevy::material::AlphaMode;
use bevy::math::Quat;
use bevy::pbr::decal::{ForwardDecal, ForwardDecalMaterial, ForwardDecalMaterialExt};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{
    Children, Commands, Component, Entity, GlobalTransform, IntoScheduleConfigs, MouseButton,
    Query, Res, ResMut, Resource, Single, Touches, Transform, Window, With, default, in_state,
};
use bevy::window::PrimaryWindow;
use std::f32::consts::PI;

pub struct DrawingPlugin;

#[derive(Component)]
pub struct Pencil {
    draw_normal: Vec3,
}

impl Pencil {
    pub fn default() -> Self {
        Pencil {
            draw_normal: Vec3::ZERO,
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
}

fn draw(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    spray_material: Res<SprayMaterial>,
    pencil: Single<(&Transform, &Pencil)>,
) {
    if buttons.pressed(MouseButton::Left) {
        let mut spray_transform = Transform::from_translation(pencil.0.translation)
            .with_scale(Vec3::ONE * 0.03)
            .with_rotation(Quat::from_rotation_arc(Vec3::Y, pencil.1.draw_normal));
        spray_transform.rotate_local_y(rand::random::<f32>() * PI * 2.0);

        commands.spawn((
            ForwardDecal,
            MeshMaterial3d(spray_material.material.clone()),
            spray_transform,
        ));
    }
}
