// mod ast;
// mod r#box;
mod connectors;
// mod line;
mod collision;
mod focus;
mod ui_box;
mod ui_line;
mod utils;

use bevy::{app::PluginGroupBuilder, prelude::*, window::PresentMode};
use ui_line::UiLinePlugin;

use crate::{focus::FocusPlugin, ui_box::UIBoxPlugin};
use connectors::ConnectorPlugin;
// use line::LinePlugin;
// use r#box::BoxPlugin;

// use crate::{ast::ASTPlugin, ui_box::UIBoxPlugin};
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

pub fn box_pos_collision(subject: Vec2, (target_pos, target_size): (Vec2, Vec2)) -> bool {
    subject.x >= target_pos.x
        && subject.x <= (target_pos.x + target_size.x)
        && subject.y >= target_pos.y
        && subject.y <= (target_pos.y + target_size.y)
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Label(pub String);

pub struct GamePlugin;

#[derive(Debug, Event, Clone, Copy)]
pub struct DeleteEvent(pub Entity);

impl GamePlugin {
    // Setup functions
    fn spawn_camera(mut commands: Commands) {
        commands.spawn_empty().insert(Camera2dBundle::default());
    }

    fn handle_delete_block(mut reader: EventReader<DeleteEvent>, mut commands: Commands) {
        for &DeleteEvent(block) in reader.read() {
            let Some(commands) = commands.get_entity(block) else {
                error!("Cannot get the commands");
                continue;
            };
            commands.despawn_recursive();
        }
    }
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::spawn_camera)
            .add_systems(Last, Self::handle_delete_block)
            .add_event::<DeleteEvent>()
            // .add_plugins(BoxPlugin)
            .add_plugins(FocusPlugin)
            .add_plugins(UiLinePlugin)
            // .add_plugins(LinePlugin)
            // .add_plugins(ASTPlugin)
            .add_plugins(UIBoxPlugin)
            .add_plugins(ConnectorPlugin);
    }
}
