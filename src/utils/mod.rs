// mod relative_position;
mod size;
// mod temp_line;

pub use size::*;
// pub use temp_line::*;

use bevy::{math::bounding::Aabb2d, prelude::*};

use crate::connectors::ConnectionDirection;

#[derive(Debug, Clone, Copy)]
pub enum ConnectionType {
    Flow = 0,
    Left = 1,
    Right = 2,
}

impl ConnectionType {
    pub const fn from_usize(input: usize) -> Option<Self> {
        let res = match input {
            0 => Self::Flow,
            1 => Self::Left,
            2 => Self::Right,
            _ => return None,
        };

        Some(res)
    }

    pub const fn get_color(self) -> Color {
        match self {
            Self::Flow => Color::YELLOW,
            Self::Left => Color::BLUE,
            Self::Right => Color::RED,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
#[non_exhaustive]
pub enum Shape {
    #[default]
    Rectangle,
    Diamond,
}

#[derive(Component, Debug, Clone, Copy)]
pub enum BlockType {
    Declaration,
    Variable,
    Text,
    If,
    Comparison,
    Start,
}

impl BlockType {
    #[inline]
    pub const fn get_template(&self) -> &'static str {
        match self {
            BlockType::Declaration => "let {{1}} = {{2}}",
            BlockType::If => "if ({{1}}) { {{2}} } else { {{3}} }",
            BlockType::Comparison => "{{1}} {{2}} {{3}}",
            BlockType::Variable | BlockType::Text => "{{1}}",
            _ => "",
        }
    }

    #[inline]
    pub const fn get_holes(&self) -> usize {
        match self {
            BlockType::Declaration => 2,
            BlockType::If => 1,
            BlockType::Comparison => 3,
            _ => 0,
        }
    }

    #[inline]
    pub const fn get_connectors(&self) -> usize {
        match self {
            BlockType::If => 3,
            BlockType::Comparison | BlockType::Variable => 0,
            _ => 1,
        }
    }

    #[inline]
    pub fn to_string(&self) -> String {
        let val = format!("{:?}", self);
        return val;
        // match self {
        //     BlockType::Declaration => "Variable",
        //     BlockType::If => "If",
        //     BlockType::Start => "Start",
        //     BlockType::Comparison => "Comparision",
        //     BlockType::Variable => todo!(),
        //     BlockType::Text => todo!(),
        // }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Position(pub Vec2);

pub fn log_transitions<T: States>(mut transitions: EventReader<StateTransitionEvent<T>>) {
    for transition in transitions.read() {
        info!(
            "Moving from {:?} ==> {:?}",
            transition.before, transition.after
        )
    }
}

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
