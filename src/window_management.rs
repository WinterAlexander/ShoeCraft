use bevy::app::{App, Plugin, Update};
use bevy::input::ButtonInput;
use bevy::prelude::{KeyCode, MonitorSelection, Query, Res, Single, Window, With};
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow, WindowMode};

pub struct WindowManagementPlugin;

impl Plugin for WindowManagementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, toggle_fullscreen);
    }
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
