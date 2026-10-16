use crate::title_screen::GameState;
use avian3d::parry::glamx::{EulerRot, Quat, Vec3};
use bevy::app::{App, Plugin, Update};
use bevy::camera::Camera3d;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::{
    Component, OnEnter, OnExit, Query, Res, ResMut, Resource, Single, Transform, With, Without,
};

pub struct ThirdPersonCameraPlugin;

impl Plugin for ThirdPersonCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_state, set_camera));
        app.add_systems(
            OnEnter(GameState::Game),
            |mut c: ResMut<ThirdPersonCameraControllerState>| c.enabled = true,
        );
        app.add_systems(
            OnExit(GameState::Game),
            |mut c: ResMut<ThirdPersonCameraControllerState>| c.enabled = false,
        );
        app.insert_resource(ThirdPersonCameraControllerState::default());
    }
}

#[derive(Component)]
pub struct CameraTarget;

#[derive(Resource)]
pub struct ThirdPersonCameraControllerState {
    pitch: f32,
    yaw: f32,
    distance: f32,

    pub enabled: bool,
}

impl Default for ThirdPersonCameraControllerState {
    fn default() -> Self {
        ThirdPersonCameraControllerState {
            pitch: -45.0f32,
            yaw: 0.0,
            distance: 7.0,
            enabled: false,
        }
    }
}

impl ThirdPersonCameraControllerState {
    fn to_quat(&self) -> Quat {
        Quat::from_euler(EulerRot::XYZ, 0.0, self.yaw, 0.0)
            * Quat::from_euler(EulerRot::XYZ, self.pitch, 0.0, 0.0)
    }
    fn to_vector(&self) -> Vec3 {
        let horizontal_scale = self.pitch.cos();
        Vec3::new(
            (self.yaw + std::f32::consts::PI).sin() * horizontal_scale,
            self.pitch.sin(),
            (self.yaw + std::f32::consts::PI).cos() * horizontal_scale,
        )
        .normalize()
    }
}

fn update_state(
    mut state: ResMut<ThirdPersonCameraControllerState>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
) {
    if !state.enabled {
        return;
    }

    let mouse_sensitivity = 0.002;

    state.yaw += -mouse_motion.delta.x * mouse_sensitivity;
    state.pitch = (state.pitch + (-mouse_motion.delta.y * mouse_sensitivity))
        .clamp((-90.0f32).to_radians(), 90.0f32.to_radians());

    state.distance -= scroll.delta.y;
}

fn set_camera(
    camera_target_query: Single<&Transform, (With<CameraTarget>, Without<Camera3d>)>,
    mut camera_transform_query: Query<&mut Transform, With<Camera3d>>,
    state: Res<ThirdPersonCameraControllerState>,
) {
    let mut camera_transform = camera_transform_query.single_mut().unwrap();

    if !state.enabled {
        return;
    }

    camera_transform.rotation = state.to_quat();
    camera_transform.translation =
        camera_target_query.translation - state.to_vector() * state.distance;
}
