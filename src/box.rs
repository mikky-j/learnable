use crate::{
    line::LineState,
    translate_vec_to_world,
    utils::{Position, Size},
    WINDOW_HEIGHT, WINDOW_WIDTH,
};
use bevy::{
    input::common_conditions::{input_just_pressed, input_just_released},
    math::bounding::{Aabb2d, IntersectsVolume},
    prelude::*,
    sprite::MaterialMesh2dBundle,
    window::PrimaryWindow,
};
use rand::Rng;

#[derive(Component)]
pub struct Box;

#[derive(Bundle)]
pub struct BoxBundle {
    r#box: Box,
    position: Position,
    size: Size,
}

impl BoxBundle {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        BoxBundle {
            position: Position(Vec2::new(x, y)),
            r#box: Box,
            size: Size(Vec2::new(w, h)),
        }
    }
}

pub struct BoxPlugin;

#[derive(Resource, Default)]
pub struct ActiveBox {
    pub entity: Option<Entity>,
}

#[derive(States, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
enum DragState {
    DragStarted,
    #[default]
    DragEnded,
}

#[derive(Event, Default)]
struct ChangedActiveBoxEvent(pub Option<Entity>);

#[derive(Event, Default)]
pub struct BoxHoverEvent(pub Option<Entity>);

impl BoxPlugin {
    fn spawn_box(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
        for _ in 0..5 {
            let mut random = rand::thread_rng();
            let random_x = random.gen_range(-WINDOW_WIDTH / 2.0..WINDOW_WIDTH / 2.);
            let random_y = random.gen_range(-WINDOW_HEIGHT / 2.0..WINDOW_HEIGHT / 2.);

            let mesh = Mesh::from(Rectangle::new(50., 100.));
            let color_material = ColorMaterial::from(Color::WHITE);
            let box_mesh = meshes.add(mesh);
            let box_color_material = materials.add(color_material);

            commands.spawn((
                BoxBundle::new(random_x, random_y, 50., 100.),
                MaterialMesh2dBundle {
                    mesh: box_mesh.into(),
                    material: box_color_material,
                    ..Default::default()
                },
            ));
        }
    }
    fn change_box_color_for_hover(
        mut boxes: Query<(Entity, &mut Handle<ColorMaterial>), With<Box>>,
        mut reader: EventReader<BoxHoverEvent>,
        mut materials: ResMut<Assets<ColorMaterial>>,
        active: Res<ActiveBox>,
    ) {
        for &BoxHoverEvent(hovered_entity) in reader.read() {
            for (entity, color_handle) in &mut boxes {
                if active
                    .entity
                    .is_some_and(|active_entity| entity == active_entity)
                {
                    continue;
                }

                let color_material = materials
                    .get_mut(color_handle.id())
                    .expect("Expected Material Handle to be there");

                if hovered_entity.is_some_and(|hovered_entity| hovered_entity == entity) {
                    *color_material = Color::GREEN.into();
                } else {
                    *color_material = Color::WHITE.into();
                }
            }
        }
    }

    fn change_box_color_for_active(
        mut boxes: Query<&mut Handle<ColorMaterial>, With<Box>>,
        mut reader: EventReader<ChangedActiveBoxEvent>,
        mut materials: ResMut<Assets<ColorMaterial>>,
        active: Res<ActiveBox>,
    ) {
        for &ChangedActiveBoxEvent(new_active) in reader.read() {
            if let Some(old_active) = active.entity {
                let color_handle = boxes
                    .get_mut(old_active)
                    .expect("Expected Old Entity to still be in the world tree");

                let color_materials = materials
                    .get_mut(color_handle.id())
                    .expect("Expected the Color Material to stil be there");

                *color_materials = Color::WHITE.into();
            }

            if let Some(new_active) = new_active {
                let color_handle = boxes
                    .get_mut(new_active)
                    .expect("Expected the entity to still be in the world tree");

                let color = materials
                    .get_mut(color_handle.id())
                    .expect("Expected the Color Material to still be there");

                *color = Color::RED.into();
            }
        }
    }

    fn change_active(
        mut active: ResMut<ActiveBox>,
        mut reader: EventReader<ChangedActiveBoxEvent>,
    ) {
        for &ChangedActiveBoxEvent(new_active) in reader.read() {
            active.entity = new_active;
        }
    }

    fn check_if_hover_on_box(
        mut boxes: Query<(Entity, &Position, &Size), With<Box>>,
        mut writer: EventWriter<BoxHoverEvent>,
        window: Query<&Window, With<PrimaryWindow>>,
    ) {
        let window = window.single();
        let window_height = window.height();
        let window_width = window.width();
        let cursor_pos = window.cursor_position().unwrap_or_default();
        let translated_cursor = translate_vec_to_world(cursor_pos, window_height, window_width);

        for (entity, translation, size) in &mut boxes {
            let collision = Aabb2d::new(translation.0, size.0 / 2.)
                .intersects(&Aabb2d::new(translated_cursor, Vec2::new(1., 1.)));

            if collision {
                writer.send(BoxHoverEvent(Some(entity)));
                return;
            }
        }
        writer.send_default();
    }

    fn check_if_box_clicked(
        mut writer: EventWriter<ChangedActiveBoxEvent>,
        mouse_button_input: Res<ButtonInput<MouseButton>>,
        mut hover_reader: EventReader<BoxHoverEvent>,
    ) {
        if mouse_button_input.just_pressed(MouseButton::Left) {
            if let Some(&BoxHoverEvent(Some(entity))) = hover_reader.read().last() {
                writer.send(ChangedActiveBoxEvent(Some(entity)));
            } else {
                writer.send_default();
            }
        }
    }

    fn move_box_according_to_mouse(
        window: Query<&Window, With<PrimaryWindow>>,
        mut boxes: Query<&mut Position, With<Box>>,
        active: Res<ActiveBox>,
        mut cursor_motions: EventReader<CursorMoved>,
    ) {
        let window = window.single();
        if let Some(entity) = active.entity {
            // if mouse_button_input.just_pressed(MouseButton::Left) {
            for cursor in cursor_motions.read() {
                // If we clicked
                let mut positions = boxes
                    .get_mut(entity)
                    .expect("Expected Active Entity to exist");

                positions.0 =
                    translate_vec_to_world(cursor.position, window.height(), window.width());
            }
            // }
        }
    }

    fn update_positions(mut query: Query<(&mut Transform, &Position)>) {
        for (mut transform, position) in &mut query {
            transform.translation = position.0.extend(0.);
        }
    }

    fn start_drag(mut next_state: ResMut<NextState<DragState>>) {
        next_state.set(DragState::DragStarted);
    }

    fn end_drag(mut next_state: ResMut<NextState<DragState>>) {
        next_state.set(DragState::DragEnded);
    }
}

impl Plugin for BoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChangedActiveBoxEvent>()
            .add_event::<BoxHoverEvent>()
            .init_state::<DragState>()
            .insert_resource(ActiveBox::default())
            .add_systems(Startup, Self::spawn_box)
            .add_systems(
                Update,
                (
                    (
                        Self::check_if_hover_on_box,
                        Self::check_if_box_clicked.run_if(in_state(LineState::NotDrawing)),
                        Self::change_box_color_for_hover,
                        Self::change_box_color_for_active.run_if(in_state(LineState::NotDrawing)),
                        Self::change_active.run_if(in_state(LineState::NotDrawing)),
                    )
                        .run_if(in_state(DragState::DragEnded))
                        .chain(),
                    Self::start_drag.run_if(
                        in_state(LineState::NotDrawing)
                            .and_then(input_just_pressed(MouseButton::Left)),
                    ),
                    Self::move_box_according_to_mouse.run_if(in_state(DragState::DragStarted)),
                    Self::end_drag.run_if(input_just_released(MouseButton::Left)),
                    Self::update_positions,
                )
                    .chain(),
            );
    }
}
