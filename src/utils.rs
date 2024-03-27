use core::fmt::Debug;

use bevy::prelude::*;

#[derive(Component, Clone, Copy)]
pub struct Position(pub Vec2);

#[derive(Component)]
pub struct Size(pub Vec2);

pub fn log_transitions<T: States>(mut transitions: EventReader<StateTransitionEvent<T>>) {
    for transition in transitions.read() {
        info!(
            "Moving from {:?} ==> {:?}",
            transition.before, transition.after
        )
    }
}

pub fn print_events<E: Event + Debug>(mut reader: EventReader<E>) {
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
