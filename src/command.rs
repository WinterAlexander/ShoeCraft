use bevy::app::{App, Plugin, Update};
use bevy::math::Curve;
use bevy::prelude::{Component, EaseFunction, Query, Res, Time, Transform};
use std::collections::VecDeque;

pub struct CommandPlugin;

impl Plugin for CommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, execute_commands);
    }
}

#[derive(Component, Default)]
pub struct CommandSupport {
    pub command_queue: VecDeque<Command>,
}

impl CommandSupport {
    pub fn queue(&mut self, command: Command) {
        self.command_queue.push_back(command);
    }
}

pub enum Command {
    Transition {
        destination: Transform,
        duration: f32,
        ease_function: EaseFunction,
        start_time: Option<f64>,
        start_position: Option<Transform>,
    },
    Wait(f32),
}

impl Command {
    pub fn new_transition(
        destination: Transform,
        duration: f32,
        ease_function: EaseFunction,
    ) -> Command {
        Command::Transition {
            destination,
            duration,
            ease_function,
            start_time: None,
            start_position: None,
        }
    }
}

fn execute_commands(cameras: Query<(&mut Transform, &mut CommandSupport)>, time: Res<Time>) {
    for (mut trans, mut cam) in cameras {
        if cam.command_queue.len() == 0 {
            continue;
        }

        match &mut cam.command_queue[0] {
            Command::Transition {
                destination,
                duration,
                ease_function,
                start_time: Some(start_time),
                start_position: Some(start_position),
            } => {
                let t = ease_function.sample_clamped(
                    ((time.elapsed_secs_f64() - *start_time) as f32 / *duration).clamp(0.0, 1.0),
                );
                trans.translation = start_position.translation.lerp(destination.translation, t);
                trans.rotation = start_position.rotation.slerp(destination.rotation, t);
                if t >= 1.0 {
                    cam.command_queue.pop_front();
                }
            }
            Command::Transition {
                destination: _destination,
                duration: _duration,
                ease_function: _ease_function,
                start_time,
                start_position,
            } => {
                *start_time = Some(time.elapsed_secs_f64());
                *start_position = Some(trans.clone());
            }
            Command::Wait(duration) => {
                *duration -= time.delta_secs();
                if *duration <= 0.0 {
                    cam.command_queue.pop_front();
                }
            }
        }
    }
}
