use bevy::prelude::*;
use learnable::{get_default_plugins, GamePlugin};

fn main() {
    App::new()
        .add_plugins(get_default_plugins())
        .add_plugins(GamePlugin)
        .run();
}
