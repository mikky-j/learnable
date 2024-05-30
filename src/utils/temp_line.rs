use crate::utils::ConnectionDirection;
use bevy::prelude::*;

pub enum TempLine {
    Horizontal(f32),
    Vertical(f32),
    None,
}

impl TempLine {
    pub fn wrap_around(
        (from_connnection_direction, to_connection_direction): (
            ConnectionDirection,
            ConnectionDirection,
        ),
        position_diff: Vec2,
    ) -> Vec<TempLine> {
        todo!()
    }

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
