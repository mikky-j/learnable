use bevy::prelude::*;

#[derive(Debug, Component)]
struct Collision;

#[derive(Debug, Event, Clone)]
struct OnCollide {
    a: Entity,
    b: Entity,
}

pub struct CollisionPlugin;

impl CollisionPlugin {
    fn handle_collision() {}
}

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<OnCollide>();
    }
}
