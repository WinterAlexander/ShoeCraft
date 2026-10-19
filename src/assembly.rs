use crate::command::CommandSupport;
use crate::cutting::Sole;
use crate::title_screen::GameState;
use bevy::app::{App, Plugin, Update};
use bevy::asset::{AssetServer, Assets, Handle};
use bevy::math::{Vec2, Vec3};
use bevy::mesh::{Mesh, Mesh3d, VertexAttributeValues};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{
    Commands, Component, IntoScheduleConfigs, OnEnter, Query, Res, ResMut, Transform, default,
    in_state,
};

/// Scale applied to the bottom end of the prism (its -Z end).
const BOTTOM_SCALE: f32 = 0.9;
/// Scale applied to the top end of the prism (its +Z end).
const TOP_SCALE: f32 = 0.5;
/// How far the top end of the prism leans toward the world's -Z direction.
const TOP_Z_OFFSET: f32 = 0.1;

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
        if cmd.command_queue.len() > 0 || sole.left {
            continue;
        }

        if let Some(mut mesh) = crate::cutting::create_extrusion_mesh(&sole.polygon, 0.1) {
            shape_shoe(&mut mesh, &transform);
            let shoe_tex = assets.load("shoe_texture/fur.png");
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

/// Tapers the extruded prism into a shoe: the two ends along Z are scaled about
/// the shape's centre by their own factor and the top end leans toward the
/// world's -Z direction by `TOP_Z_OFFSET`.
///
/// The prism ends sit at ±depth/2, so the sign of a vertex's Z tells which end
/// it belongs to. Scaling both the caps and the end rings of the side walls
/// keeps the prism closed. The entity's rotation converts the world -Z lean
/// into this mesh's local axes, because the sole rotation chain orients the
/// extrusion axis (local Z) along world X rather than straight out of the table.
fn shape_shoe(mesh: &mut Mesh, transform: &Transform) {
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    else {
        return;
    };

    let offset = transform.rotation.inverse() * (Vec3::NEG_Z * TOP_Z_OFFSET);

    let mut center = Vec2::ZERO;
    for &[x, y, _] in positions.iter() {
        center += Vec2::new(x, y);
    }
    center /= positions.len() as f32;

    for [x, y, z] in positions.iter_mut() {
        let top = *z > 0.0;
        let scale = if top { TOP_SCALE } else { BOTTOM_SCALE };
        *x = center.x + (*x - center.x) * scale;
        *y = center.y + (*y - center.y) * scale;
        if top {
            *x += offset.x;
            *y += offset.y;
            *z += offset.z;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    /// The world -Z lean converts to the mesh's local -Y for the sole rotation
    /// chain (rotate_z then rotate_y by 90 degrees).
    fn sole_rotation() -> Transform {
        let mut transform = Transform::IDENTITY;
        transform.rotate_z(PI / 2.0);
        transform.rotate_y(PI / 2.0);
        transform
    }

    #[test]
    fn shape_shoe_scales_ends_and_leans_top() {
        let square = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 1.0),
        ];
        let transform = sole_rotation();
        let mut mesh = crate::cutting::create_extrusion_mesh(&square, 0.1).expect("extrusion mesh");

        let Some(VertexAttributeValues::Float32x3(raw)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("mesh has no positions");
        };
        let raw = raw.clone();

        shape_shoe(&mut mesh, &transform);

        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("mesh has no positions");
        };

        // World -Z is the mesh's local -Y for this rotation chain.
        let offset = transform.rotation.inverse() * (Vec3::NEG_Z * TOP_Z_OFFSET);
        assert!(
            (offset - Vec3::new(0.0, -TOP_Z_OFFSET, 0.0)).length() < 1e-5,
            "offset = {offset:?}"
        );

        let center = Vec2::new(0.5, 0.5);
        for (&[x0, y0, z0], &shaped) in raw.iter().zip(positions.iter()) {
            let top = z0 > 0.0;
            let scale = if top { TOP_SCALE } else { BOTTOM_SCALE };
            let mut expected = [
                center.x + (x0 - center.x) * scale,
                center.y + (y0 - center.y) * scale,
                z0,
            ];
            if top {
                expected[0] += offset.x;
                expected[1] += offset.y;
                expected[2] += offset.z;
            }
            for i in 0..3 {
                assert!(
                    (shaped[i] - expected[i]).abs() < 1e-4,
                    "vertex {x0},{y0},{z0} -> {shaped:?}, expected {expected:?}"
                );
            }
        }
    }
}
