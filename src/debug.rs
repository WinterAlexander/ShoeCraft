use crate::debug_camera_controller::DebugCameraControllerState;
use crate::thirdperson_camera_controller::ThirdPersonCameraControllerState;
use avian3d::debug_render::PhysicsGizmos;
use avian3d::prelude::PhysicsDebugPlugin;
use bevy::app::{App, Plugin, Startup, Update};
use bevy::input::ButtonInput;
use bevy::prelude::{GizmoConfigStore, KeyCode, Res, ResMut};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsDebugPlugin);
        app.add_systems(Startup, |mut store: ResMut<GizmoConfigStore>| {
            store.config_mut::<PhysicsGizmos>().0.enabled = false;
        });
        app.add_systems(Update, toggle_debug_renderer);
    }
}

fn toggle_debug_renderer(
    input: Res<ButtonInput<KeyCode>>,
    mut store: ResMut<GizmoConfigStore>,
    mut debug_camera_state: ResMut<DebugCameraControllerState>,
    mut thirdp_camera_controller_state: ResMut<ThirdPersonCameraControllerState>,
) {
    if input.just_pressed(KeyCode::F12) {
        let (config, _) = store.config_mut::<PhysicsGizmos>();
        config.enabled = !config.enabled;
    } else if input.just_pressed(KeyCode::F5) {
        debug_camera_state.enabled = !debug_camera_state.enabled;
        thirdp_camera_controller_state.enabled = !debug_camera_state.enabled;
    }
}
