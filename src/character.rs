use crate::scene_loader::SceneElement;
use crate::thirdperson_camera_controller::CameraTarget;
use avian3d::prelude::{
    ColliderConstructor, ColliderConstructorHierarchy, Forces, RigidBody, WriteRigidBodyForces,
};
use bevy::animation::AnimationPlayer;
use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::{AssetServer, Assets};
use bevy::gltf::GltfAssetLabel;
use bevy::input::ButtonInput;
use bevy::math::Vec3;
use bevy::math::ops::sqrt;
use bevy::prelude::{
    AnimationGraph, AnimationGraphHandle, Children, Commands, Component, KeyCode, On, Query, Res,
    ResMut, Transform, WorldAssetRoot,
};
use bevy::world_serialization::WorldInstanceReady;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player);
        app.add_systems(Update, control_player);
    }
}

#[derive(Component)]
pub struct Controllable {
    walk_speed: f32,
    sprint_speed: f32,
}

impl Default for Controllable {
    fn default() -> Self {
        Controllable {
            walk_speed: 5.0,
            sprint_speed: 7.0,
        }
    }
}

fn spawn_player(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let (graph, node) = AnimationGraph::from_clip(
        asset_server.load(GltfAssetLabel::Animation(0).from_asset("character.glb")),
    );

    let graph = graphs.add(graph);
    commands
        .spawn((
            WorldAssetRoot(asset_server.load("character.glb#Scene0")),
            ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh),
            SceneElement,
            Transform::from_xyz(
                rand::random::<f32>() * 100.0 - 50.0,
                10.0,
                rand::random::<f32>() * 100.0 - 50.0,
            ),
            RigidBody::Dynamic,
            Controllable::default(),
            CameraTarget,
        ))
        .observe(
            move |ready: On<WorldInstanceReady>,
                  mut commands: Commands,
                  children: Query<&Children>,
                  mut players: Query<&mut AnimationPlayer>| {
                for child in children.iter_descendants(ready.entity) {
                    if let Ok(mut player) = players.get_mut(child) {
                        player.play(node).repeat();
                        commands
                            .entity(child)
                            .insert(AnimationGraphHandle(graph.clone()));
                    }
                }
            },
        );
}

fn control_player(
    mut controllable_forces: Query<(Forces, &Controllable)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let mut dir = Vec3::new(0.0, 0.0, 0.0);
    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        dir.z += 1.0;
    } else if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        dir.z -= 1.0;
    } else if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    } else if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        dir.x += 1.0;
    }

    let len2 = dir.length_squared();

    if len2 != 0.0 && len2 != 1.0 {
        dir /= sqrt(2f32);
    }

    let sprint = input.pressed(KeyCode::ShiftLeft) && input.pressed(KeyCode::ShiftRight);

    for (mut forces, controllable) in &mut controllable_forces {
        let dir = dir
            * if sprint {
                controllable.sprint_speed
            } else {
                controllable.walk_speed
            };
        forces.apply_force(dir * 10.0);
    }
}
