use bevy::{
    input::{
        common_conditions::{input_just_pressed, input_pressed},
        keyboard::KeyboardInput,
    },
    prelude::*,
    render::camera::ScalingMode,
};

use crate::{
    ui_box::{BackgroundBox, UIBox},
    utils::Position,
    GameSets,
};

#[derive(Debug, Clone, Copy, Component)]
pub struct MyCameraComponent;

pub struct CameraPlugin;

impl CameraPlugin {
    fn spawn_camera(mut commands: Commands) {
        let camera = Camera2dBundle::default();
        commands.spawn_empty().insert((camera, MyCameraComponent));
    }

    fn move_camera(
        mut boxes: Query<&mut Position, With<UIBox>>,
        mut input_event: EventReader<KeyboardInput>,
    ) {
        for event in input_event.read() {
            for mut position in &mut boxes {
                let offset = match event.key_code {
                    KeyCode::ArrowDown => Vec2::new(0.0, 3.0),
                    KeyCode::ArrowUp => Vec2::new(0.0, -3.0),
                    KeyCode::ArrowLeft => Vec2::new(-3.0, 0.0),
                    KeyCode::ArrowRight => Vec2::new(3.0, 0.0),
                    _ => Vec2::ZERO,
                };

                position.0 += offset;
            }
        }
    }

    fn zoom_camera(mut ui_scale: ResMut<UiScale>) {
        ui_scale.0 += 0.25;
    }
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::spawn_camera)
            .add_systems(Update, Self::move_camera.in_set(GameSets::Running));
    }
}
