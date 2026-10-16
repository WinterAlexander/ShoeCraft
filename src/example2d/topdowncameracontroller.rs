use bevy::app::{App, Plugin, Update};
use bevy::camera::Camera;
use bevy::input::ButtonInput;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::math::{Vec2, Vec3Swizzles};
use bevy::prelude::{
    GlobalTransform, MouseButton, Res, ResMut, Resource, Single, Transform, Window, With,
};
use bevy::window::PrimaryWindow;

pub struct TopDownCameraControllerPlugin;

#[derive(Resource, Default)]
pub struct MouseDragState {
    pub dragged: bool,
    pub start_drag_position: Vec2,
    pub start_cam_position: Vec2,
}

const DRAG_BUTTON: MouseButton = MouseButton::Right;
const SCROLL_SENSITIVITY: f32 = 0.01;

impl Plugin for TopDownCameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_camera, update_drag_state));
        app.insert_resource(MouseDragState::default());
    }
}

fn update_drag_state(
    camera_transform: Single<&Transform, With<Camera>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut drag_state: ResMut<MouseDragState>,
) {
    let pressed = mouse_buttons.pressed(DRAG_BUTTON);
    if pressed == drag_state.dragged {
        return;
    }

    drag_state.dragged = pressed;
    if pressed {
        drag_state.start_drag_position = window.cursor_position().unwrap();
        drag_state.start_cam_position = camera_transform.translation.xy();
    }
}

fn update_camera(
    mut camera_transform_query: Single<(&Camera, &GlobalTransform, &mut Transform)>,
    window: Single<&Window, With<PrimaryWindow>>,
    drag_state: Res<MouseDragState>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
) {
    let (camera, gt, transform) = &mut *camera_transform_query;
    transform.scale -= mouse_scroll.delta.y * SCROLL_SENSITIVITY;

    if !drag_state.dragged {
        return;
    }

    let Some(current_mouse) = window.cursor_position() else {
        return;
    };

    let start_drag_world = camera
        .viewport_to_world_2d(gt, drag_state.start_drag_position)
        .unwrap();
    let current_mouse_world = camera.viewport_to_world_2d(gt, current_mouse).unwrap();

    transform.translation =
        (drag_state.start_cam_position + start_drag_world - current_mouse_world).extend(0.0);
}
