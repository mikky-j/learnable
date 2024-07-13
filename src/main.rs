use bevy::{
    prelude::*,
    utils::{info, tracing::instrument::WithSubscriber},
};
use learnable::{get_default_plugins, GamePlugin};
use std::{
    cell::{OnceCell, RefCell},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Mutex, OnceLock,
    },
};

fn main() {
    App::new()
        .add_plugins(get_default_plugins())
        .add_plugins(GamePlugin)
        .run();
}
