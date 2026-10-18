use crate::title_screen::GameState;
use avian3d::parry::glamx::{Quat, Vec3};
use avian3d::prelude::{SpatialQuery, SpatialQueryFilter};
use bevy::app::{App, Plugin, Update};
use bevy::camera::Camera;
use bevy::input::ButtonInput;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::math::{Dir3, Vec2, Vec3Swizzles};
use bevy::prelude::{
    Children, Component, Entity, GlobalTransform, InfinitePlane3d, IntoScheduleConfigs,
    MouseButton, Query, Res, Single, Touches, Transform, Window, With, in_state,
};
use bevy::window::PrimaryWindow;
use std::f32::consts::PI;

const TABLE_HEIGHT: f32 = 0.617;
const MAX_CUT_DISTANCE: f32 = 0.135;

pub struct KnifePlugin;

impl Plugin for KnifePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (pick_up_knife, move_knife, rotate_knife, use_knife)
                .run_if(in_state(GameState::Cutting)),
        );
    }
}

#[derive(Component)]
pub struct Knife {
    in_hand: bool,
    pressed: bool,
    first_click: bool,
    angle: f32,
    current_cut: Option<Entity>,
}

#[derive(Component)]
pub struct Cut {
    extent_positive: f32,
    extent_negative: f32,
}

impl Default for Knife {
    fn default() -> Self {
        Knife {
            in_hand: false,
            pressed: false,
            first_click: true,
            angle: 0.0,
            current_cut: None,
        }
    }
}

impl Default for Cut {
    fn default() -> Self {
        Cut {
            extent_positive: 0.0,
            extent_negative: 0.0,
        }
    }
}

fn pick_up_knife(
    mut knife: Single<(Entity, &mut Transform, &mut Knife)>,
    buttons: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    window_query: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    spatial_query: SpatialQuery,
    children: Query<&Children>,
) {
    if knife.2.in_hand {
        return;
    }
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let position = if let Some(touch) = touches.iter().next() {
        touch.position()
    } else if let Some(cursor_pos) = window_query.cursor_position() {
        cursor_pos
    } else {
        return;
    };
    let Ok(ray) = camera_query.0.viewport_to_world(&camera_query.1, position) else {
        return;
    };

    let Some(_hit) = spatial_query.cast_ray_predicate(
        ray.origin,
        ray.direction,
        50.0,
        false,
        &SpatialQueryFilter::default(),
        &|entity| {
            // If entity is not the knife, skip
            entity == knife.0 || !children.iter_descendants(knife.0).any(|e| e == entity)
        },
    ) else {
        return;
    };

    knife.2.in_hand = true;
    knife.1.rotation = Quat::from_axis_angle(Vec3::X, PI);
}

fn move_knife(
    mut knife: Single<(Entity, &mut Transform, &mut Knife)>,
    touches: Res<Touches>,
    window_query: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    spatial_query: SpatialQuery,
    children: Query<&Children>,
) {
    if !knife.2.in_hand {
        return;
    }

    let position = if let Some(touch) = touches.iter().next() {
        touch.position()
    } else if let Some(cursor_pos) = window_query.cursor_position() {
        cursor_pos
    } else {
        return;
    };
    let Ok(ray) = camera_query.0.viewport_to_world(&camera_query.1, position) else {
        return;
    };

    if knife.2.pressed {
        let Some(t) = ray.intersect_plane(
            Vec3::new(0.0, TABLE_HEIGHT, 0.0),
            InfinitePlane3d { normal: Dir3::Y },
        ) else {
            return;
        };

        let hit_point = ray.origin + ray.direction * t;

        knife.1.translation.y = TABLE_HEIGHT;
        let offset = knife.1.translation.xz() - hit_point.xz();
        let angle = knife.2.angle;
        let direction_of_interest = Vec2::from_angle(angle);
        let offset = -offset.dot(direction_of_interest) * 2.0;

        knife.1.rotation = Quat::default();
        knife.1.rotate_local_x(PI);
        knife.1.rotate_local_y(angle);
        knife.1.rotate_local_z(offset.clamp(-PI / 5.0, PI / 5.0));
        return;
    } else {
        let Some(hit) = spatial_query.cast_ray_predicate(
            ray.origin,
            ray.direction,
            50.0,
            false,
            &SpatialQueryFilter::default(),
            &|entity| {
                // If entity is the knife itself or a descendant of the knife, skip
                entity != knife.0 && !children.iter_descendants(knife.0).any(|e| e == entity)
            },
        ) else {
            return;
        };

        let hit_point = ray.origin + ray.direction * hit.distance;

        knife.1.translation = hit_point;
        knife.1.rotation = Quat::default();
        knife.1.rotate_local_x(PI);
        let angle = knife.2.angle;
        knife.1.rotate_local_y(angle);
    }
}

fn rotate_knife(mut knife: Single<&mut Knife>, mouse_scroll: Res<AccumulatedMouseScroll>) {
    if !knife.in_hand || knife.pressed {
        return;
    }
    knife.angle += mouse_scroll.delta.y * PI / 8.0;
}

fn use_knife(
    mut knife: Single<(Entity, &mut Transform, &mut Knife)>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if !knife.2.in_hand {
        return;
    }

    if !buttons.pressed(MouseButton::Left) {
        knife.2.first_click = false;
    }

    if knife.2.first_click {
        return;
    }

    knife.2.pressed = buttons.pressed(MouseButton::Left);
}

fn render_cuts() {}
