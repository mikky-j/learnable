use crate::utils::{ConnectionDirection, Size, TempLine};
use bevy::prelude::*;

type RelativePositionArgument = (Vec2, Size);

#[derive(Debug, Clone, Copy)]
pub enum RelativePosition {
    Top,
    Left,
    Right,
    Bottom,
    None,
}

impl RelativePosition {
    pub fn get_relative_x_position(
        (from_position, Size(from_size)): RelativePositionArgument,
        to_position: Vec2,
    ) -> Self {
        let x_diff = from_position.x - to_position.x;
        let is_inside = x_diff.abs() <= from_size.x / 2.;
        if is_inside {
            return RelativePosition::None;
        }
        if x_diff < 0. {
            return RelativePosition::Left;
        }
        RelativePosition::Right
    }

    pub fn get_relative_y_position(
        (from_position, Size(from_size)): RelativePositionArgument,
        to_position: Vec2,
    ) -> Self {
        let y_diff = from_position.y - to_position.y;
        let is_inside = y_diff.abs() <= from_size.y / 2.;
        if is_inside {
            return RelativePosition::None;
        }
        if y_diff < 0. {
            return RelativePosition::Top;
        }
        RelativePosition::Bottom
    }

    pub fn get_connection_direction(self) -> ConnectionDirection {
        match self {
            RelativePosition::Top => ConnectionDirection::Top,
            RelativePosition::Left => ConnectionDirection::Left,
            RelativePosition::Right => ConnectionDirection::Right,
            RelativePosition::Bottom => ConnectionDirection::Bottom,
            RelativePosition::None => ConnectionDirection::All,
        }
    }

    pub fn get_connection_direction_from_relative_position(
        (relative_x_position, relative_y_position): (RelativePosition, RelativePosition),
        position_diff: Vec2,
        connection_direction: ConnectionDirection,
    ) -> ConnectionDirection {
        match connection_direction {
            ConnectionDirection::All => {
                info!("Position Diff: {position_diff}");
                match position_diff.x.abs().total_cmp(&position_diff.y.abs()) {
                    std::cmp::Ordering::Less => relative_y_position.get_connection_direction(),
                    std::cmp::Ordering::Equal => unimplemented!(),
                    std::cmp::Ordering::Greater => relative_x_position.get_connection_direction(),
                }
            }
            direction => direction,
        }
    }

    /// This function gets the point by which a line would be connected to a box
    pub fn get_box_point(
        position: Vec2,
        size: Size,
        connection_direction: ConnectionDirection,

        // Args that I'm going to remove but just to make it work for now
        relative_positions: (RelativePosition, RelativePosition),
        position_diff: Vec2,
    ) -> Vec2 {
        const HALF_X: Vec2 = Vec2::new(0.5, 0.);
        const HALF_Y: Vec2 = Vec2::new(0., 0.5);
        match connection_direction {
            ConnectionDirection::Left => position - (size.0 * HALF_X),
            ConnectionDirection::Right => position + (size.0 * HALF_X),
            ConnectionDirection::Bottom => position - (size.0 * HALF_Y),
            ConnectionDirection::Top => position + (size.0 * HALF_Y),
            ConnectionDirection::None => position,
            direction @ ConnectionDirection::All => {
                let new_direction = Self::get_connection_direction_from_relative_position(
                    relative_positions,
                    position_diff,
                    direction,
                );
                Self::get_box_point(
                    position,
                    size,
                    new_direction,
                    relative_positions,
                    position_diff,
                )
            }
        }
    }

    /// This is a function that returns a list of lines that should be used to draw the connection
    /// between two boxes
    pub fn get_temp_lines_between_two_points(
        (from_connnection_direction, to_connection_direction): (
            ConnectionDirection,
            ConnectionDirection,
        ),
        position_diff: Vec2,
    ) -> Vec<TempLine> {
        match (from_connnection_direction, to_connection_direction) {
            (ConnectionDirection::Left, ConnectionDirection::Right) => {
                if position_diff.y == 0. {
                    vec![TempLine::Horizontal(position_diff.x)]
                } else {
                    vec![
                        TempLine::Horizontal(position_diff.x / 2.),
                        TempLine::Vertical(position_diff.y),
                        TempLine::Horizontal(position_diff.x / 2.),
                    ]
                }
            }
            (ConnectionDirection::All, _) => panic!(),
            (_, ConnectionDirection::All) => panic!(),
            _ => unimplemented!(),
        }
    }
}
