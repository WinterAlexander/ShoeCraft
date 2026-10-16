use avian3d::PhysicsPlugins;
use avian3d::prelude::{
    ColliderConstructor, ColliderConstructorHierarchy, PhysicsDebugPlugin, RigidBody,
};
use bevy::anti_alias::taa::{TemporalAntiAliasPlugin, TemporalAntiAliasing};
use bevy::camera::Hdr;
use bevy::core_pipeline::Skybox;
use bevy::image::CompressedImageFormats;
use bevy::light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap};
use bevy::pbr::{ScreenSpaceAmbientOcclusion, ScreenSpaceAmbientOcclusionQualityLevel};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::camera::TemporalJitter;
use bevy::render::render_resource::{TextureViewDescriptor, TextureViewDimension};

pub struct SceneLoaderPlugin;

pub const CUBEMAPS: &[(&str, CompressedImageFormats)] =
    &[("test_skybox.png", CompressedImageFormats::NONE)];

#[derive(Resource)]
pub struct Cubemap {
    pub(crate) is_loaded: bool,
    pub(crate) image_handle: Handle<Image>,
}

#[derive(Component)]
pub struct SceneElement;

impl Plugin for SceneLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_basic);

        app.add_systems(Update, asset_loaded);
        app.add_systems(Update, scene_switcher);
        app.add_plugins(PhysicsPlugins::default())
            .insert_resource(ClearColor(Color::srgb(0.3, 0.6, 0.9)))
            .insert_resource(DirectionalLightShadowMap { size: 4096 });
    }
}

fn scene_switcher(
    input: Res<ButtonInput<KeyCode>>,
    mut scene_elements: Query<(Entity, &SceneElement)>,
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
) {
    if !input.pressed(KeyCode::ControlLeft) && !input.pressed(KeyCode::ControlRight) {
        return;
    }

    if input.just_pressed(KeyCode::Numpad1) || input.just_pressed(KeyCode::Digit1) {
        for (entity, _) in scene_elements.iter_mut() {
            commands.entity(entity).despawn();
        }
        setup_basic(commands, asset_server);
        return;
    } else if input.just_pressed(KeyCode::Numpad2) || input.just_pressed(KeyCode::Digit2) {
        for (entity, _) in scene_elements.iter_mut() {
            commands.entity(entity).despawn();
        }
        setup_kirby(commands, asset_server);
        return;
    }
}

/// set up a simple 3D scene
fn setup_basic(mut commands: Commands, mut asset_server: ResMut<AssetServer>) {
    commands.spawn((
        SceneElement,
        AudioPlayer::new(asset_server.load("test_song.ogg")),
        PlaybackSettings::LOOP,
    ));

    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 1000.0,
        affects_lightmapped_meshes: true,
    });
    commands.spawn((
        SceneElement,
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
    let skybox_handle = asset_server.load(CUBEMAPS[0].0);

    commands.insert_resource(Cubemap {
        is_loaded: false,
        image_handle: skybox_handle.clone(),
    });

    commands.spawn((
        SceneElement,
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
        Transform::from_xyz(-0.5, 0.3, 4.5).with_rotation(Quat::from_axis_angle(Vec3::Y, 0.0)),
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

    commands.spawn((
        WorldAssetRoot(asset_server.load("test_scene.glb#Scene0")),
        RigidBody::Static,
        ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
    ));
}

fn add_scene_tag(commands: &mut EntityCommands) {
    commands.insert(SceneElement);
}

fn setup_kirby(mut commands: Commands, mut asset_server: ResMut<AssetServer>) {
    commands.spawn((
        SceneElement,
        AudioPlayer::new(asset_server.load("test_song.ogg")),
        PlaybackSettings::LOOP,
    ));

    commands.insert_resource(ClearColor(Color::srgb(0.3, 0.6, 0.9)));
    commands.insert_resource(DirectionalLightShadowMap { size: 4096 });

    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 1000.0,
        affects_lightmapped_meshes: true,
    });
    commands.spawn((
        SceneElement,
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
    let skybox_handle = asset_server.load(CUBEMAPS[0].0);

    commands.insert_resource(Cubemap {
        is_loaded: false,
        image_handle: skybox_handle.clone(),
    });

    commands.spawn((
        SceneElement,
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
        Transform::from_xyz(-0.5, 0.3, 4.5).with_rotation(Quat::from_axis_angle(Vec3::Y, 0.0)),
        Skybox {
            image: Some(skybox_handle.clone()),
            brightness: 1000.0,
            rotation: Default::default(),
        },
        DistanceFog {
            color: Color::srgb(0.25, 0.25, 0.25),
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
    ));

    commands.spawn((
        WorldAssetRoot(asset_server.load("test_scene2.glb#Scene0")),
        RigidBody::Static,
        ColliderConstructorHierarchy::new(ColliderConstructor::ConvexDecompositionFromMesh),
    ));
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
