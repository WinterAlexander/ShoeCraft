use crate::title_screen::GameState;
use avian3d::parry::glamx::{Quat, Vec3};
use avian3d::prelude::{SpatialQuery, SpatialQueryFilter};
use bevy::app::{App, Plugin, Update};
use bevy::camera::Camera;
use bevy::input::ButtonInput;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::{
    Children, Component, Entity, GlobalTransform, IntoScheduleConfigs, MouseButton, Query, Res,
    Single, Touches, Transform, Window, With, in_state,
};
use bevy::window::PrimaryWindow;
use std::f32::consts::PI;

pub struct KnifePlugin;

impl Plugin for KnifePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (pick_up_knife, move_knife, rotate_knife).run_if(in_state(GameState::Cutting)),
        );
    }
}

#[derive(Component, Default)]
pub struct Knife {
    in_hand: bool,
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
}

fn rotate_knife(
    mut knife: Single<(&mut Transform, &Knife)>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
) {
    if !knife.1.in_hand {
        return;
    }
    knife.0.rotate_local_y(mouse_scroll.delta.y * PI / 8.0);
}

fn use_knife() {}
