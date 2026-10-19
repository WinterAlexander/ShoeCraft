use crate::command::CommandSupport;
use crate::cutting::Sole;
use crate::title_screen::GameState;
use bevy::app::{App, Plugin, Update};
use bevy::asset::{AssetServer, Assets, Handle};
use bevy::mesh::{Mesh, Mesh3d};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{
    Commands, Component, IntoScheduleConfigs, OnEnter, Query, Res, ResMut, Transform, default,
    in_state,
};

pub struct AssemblyPlugin;

#[derive(Component)]
pub struct TopPiece;

impl Plugin for AssemblyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_covers.run_if(in_state(GameState::Assembly)));
    }
}

fn spawn_covers(
    mut commands: Commands,
    soles: Query<(&Sole, &Transform, &CommandSupport)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
    top_pieces: Query<&TopPiece>,
) {
    for _ in top_pieces {
        return;
    }

    for (sole, transform, cmd) in soles {
        if cmd.command_queue.len() > 0 {
            continue;
        }

        if let Some(mesh) = crate::cutting::create_extrusion_mesh(&sole.polygon, 0.1) {
            let shoe_tex = assets.load("shoe_texture/old_leather.png");
            let material_handle: Handle<StandardMaterial> = materials.add(StandardMaterial {
                base_color_texture: Some(shoe_tex.clone()),
                ..default()
            });

            let mut transform = transform.clone();
            transform.translation.y += 0.05;

            let mesh = meshes.add(mesh);
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material_handle.clone()),
                transform,
                TopPiece,
            ));
        }
    }
}
