use bevy::prelude::*;
use bevy_simple_text_input::{TextInputBundle, TextInputInactive};

use crate::{
    focus::{ActiveEntity, FocusBundle},
    GameSets,
};

#[derive(Debug, Component, Clone, Copy)]
pub struct TextInput {
    pub owner: Entity,
}

#[derive(Bundle)]
pub struct CustomTextInputBundle {
    text_input_bundle: TextInputBundle,
    text_input: TextInput,
    focusable: FocusBundle,
    node_bundle: NodeBundle,
}

impl CustomTextInputBundle {
    pub fn new(text_input_bundle: TextInputBundle, owner: Entity) -> Self {
        Self {
            node_bundle: NodeBundle {
                style: Style {
                    min_width: Val::Px(30.),
                    border: UiRect::bottom(Val::Px(2.)),
                    height: Val::Px(20.),
                    ..default()
                },
                border_color: Color::WHITE.into(),
                ..default()
            },
            text_input_bundle,
            text_input: TextInput { owner },
            focusable: FocusBundle::new(Color::RED, Color::GREEN, Color::WHITE),
        }
    }
}

pub struct CustomTextInputPlugin;

impl CustomTextInputPlugin {
    fn handle_text_focus(
        active_entity: Res<ActiveEntity>,
        mut query: Query<(Entity, &mut TextInputInactive), With<TextInput>>,
    ) {
        if let Some(active_entity) = active_entity.entity {
            for (text_entity, mut text_inactive) in &mut query {
                text_inactive.0 = text_entity != active_entity;
            }
        }
    }
}

impl Plugin for CustomTextInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::handle_text_focus.in_set(GameSets::Running));
    }
}
