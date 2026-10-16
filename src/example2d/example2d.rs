use bevy::app::{App, Plugin, Startup};
use bevy::asset::{AssetServer, Assets};
use bevy::camera::{Camera, Camera2d, ClearColor, OrthographicProjection, Projection};
use bevy::color::Color;
use bevy::mesh::{Mesh, Mesh2d};
use bevy::prelude::{
    ColorMaterial, Commands, FontSize, MeshMaterial2d, Rectangle, Res, ResMut, Transform, default,
};
use bevy::sprite::Text2d;
use bevy::text::{TextColor, TextFont};

pub struct Example2DPlugin;

impl Plugin for Example2DPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_camera, spawn_object, spawn_text))
            .insert_resource(ClearColor(Color::srgb(1.0, 1.0, 1.0)));
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera { ..default() },
        Projection::Orthographic(OrthographicProjection::default_2d()),
        Camera2d,
        Transform::from_xyz(100.0, 200.0, 0.0),
    ));
}

fn spawn_object(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(Rectangle::new(10.0, 10.0));
    let material = materials.add(Color::srgb(1.0, 0.0, 0.0));
    commands.spawn((
        Transform::from_xyz(50.0, 50.0, 0.0),
        Mesh2d(mesh),
        MeshMaterial2d(material),
    ));
}

fn spawn_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("composite.ttf");
    let text_font = TextFont {
        font: font.clone().into(),
        font_size: FontSize::Px(50.0),
        ..default()
    };

    commands.spawn((
        Transform::from_xyz(-50.0, 50.0, 0.0),
        Text2d::new("人"),
        TextColor(Color::BLACK),
        text_font.clone(),
    ));
}
