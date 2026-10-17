use crate::decal::{Decal, Decalable, spray_decal};
use crate::drawing::Pencil;
use avian3d::PhysicsPlugins;
use avian3d::prelude::{ColliderConstructor, ColliderConstructorHierarchy, RigidBody};
use bevy::anti_alias::taa::TemporalAntiAliasing;
use bevy::camera::Hdr;
use bevy::core_pipeline::Skybox;
use bevy::image::CompressedImageFormats;
use bevy::light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap};
use bevy::pbr::{ScreenSpaceAmbientOcclusion, ScreenSpaceAmbientOcclusionQualityLevel};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::camera::TemporalJitter;
use bevy::render::render_resource::{TextureViewDescriptor, TextureViewDimension};
use bevy::world_serialization::WorldInstanceReady;

pub struct SceneLoaderPlugin;

pub const CUBEMAPS: &[(&str, CompressedImageFormats)] =
    &[("test_skybox.png", CompressedImageFormats::NONE)];

#[derive(Resource)]
pub struct Cubemap {
    pub(crate) is_loaded: bool,
    pub(crate) image_handle: Handle<Image>,
}

impl Plugin for SceneLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_shop);
        app.add_systems(Update, asset_loaded);
        app.add_plugins(PhysicsPlugins::default())
            .insert_resource(ClearColor(Color::srgb(0.3, 0.6, 0.9)))
            .insert_resource(DirectionalLightShadowMap { size: 4096 });
    }
}

fn setup_shop(mut commands: Commands, mut assets: ResMut<AssetServer>) {
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
    let skybox_handle = assets.load(CUBEMAPS[0].0);

    commands.insert_resource(Cubemap {
        is_loaded: false,
        image_handle: skybox_handle.clone(),
    });

    commands.spawn((
        //NormalPrepass,
        //ShowPrepass::Normals,
        Camera3d::default(),
        Camera {
            // renders after / on top of the main camera
            order: 1,
            // don't clear the color while rendering this camera
            clear_color: ClearColorConfig::Default,
            ..default()
        },
        Hdr,
        Projection::Perspective(PerspectiveProjection {
            fov: 55.0f32.to_radians(),
            ..default()
        }),
        Transform::from_xyz(-1.0, 1.0, -0.75).looking_to(Vec3::NEG_Y, Vec3::Y),
        Skybox {
            image: Some(skybox_handle.clone()),
            brightness: 1000.0,
            rotation: Default::default(),
        },
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
            if name.contains("Material") {
                continue;
            }
            match name.as_str() {
                "Pencil" => {
                    commands.entity(child).insert(Pencil);
                }
                _ => {
                    commands.entity(child).insert(Decalable::default());
                }
            }
        }
    }
}

fn asset_loaded(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut cubemap: ResMut<Cubemap>,
    mut skyboxes: Query<&mut Skybox>,
) {
    if !cubemap.is_loaded && asset_server.load_state(&cubemap.image_handle).is_loaded() {
        let mut image = images.get_mut(&cubemap.image_handle).unwrap();
        // NOTE: PNGs do not have any metadata that could indicate they contain a cubemap texture,
        // so they appear as one texture. The following code reconfigures the texture as necessary.
        if image.texture_descriptor.array_layer_count() == 1 {
            let layer_count = image.height() / image.width();
            let _ = image.reinterpret_stacked_2d_as_array(layer_count);
            image.texture_view_descriptor = Some(TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),
                ..default()
            });
        }

        for mut skybox in &mut skyboxes {
            skybox.image = Some(cubemap.image_handle.clone());
        }

        cubemap.is_loaded = true;
    }
}
