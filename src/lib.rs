#![allow(clippy::type_complexity)]

// TODO:
// - Fix focus to allow focusing on lines
// - Fix `ui_box` to allow the dragging outside of arguments

mod ast;
//
// mod r#box;
mod camera;
mod connectors;
// mod line;
// mod collision;
mod focus;
mod text_input;
mod ui_box;
mod ui_line;
mod utils;

use bevy::{app::PluginGroupBuilder, prelude::*, window::PresentMode};
use ui_line::UiLinePlugin;

use crate::{
    camera::CameraPlugin, focus::FocusPlugin, text_input::CustomTextInputPlugin,
    ui_box::UIBoxPlugin,
};
use connectors::ConnectorPlugin;
// use line::LinePlugin;
// use r#box::BoxPlugin;

use ast::ASTPlugin;
// use ui_box::UIBoxPlugin;

pub const WINDOW_HEIGHT: f32 = 600.;
pub const WINDOW_WIDTH: f32 = 600.;

pub const WHITE: Color = Color::rgb(255., 255., 255.);
pub const RED: Color = Color::rgb(255., 0., 0.);

// pub fn input_just_pressed_with_modifier<T>(
//     input: T,
// ) -> impl FnMut(Res<ButtonInput<T>>) -> bool + Clone
// where
//     T: Copy + Eq + std::hash::Hash + Send + Sync + 'static,
// {
//     move |inputs: Res<ButtonInput<T>>| inputs.get_just_pressed(KeyCode::ShiftLeft)
// }

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

#[derive(Component, Debug, Clone, PartialEq, Eq, Default)]
pub struct EntityLabel(pub String);

impl EntityLabel {
    pub fn new(data: impl Into<String>) -> Self {
        Self(data.into())
    }
}

#[derive(SystemSet, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
pub enum GameSets {
    Running,
    Despawn,
}

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
        app.configure_sets(Update, (GameSets::Despawn, GameSets::Running).chain())
            .add_systems(
                Update,
                apply_deferred
                    .before(GameSets::Running)
                    .after(GameSets::Despawn),
            )
            .add_systems(Update, Self::handle_delete_block.in_set(GameSets::Despawn))
            .add_event::<DeleteEvent>()
            // .add_plugins(BoxPlugin)
            .add_plugins(FocusPlugin)
            .add_plugins(UiLinePlugin)
            // .add_plugins(LinePlugin)
            .add_plugins(ASTPlugin)
            .add_plugins(UIBoxPlugin)
            .add_plugins(CustomTextInputPlugin)
            .add_plugins(CameraPlugin)
            .add_plugins(ConnectorPlugin);
    }
}
