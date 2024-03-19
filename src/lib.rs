mod r#box;
// mod ui_box;
mod connectors;
mod line;
mod utils;

use bevy::{app::PluginGroupBuilder, prelude::*, window::PresentMode};
use connectors::ConnectorPlugin;
use line::LinePlugin;
use r#box::BoxPlugin;
// use ui_box::UIBoxPlugin;

pub const WINDOW_HEIGHT: f32 = 600.;
pub const WINDOW_WIDTH: f32 = 600.;

pub const WHITE: Color = Color::rgb(255., 255., 255.);
pub const RED: Color = Color::rgb(255., 0., 0.);

pub fn get_default_plugins() -> PluginGroupBuilder {
    DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Learnable test".into(),
            resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
            present_mode: PresentMode::AutoVsync,
            visible: true,
            ..default()
        }),
        ..default()
    })
}

pub fn translate_vec_to_world(mut vector: Vec2, window_height: f32, window_width: f32) -> Vec2 {
    vector.x -= window_width / 2.;
    vector.y = window_height / 2. - vector.y;
    vector
}

pub struct GamePlugin;

#[derive(Event)]
pub struct ChangedActiveEvent(pub Entity);

impl GamePlugin {
    // Setup functions
    fn spawn_camera(mut commands: Commands) {
        commands.spawn_empty().insert(Camera2dBundle::default());
    }
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::spawn_camera)
            .add_plugins(BoxPlugin)
            .add_plugins(ConnectorPlugin)
            .add_plugins(LinePlugin);
    }
}
