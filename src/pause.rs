use crate::title_screen::GameState;
use avian3d::prelude::{Physics, PhysicsTime};
use bevy::app::{App, Plugin, Update};
use bevy::input::ButtonInput;
use bevy::prelude::{DetectChanges, KeyCode, NextState, Res, ResMut, State, Time};

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (toggle_pause, pause_physics));
    }
}

fn toggle_pause(
    input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        match current_state.get() {
            GameState::Game => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::Game),
            _ => {}
        }
    }
}

fn pause_physics(state: Res<State<GameState>>, mut physics_time: ResMut<Time<Physics>>) {
    if state.is_changed() {
        match state.get() {
            GameState::Paused => physics_time.pause(),
            GameState::Game => physics_time.unpause(),
            _ => {}
        }
    }
}
