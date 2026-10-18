use avian3d::parry::glamx::Quat;
use avian3d::prelude::{ColliderConstructor, ColliderConstructorHierarchy};
use bevy::animation::AnimationPlayer;
use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::{AssetServer, Assets};
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::{
    AnimationGraph, AnimationGraphHandle, AnimationNodeIndex, Children, Commands, Component, On,
    Query, ResMut, Transform, Visibility, WorldAssetRoot,
};
use bevy::world_serialization::WorldInstanceReady;
use std::f32::consts::PI;

#[derive(Component)]
pub struct CustomerCollider;

#[derive(Component)]
pub struct AnimatedCustomer {
    idle: AnimationNodeIndex,
    learn_forward: AnimationNodeIndex,
    start_lean_forward: AnimationNodeIndex,
}

pub struct CustomerPlugin;

impl Plugin for CustomerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_customer);
        app.add_systems(Update, (animation_transition));
    }
}

fn animation_transition(query: Query<(&mut AnimationPlayer, &AnimatedCustomer)>) {
    for (mut player, anims) in query {
        if player.all_finished() {
            player.play(anims.learn_forward).repeat();
        }
    }
}

fn spawn_customer(
    mut commands: Commands,
    assets: ResMut<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let mut graph = AnimationGraph::new();

    let idle = graph.add_clip(
        assets.load(GltfAssetLabel::Animation(0).from_asset("cat.glb")),
        1.0,
        graph.root,
    );

    let learn_forward = graph.add_clip(
        assets.load(GltfAssetLabel::Animation(1).from_asset("cat.glb")),
        1.0,
        graph.root,
    );

    let start_lean_forward = graph.add_clip(
        assets.load(GltfAssetLabel::Animation(2).from_asset("cat.glb")),
        1.0,
        graph.root,
    );

    let graph = graphs.add(graph);
    commands
        .spawn((
            WorldAssetRoot(assets.load("cat.glb#Scene0")),
            ColliderConstructorHierarchy::new(ColliderConstructor::ConvexDecompositionFromMesh),
            Transform::from_xyz(-0.65, 0.0, -0.75).with_rotation(Quat::from_rotation_y(-PI / 2.0)),
        ))
        .observe(
            move |ready: On<WorldInstanceReady>,
                  mut commands: Commands,
                  children: Query<&Children>,
                  mut players: Query<&mut AnimationPlayer>| {
                for child in children.iter_descendants(ready.entity) {
                    if let Ok(mut player) = players.get_mut(child) {
                        player.play(start_lean_forward);
                        commands.entity(child).insert((
                            AnimatedCustomer {
                                idle,
                                learn_forward,
                                start_lean_forward,
                            },
                            AnimationGraphHandle(graph.clone()),
                        ));
                    }
                }
            },
        );

    commands.spawn((
        WorldAssetRoot(assets.load("cat_leaning.glb#Scene0")),
        ColliderConstructorHierarchy::new(ColliderConstructor::ConvexDecompositionFromMesh),
        Transform::from_xyz(-0.65, 0.0, -0.75).with_rotation(Quat::from_rotation_y(-PI / 2.0)),
        Visibility::Hidden,
        CustomerCollider,
    ));
}
