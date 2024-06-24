use bevy::{
    input::{
        common_conditions::{input_just_pressed, input_pressed},
        keyboard::KeyboardInput,
        mouse::MouseWheel,
    },
    prelude::*,
    render::camera::ScalingMode,
};

use crate::{
    focus::ActiveEntity,
    ui_box::{BackgroundBox, UIBox},
    utils::Position,
    GameSets,
};

#[derive(Debug, Clone, Copy, Component)]
pub struct MyCameraComponent;

pub struct CameraPlugin;

#[derive(Debug, Clone, Copy, Resource, Default)]
pub struct PanSpeed(f32);

impl CameraPlugin {
    fn spawn_camera(mut commands: Commands) {
        let camera = Camera2dBundle::default();
        commands.spawn_empty().insert((camera, MyCameraComponent));
    }

    fn move_camera(
        mut boxes: Query<&mut Position, With<UIBox>>,
        mut input_event: EventReader<KeyboardInput>,
        pan_speed: Res<PanSpeed>,
        active_entity: Res<ActiveEntity>,
        background: Query<&BackgroundBox>,
    ) {
        if active_entity
            .entity
            .filter(|entity| background.get(*entity).is_ok())
            .is_none()
        {
            return;
        }
        let speed = pan_speed.0;
        for event in input_event.read() {
            for mut position in &mut boxes {
                let offset = match event.key_code {
                    KeyCode::ArrowUp => Vec2::new(0.0, speed),
                    KeyCode::ArrowDown => Vec2::new(0.0, -speed),
                    KeyCode::ArrowRight => Vec2::new(-speed, 0.0),
                    KeyCode::ArrowLeft => Vec2::new(speed, 0.0),
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
        app.insert_resource(PanSpeed(10.0))
            .add_systems(Startup, Self::spawn_camera)
            .add_systems(Update, Self::move_camera.in_set(GameSets::Running));
    }
}
