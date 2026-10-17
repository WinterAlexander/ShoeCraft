mod character;
pub mod debug;
mod debug_camera_controller;
pub mod decal;
mod drawing;
mod object_spawner;
mod particle;
pub mod pause;
mod scene_loader;
pub mod thirdperson_camera_controller;
mod title_screen;
mod ui;
mod window_management;

use crate::debug::DebugPlugin;
use crate::debug_camera_controller::DebugCameraControllerPlugin;
use crate::decal::DecalPlugin;
use crate::drawing::DrawingPlugin;
use crate::object_spawner::ObjectSpawnerPlugin;
use crate::pause::PausePlugin;
use crate::scene_loader::SceneLoaderPlugin;
use crate::title_screen::{GameState, TitleScreenPlugin};
use crate::ui::interactive_button::InteractiveButtonPlugin;
use crate::window_management::WindowManagementPlugin;
use bevy::DefaultPlugins;
use bevy::app::{App, PluginGroup};
use bevy::asset::AssetMetaCheck;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{AddressMode, FilterMode};
use bevy::window::{CursorGrabMode, CursorOptions};
use bevy_show_prepass::ShowPrepassPlugin;

fn main() {
    let mut app = App::new();

    let default_sampler = ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::from(AddressMode::Repeat),
        address_mode_v: ImageAddressMode::from(AddressMode::Repeat),
        address_mode_w: ImageAddressMode::from(AddressMode::Repeat),
        mag_filter: ImageFilterMode::from(FilterMode::Linear),
        min_filter: ImageFilterMode::from(FilterMode::Linear),
        mipmap_filter: ImageFilterMode::from(FilterMode::Linear),
        ..default()
    };

    let is_web = cfg!(target_arch = "wasm32");
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    fit_canvas_to_parent: true,
                    title: "Bevy Quickstart Game".to_string(),
                    ..default()
                }),
                primary_cursor_options: Some(CursorOptions {
                    visible: true,
                    // note this bug: https://github.com/bevyengine/bevy/issues/16237
                    grab_mode: CursorGrabMode::None,
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin { default_sampler })
            .set(AssetPlugin {
                meta_check: if is_web {
                    AssetMetaCheck::Never
                } else {
                    Default::default()
                },
                ..default()
            }),
    );
    app.add_plugins(InteractiveButtonPlugin);
    app.add_plugins((SceneLoaderPlugin, TitleScreenPlugin, DecalPlugin));
    app.add_plugins(DebugCameraControllerPlugin);
    app.add_plugins(DrawingPlugin);
    app.add_plugins((
        DebugPlugin,
        WindowManagementPlugin,
        PausePlugin,
        ShowPrepassPlugin,
    ));
    app.insert_state(GameState::TitleScreen);

    app.run();
}
