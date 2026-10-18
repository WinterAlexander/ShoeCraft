use crate::camera_command::ControllableCamera;
use crate::drawing::Pencil;
use avian3d::PhysicsPlugins;
use avian3d::prelude::{ColliderConstructor, ColliderConstructorHierarchy};
use bevy::anti_alias::taa::TemporalAntiAliasing;
use bevy::camera::Hdr;
use bevy::core_pipeline::prepass::DepthPrepass;
use bevy::light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap};
use bevy::pbr::{ScreenSpaceAmbientOcclusion, ScreenSpaceAmbientOcclusionQualityLevel};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::camera::TemporalJitter;
use bevy::world_serialization::WorldInstanceReady;

pub struct ShopPlugin;

#[derive(Component)]
pub struct LeatherSheet;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_shop);
        app.add_plugins(PhysicsPlugins::default())
            .insert_resource(ClearColor(Color::srgb(0.3, 0.6, 0.9)))
            .insert_resource(DirectionalLightShadowMap { size: 4096 });
    }
}

fn setup_shop(mut commands: Commands, assets: ResMut<AssetServer>) {
    commands.spawn((
        AudioPlayer::new(assets.load("test_song.ogg")),
        PlaybackSettings::LOOP,
    ));

    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 1000.0,
        affects_lightmapped_meshes: true,
    });
    commands.spawn((
        Transform::from_translation(Vec3::Y).looking_at(Vec3::ZERO, Vec3::Y),
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 5000.0,
            shadow_maps_enabled: true,
            contact_shadows_enabled: true,
            affects_lightmapped_mesh_diffuse: true,
            shadow_depth_bias: 1.0,
            shadow_normal_bias: 1.0,
        },
        CascadeShadowConfigBuilder {
            maximum_distance: 500.0,
            ..default()
        }
        .build(),
    ));

    commands.spawn((
        //NormalPrepass,
        //ShowPrepass::Normals,
        DepthPrepass,
        Camera3d::default(),
        Camera {
            // renders after / on top of the main camera
            order: 1,
            // don't clear the color while rendering this camera
            clear_color: ClearColorConfig::Default,
            ..default()
        },
        ControllableCamera::default(),
        Hdr,
        Projection::Perspective(PerspectiveProjection {
            fov: 55.0f32.to_radians(),
            ..default()
        }),
        Transform::from_xyz(-1.5, 1.0, -0.75).looking_at(Vec3::new(-1.3, 0.0, -0.75), Vec3::Y),
        DistanceFog {
            color: Color::srgb(5.0, 0.25, 0.25),
            falloff: FogFalloff::Linear {
                start: 500.0,
                end: 600.0,
            },
            ..default()
        },
        Msaa::Off,
        ScreenSpaceAmbientOcclusion {
            quality_level: ScreenSpaceAmbientOcclusionQualityLevel::Ultra,
            ..default()
        },
        TemporalAntiAliasing::default(),
        TemporalJitter::default(),
        Bloom::default(),
    ));

    commands
        .spawn((
            WorldAssetRoot(assets.load("shop.glb#Scene0")),
            ColliderConstructorHierarchy::new(ColliderConstructor::ConvexDecompositionFromMesh),
        ))
        .observe(prepare_scene_children);
}

fn prepare_scene_children(
    ready: On<WorldInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    names: Query<&Name>,
) {
    for child in children.iter_descendants(ready.entity) {
        if let Ok(name) = names.get(child) {
            match name.as_str() {
                "Pencil" => {
                    commands.entity(child).insert(Pencil::default());
                }
                "Leather Sheet" => {
                    commands.entity(child).insert(LeatherSheet);
                }
                _ => {
                    if name.contains("Pencil") {
                        // Pencil's materials
                        continue;
                    }
                    //commands.entity(child).insert(Decalable::default());
                }
            }
        }
    }
}
