use bevy::app::{App, Plugin};
use bevy::prelude::Resource;

pub struct HintPlugin;

pub enum HintType {
    Cut,
    Draw,
    Next,
    PickupPencil,
    PickupKnife,
    RotateKnife,
}

#[derive(Resource)]
pub struct DisplayedHints {
    hints: Vec<HintType>,
}

impl Plugin for HintPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DisplayedHints { hints: Vec::new() });
    }
}
