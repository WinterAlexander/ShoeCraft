use crate::scene_loader::SceneElement;
use avian3d::prelude::{ColliderConstructor, ColliderConstructorHierarchy, RigidBody};
use bevy::animation::AnimationPlayer;
use bevy::app::{App, Plugin, Update};
use bevy::asset::{AssetServer, Assets};
use bevy::gltf::GltfAssetLabel;
use bevy::input::ButtonInput;
use bevy::prelude::{
    AnimationGraph, AnimationGraphHandle, Children, Commands, KeyCode, On, Query, Res, ResMut,
    Transform, WorldAssetRoot,
};
use bevy::world_serialization::WorldInstanceReady;

pub struct ObjectSpawnerPlugin;

impl Plugin for ObjectSpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, object_spawner);
    }
}

fn object_spawner(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    asset_server: ResMut<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    if input.pressed(KeyCode::ControlLeft) || input.pressed(KeyCode::ControlRight) {
        return;
    }

    if input.just_pressed(KeyCode::Numpad1) || input.just_pressed(KeyCode::Digit1) {
        spawn_object("fruit_fly.glb#Scene0", commands, asset_server);
    } else if input.just_pressed(KeyCode::Numpad2) || input.just_pressed(KeyCode::Digit2) {
        spawn_object("pistol_shrimp.glb#Scene0", commands, asset_server);
    } else if input.just_pressed(KeyCode::Numpad3) || input.just_pressed(KeyCode::Digit3) {
        spawn_object("table_set.glb#Scene0", commands, asset_server);
    } else if input.just_pressed(KeyCode::Numpad4) || input.just_pressed(KeyCode::Digit4) {
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
}

fn spawn_object(name: &'static str, mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let _ = &mut commands.spawn((
        WorldAssetRoot(asset_server.load(name)),
        ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh),
        SceneElement,
        Transform::from_xyz(
            rand::random::<f32>() * 100.0 - 50.0,
            10.0,
            rand::random::<f32>() * 100.0 - 50.0,
        ),
        RigidBody::Dynamic,
    ));
}
