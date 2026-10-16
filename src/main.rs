mod character;
pub mod debug;
mod debug_camera_controller;
mod example2d;
mod object_spawner;
mod particle;
pub mod pause;
mod scene_loader;
pub mod thirdperson_camera_controller;
mod title_screen;
mod ui;
mod window_management;

use crate::character::CharacterPlugin;
use crate::debug::DebugPlugin;
use crate::debug_camera_controller::DebugCameraControllerPlugin;
use crate::example2d::example2d::Example2DPlugin;
use crate::example2d::topdowncameracontroller::TopDownCameraControllerPlugin;
use crate::object_spawner::ObjectSpawnerPlugin;
use crate::pause::PausePlugin;
use crate::scene_loader::SceneLoaderPlugin;
use crate::thirdperson_camera_controller::ThirdPersonCameraPlugin;
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
use std::env;

fn make_2d(app: &mut App) {
    app.add_plugins(DefaultPlugins)
        .add_plugins(Example2DPlugin)
        .add_plugins(TopDownCameraControllerPlugin);
}

fn make_3d(app: &mut App) {
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
    app.add_plugins(SceneLoaderPlugin);
    app.add_plugins(TitleScreenPlugin);
    app.add_plugins((ThirdPersonCameraPlugin, DebugCameraControllerPlugin));
    app.add_plugins((ObjectSpawnerPlugin, CharacterPlugin));
    app.add_plugins((DebugPlugin, WindowManagementPlugin, PausePlugin));
    app.insert_state(GameState::TitleScreen);
}

fn main() {
    let mut app = App::new();

    let args: Vec<String> = env::args().collect();

    if args.contains(&String::from("2d")) {
        make_2d(&mut app);
    } else {
        make_3d(&mut app);
    }

    app.run();
}
