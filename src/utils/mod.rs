// mod relative_position;
mod size;
// mod temp_line;

use std::{fmt::Debug, fs};

use serde::{Deserialize, Serialize};
pub use size::*;
// pub use temp_line::*;

use bevy::{
    math::bounding::Aabb2d, prelude::*, render::render_resource::encase::rts_array::Length,
};

use crate::connectors::ConnectionDirection;

/// Expects the points to be top left aligned NOT CENTERED ALIGN
// pub fn get_grid(
//     (from_pos, from_size): (Vec2, Vec2),
//     (to_pos, to_size): (Vec2, Vec2),
// ) -> [Vec2; 49] {
//     let from_rect = Rect::from_center_size(from_pos, from_size);
//     let to_rect = Rect::from_center_size(from_pos, from_size);
//
//     let smallest_rect = match from_rect.min.min(to_rect.min) {
//         x if x == from_rect.min => from_rect,
//         _ => to_rect,
//     };
//
//     [Vec2::default(); 49]
// }

// #[derive(Debug, Clone, Copy)]
// pub enum ConnectionType {
//     Flow = 0,
//     Left = 1,
//     Right = 2,
// }
//
// impl ConnectionType {
//     pub const fn from_usize(input: usize) -> Option<Self> {
//         let res = match input {
//             0 => Self::Flow,
//             1 => Self::Left,
//             2 => Self::Right,
//             _ => return None,
//         };
//
//         Some(res)
//     }
//
//     pub const fn get_color(self) -> Color {
//         match self {
//             Self::Flow => Color::YELLOW,
//             Self::Left => Color::BLUE,
//             Self::Right => Color::RED,
//         }
//     }
// }

// #[derive(Default, Debug, Clone, Copy)]
// #[non_exhaustive]
// pub enum Shape {
//     #[default]
//     Rectangle,
//     Diamond,
// }

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub enum HoleType {
    #[default]
    Unit,
    Any,
    Number,
    String,
    Bool,
    Comparitor,
    Variable,
    Type(String),
}

impl HoleType {
    pub fn valid_input(&self, value: &str) -> bool {
        if value.is_empty() {
            return false;
        }
        match self {
            HoleType::Number => value.parse::<u128>().is_ok() || value.parse::<f64>().is_ok(),
            HoleType::String => true,
            HoleType::Bool => value.eq("true") || value.eq("false"),
            HoleType::Comparitor => matches!(value, ">" | "<" | "==" | "!="),
            HoleType::Variable => {
                matches!(value.chars().next().unwrap(), 'a'..='z' | 'A'..='Z' | '_')
                    && !value.contains(char::is_whitespace)
            }
            _ => true,
        }
    }

    // This function tries to get the HoleType from the value
    pub fn get_derived_type(value: &str) -> Self {
        match value {
            e if HoleType::Number.valid_input(e) => Self::Number,
            e if HoleType::Bool.valid_input(e) => Self::Bool,
            e if HoleType::Comparitor.valid_input(e) => Self::Comparitor,
            e if HoleType::Variable.valid_input(e) => Self::Variable,
            _ => Self::Any,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Default)]
pub enum ConceptType {
    #[default]
    ControlFlow,
    Input,
    Output,
}

impl ConceptType {
    pub fn get_color(&self) -> Color {
        match self {
            ConceptType::ControlFlow => Color::rgb_u8(170, 203, 253),
            ConceptType::Input => Color::rgb_u8(208, 227, 218),
            ConceptType::Output => Color::rgb_u8(252, 240, 137),
        }
    }
}

// TODO: Custom Defaultl
#[derive(Debug, Serialize, Deserialize, Component, Clone, Default, PartialEq)]
pub struct BlockType {
    pub name: String,
    pub language: String,
    pub holes: Vec<HoleType>,
    pub connectors: Vec<ConnectionDirection>,
    pub template_string: String,
    pub in_hole: bool,
    pub value: HoleType,
    pub concept_type: ConceptType,
}

#[derive(Debug, Resource, Serialize, Deserialize)]
pub struct Language {
    pub blocks: Vec<BlockType>,
}

impl Language {
    pub fn new() -> Self {
        let file = include_str!("../../blocks/javascript.toml");
        let language: Language = toml::from_str(file).unwrap();
        language
    }
    pub fn get_block(&self, name: &str) -> Option<BlockType> {
        self.blocks
            .iter()
            .find(|block| block.name == name)
            .map(ToOwned::to_owned)
    }
}

// #[derive(Component, Debug, Clone, Copy, Default)]
// pub enum BlockType {
//     #[default]
//     Declaration,
//     Variable,
//     Text,
//     If,
//     Comparison,
//     Start,
//     Print,
// }
//
impl BlockType {
    #[inline]
    pub fn get_template(&self) -> String {
        self.template_string.clone()
    }

    #[allow(unused)]
    pub fn can_be_in_a_hole(&self) -> bool {
        self.in_hole
    }

    #[inline]
    pub fn get_holes(&self) -> usize {
        self.holes.len()
    }

    // #[inline]
    // pub fn get_connectors(&self) -> Co {
    //     self.connectors.len()
    // }
}

impl std::fmt::Display for BlockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.name, f)
    }
}

#[derive(Component, Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Position(pub Vec2);

impl Position {
    #[inline]
    #[allow(unused)]
    pub const fn x(&self) -> f32 {
        self.0.x
    }

    #[inline]
    #[allow(unused)]
    pub const fn y(&self) -> f32 {
        self.0.y
    }
}

#[allow(unused)]
pub fn log_transitions<T: States>(mut transitions: EventReader<StateTransitionEvent<T>>) {
    for transition in transitions.read() {
        info!(
            "Moving from {:?} ==> {:?}",
            transition.before, transition.after
        )
    }
}

#[allow(unused)]
pub fn print_events<E: Event + std::fmt::Debug>(mut reader: EventReader<E>) {
    for event in reader.read() {
        info!("Event was fired: {event:?}");
    }
}

/// This function checks if a line and a point is intersecting, It uses the distance from the two
/// points to do so. If the addition of the distances are equal to length of the line then we can
/// say that the point is colliding and even get where we are colliding
pub fn point_line_collision(
    (from_vec, to_vec): (Vec2, Vec2),
    point: Vec2,
    buffer: Option<f32>,
) -> bool {
    let line_len = from_vec.distance(to_vec);
    let d1 = from_vec.distance(point);
    let d2 = to_vec.distance(point);
    let buffer = buffer.unwrap_or_default();
    let added_dist = d1 + d2;
    added_dist >= line_len - buffer && added_dist <= line_len + buffer
}

pub fn get_aabb2d(pos: &Position, size: &Size) -> Aabb2d {
    let center_size = size.0 / 2.;
    let position = pos.0 + center_size;
    Aabb2d::new(position, center_size)
}

/// This gets the relative direction of the target from the source
pub fn get_relative_direction(
    (&Position(src_pos), &Size(src_size)): (&Position, &Size),
    (&Position(target_pos), &Size(target_size)): (&Position, &Size),
) -> ConnectionDirection {
    let src_center = src_pos + (src_size / 2.);
    let target_center = target_pos + (target_size / 2.);
    let difference = target_center - src_center;
    let normal_vector = difference.normalize_or_zero();

    if normal_vector == Vec2::ZERO {
        info!("They are the same going to return all");
        return ConnectionDirection::Center;
    }
    let shifted_center = target_center + normal_vector;
    let distance_from_top_edge = (shifted_center.y - target_pos.y).abs();
    let distance_from_left_edge = (shifted_center.x - target_pos.x).abs();
    let distance_from_bottom_edge = (shifted_center.y - (target_pos.y + target_size.y)).abs();
    let distance_from_right_edge = (shifted_center.x - (target_pos.x + target_size.x)).abs();

    match distance_from_top_edge
        .min(distance_from_left_edge)
        .min(distance_from_bottom_edge)
        .min(distance_from_right_edge)
    {
        x if x == distance_from_right_edge => ConnectionDirection::Right,
        x if x == distance_from_bottom_edge => ConnectionDirection::Bottom,
        x if x == distance_from_left_edge => ConnectionDirection::Left,
        x if x == distance_from_top_edge => ConnectionDirection::Top,
        _ => unreachable!(),
    }
}

// #[derive(Debug, Default, Component, Clone, PartialEq, Eq, Copy)]
// pub enum BlockType {
//     #[default]
//     Start,
//     Expression,
//     Conditionals,
// }
//
// impl std::fmt::Display for BlockType {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match *self {
//             BlockType::Start => "Start",
//             BlockType::Expression => "Expression",
//             BlockType::Conditionals => "Conditionals",
//         }
//         .fmt(f)
//     }
// }
