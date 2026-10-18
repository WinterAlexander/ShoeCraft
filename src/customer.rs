use avian3d::parry::glamx::Quat;
use avian3d::prelude::{ColliderConstructor, ColliderConstructorHierarchy};
use bevy::animation::AnimationPlayer;
use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::{AssetServer, Assets};
use bevy::gltf::{GltfAssetLabel, GltfMaterialName};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{
    AnimationGraph, AnimationGraphHandle, AnimationNodeIndex, Children, Commands, Component,
    Entity, On, Query, Res, ResMut, Time, Transform, Visibility, WorldAssetRoot,
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
    pub angry: bool,
    pub render_angry: bool,
    pub angry_timer: f32,
    pub not_angry_timer: f32,
    pub anger_cancel_click: bool,
}

pub struct CustomerPlugin;

impl Plugin for CustomerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_customer);
        app.add_systems(
            Update,
            (animation_transition, update_customer_angry_material),
        );
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
                                angry: false,
                                render_angry: false,
                                angry_timer: 0.0,
                                not_angry_timer: 0.0,
                                anger_cancel_click: false,
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

fn update_customer_angry_material(
    mut commands: Commands,
    customers: Query<(Entity, &mut AnimatedCustomer)>,
    children: Query<&Children>,
    materials: Query<(&MeshMaterial3d<StandardMaterial>, &GltfMaterialName)>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
    time: Res<Time>,
) {
    for mut customer in customers {
        if customer.1.angry {
            if customer.1.angry_timer > 0.0 {
                customer.1.angry_timer -= time.delta_secs();
                if customer.1.angry_timer <= 0.0 {
                    customer.1.angry = false;
                }
            }
        } else {
            if customer.1.not_angry_timer > 0.0 {
                customer.1.not_angry_timer -= time.delta_secs();
                if customer.1.not_angry_timer <= 0.0 {
                    customer.1.angry = true;
                }
            }
        }

        if customer.1.angry == customer.1.render_angry {
            continue;
        }
        let regular_material_tex = assets.load("cat.glb#Texture1");
        let angry_material_tex = assets.load("cat.glb#Texture0");

        for descendant in children.iter_descendants(customer.0) {
            if let Ok((id, material_name)) = materials.get(descendant) {
                let Some(material) = material_assets.get(id.id()) else {
                    continue;
                };

                if customer.1.angry {
                    let mut new_material = material.clone();
                    new_material.base_color_texture = Some(angry_material_tex.clone());
                    commands
                        .entity(descendant)
                        .insert(MeshMaterial3d(material_assets.add(new_material)));
                } else {
                    let mut new_material = material.clone();
                    new_material.base_color_texture = Some(regular_material_tex.clone());
                    commands
                        .entity(descendant)
                        .insert(MeshMaterial3d(material_assets.add(new_material)));
                }
            }
        }

        customer.1.render_angry = customer.1.angry;
    }
}
