use crate::title_screen::GameState;
use bevy::app::{App, Plugin, Update};
use bevy::input::ButtonInput;
use bevy::prelude::{KeyCode, MonitorSelection, OnEnter, OnExit, Query, Res, Single, Window, With};
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow, WindowMode};

pub struct WindowManagementPlugin;

impl Plugin for WindowManagementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, toggle_fullscreen);
        app.add_systems(OnEnter(GameState::Game), capture_cursor);
        app.add_systems(OnExit(GameState::Game), release_cursor);
    }
}

fn capture_cursor(mut cursor_options: Single<&mut CursorOptions>) {
    cursor_options.visible = false;
    cursor_options.grab_mode = CursorGrabMode::Confined;
}

fn release_cursor(mut cursor_options: Single<&mut CursorOptions>) {
    cursor_options.visible = true;
    cursor_options.grab_mode = CursorGrabMode::None;
}

fn toggle_fullscreen(
    input: Res<ButtonInput<KeyCode>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    if input.just_pressed(KeyCode::F11) {
        for mut window in window_query.iter_mut() {
            window.mode = match window.mode {
                WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                _ => WindowMode::Windowed,
            }
        }
    }
}
