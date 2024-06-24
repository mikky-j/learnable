use bevy::prelude::*;
use bevy_simple_text_input::TextInputBundle;

use crate::{
    focus::InteractionFocusBundle,
    text_input::CustomTextInputBundle,
    ui_box::{Hole, HoleContainerBundle, UIBox},
};

#[derive(Debug, Component)]
pub struct FunctionComponent;

#[derive(Debug, Component)]
pub struct FunctionNameComponent;

#[derive(Debug, Component)]
pub struct FunctionReturnComponent;

#[derive(Debug, Component)]
pub struct FunctionArgContainerComponent;

#[derive(Debug, Component)]
pub struct FunctionArgComponent;

#[derive(Debug, Component)]
pub struct FunctionBodyComponent;

#[derive(Bundle)]
pub struct FunctionBundle {
    marker: (FunctionComponent, UIBox),
    node: NodeBundle,
    focus: InteractionFocusBundle,
}

impl FunctionBundle {
    fn new() -> Self {
        Self {
            node: NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    min_width: Val::Px(50.),
                    min_height: Val::Px(100.),
                    ..default()
                },

                ..default()
            },
            marker: (FunctionComponent, UIBox),
            focus: InteractionFocusBundle::default(),
        }
    }
}

#[derive(Bundle)]
pub struct FunctionNameBundle {
    text_input_bundle: CustomTextInputBundle,
    marker: FunctionNameComponent,
}

impl FunctionNameBundle {
    fn new(function_entity: Entity) -> Self {
        Self {
            text_input_bundle: CustomTextInputBundle::new(
                TextInputBundle::default(),
                function_entity,
            ),
            marker: FunctionNameComponent,
        }
    }
}

// pub struct FunctionArgBundle {
//     hole_bundle: HoleContainerBundle,
//     marker:
// }
//
// impl FunctionArgBundle {
//     fn new()
// }

pub struct FunctionPlugin;

impl FunctionPlugin {}

impl Plugin for FunctionPlugin {
    fn build(&self, app: &mut App) {
        todo!()
    }
}
