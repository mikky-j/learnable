use std::f32::consts::PI;

use crate::{
    ast::{AddToASTEvent, RemoveFromAST},
    line::{DeleteLine, Line, LineState},
    translate_vec_to_world,
    utils::{BlockType, ConnectionDirection, ConnectionType, Position, Shape, Size},
};
use bevy::{
    input::{
        common_conditions::{input_just_pressed, input_just_released},
        keyboard::KeyboardInput,
        ButtonState,
    },
    math::bounding::{Aabb2d, IntersectsVolume},
    prelude::*,
    sprite::MaterialMesh2dBundle,
    text::Text2dBounds,
    window::PrimaryWindow,
};

#[derive(Component)]
pub struct Box;

#[derive(Bundle)]
pub struct BoxBundle {
    r#box: Box,
    connection_direction: ConnectionDirection,
    position: Position,
    size: Size,
}

impl BoxBundle {
    pub fn new(x: f32, y: f32, w: f32, h: f32, connection_direction: ConnectionDirection) -> Self {
        BoxBundle {
            position: Position(Vec2::new(x, y)),
            r#box: Box,
            connection_direction,
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
enum BoxDragState {
    DragStarted,
    #[default]
    DragEnded,
}

#[derive(Debug, Default, Clone)]
pub struct SpawnBox {
    shape: Shape,
    position: (f32, f32),
    connection_direction: ConnectionDirection,
    size: (f32, f32),
    color: Color,
    text: Option<String>,
    block_type: BlockType,
}

#[derive(Event, Default)]
struct ChangedActiveBoxEvent(pub Option<Entity>);

#[derive(Event, Default, Debug)]
pub struct BoxHoverEvent(pub Option<Entity>);

#[derive(Event, Default, Debug, Clone)]
pub struct SpawnBoxEvent(pub SpawnBox);

#[derive(Event, Debug, Clone, Copy)]
pub struct DeleteBoxEvent(pub Entity);

impl BoxPlugin {
    fn print_tree(boxes: Query<(Entity, &BlockType), With<Box>>, lines: Query<&Line>) {
        let mut output: String = String::new();

        let lines = lines
            .iter()
            .filter(|line| line.to.is_some())
            .collect::<Vec<_>>();

        let Some((start_box_entity, block_type)) = boxes
            .iter()
            .find(|&(_, block_type)| *block_type == BlockType::Start)
        else {
            return;
        };

        // Start box would only have one connection
        let mut current_line = lines
            .iter()
            .find(|&&line| line.from == start_box_entity)
            .unwrap();

        output.push_str(block_type.to_string().as_ref());

        loop {
            let next_entity = current_line.to;
            match next_entity {
                Some(entity) => {
                    let (_, block_type) = boxes.get(entity).unwrap();
                    output.push_str(&format!(" -> {block_type}"));
                    current_line = match lines.iter().find(|line| line.from == entity) {
                        Some(line) => line,
                        None => break,
                    };
                }
                None => break,
            }
        }

        info!("The tree is as follows: {}", output);
    }

    fn spawn_new_box(
        mut spawn_event: EventReader<SpawnBoxEvent>,
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
        mut ast_writer: EventWriter<AddToASTEvent>,
    ) {
        for SpawnBoxEvent(box_info) in spawn_event.read() {
            let box_info = box_info.to_owned();
            let mut box_entity = match box_info.shape {
                Shape::Rectangle => {
                    let mesh = Mesh::from(Rectangle::new(box_info.size.0, box_info.size.1));
                    let box_mesh = meshes.add(mesh);
                    let color_material = ColorMaterial::from(box_info.color);
                    let box_color_material = materials.add(color_material);

                    commands.spawn((
                        BoxBundle::new(
                            box_info.position.0,
                            box_info.position.1,
                            box_info.size.0,
                            box_info.size.1,
                            box_info.connection_direction,
                        ),
                        MaterialMesh2dBundle {
                            mesh: box_mesh.clone().into(),
                            material: box_color_material.clone(),
                            ..Default::default()
                        },
                        box_info.block_type,
                    ))
                }
                Shape::Diamond => {
                    let mesh = Mesh::from(Rectangle::new(box_info.size.0, box_info.size.1));
                    let box_mesh = meshes.add(mesh);
                    let color_material = ColorMaterial::from(box_info.color);
                    let box_color_material = materials.add(color_material);

                    let mut transform = Transform::default();

                    transform.rotate_z(PI / 4.);

                    commands.spawn((
                        BoxBundle::new(
                            box_info.position.0,
                            box_info.position.1,
                            box_info.size.0,
                            box_info.size.1,
                            box_info.connection_direction,
                        ),
                        MaterialMesh2dBundle {
                            mesh: box_mesh.clone().into(),
                            material: box_color_material.clone(),
                            transform,
                            ..Default::default()
                        },
                        box_info.block_type,
                    ))
                }
                _ => unimplemented!(),
            };

            if let Some(text) = box_info.text {
                box_entity.with_children(|builder| {
                    let mut transform = Transform::from_translation(Vec3::Z);

                    if matches!(box_info.shape, Shape::Diamond) {
                        transform.rotate_z(-PI / 4.);
                    }

                    builder.spawn(Text2dBundle {
                        text: Text {
                            sections: vec![TextSection::new(
                                text,
                                TextStyle {
                                    color: Color::BLACK,
                                    ..default()
                                },
                            )],
                            ..default()
                        },
                        text_2d_bounds: Text2dBounds {
                            size: box_info.size.into(),
                        },
                        transform,
                        ..default()
                    });
                });
            }

            ast_writer.send(AddToASTEvent {
                parent: None,
                child: box_entity.id(),
                connection_type: ConnectionType::Flow,
            });
        }
    }

    fn spawn_initial_box(mut box_writer: EventWriter<SpawnBoxEvent>) {
        let new_box = SpawnBox {
            shape: Shape::Rectangle,
            position: (0., 0.),
            connection_direction: ConnectionDirection::All,
            size: (50., 50.),
            color: Color::WHITE,
            text: Some("Start block".into()),
            block_type: BlockType::Start,
        };
        box_writer.send(SpawnBoxEvent(new_box));
    }

    fn spawn_box(
        mut box_writer: EventWriter<SpawnBoxEvent>,
        mut keyboard_events: EventReader<KeyboardInput>,
    ) {
        for keyboard_input in keyboard_events.read() {
            let current_shape = match (keyboard_input.state, keyboard_input.key_code) {
                (ButtonState::Released, KeyCode::KeyS) => Shape::Rectangle,
                (ButtonState::Released, KeyCode::KeyD) => Shape::Diamond,
                _ => continue,
            };

            let block_type = match current_shape {
                Shape::Rectangle => BlockType::Expression,
                Shape::Diamond => BlockType::Conditionals,
            };

            let new_box = SpawnBox {
                shape: current_shape,
                position: (0., 0.),
                connection_direction: ConnectionDirection::All,
                size: (50., 50.),
                color: Color::WHITE,
                text: Some(format!("{block_type} block")),
                block_type,
            };

            box_writer.send(SpawnBoxEvent(new_box));
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
            for cursor in cursor_motions.read() {
                let mut positions = boxes
                    .get_mut(entity)
                    .expect("Expected Active Entity to exist");

                positions.0 =
                    translate_vec_to_world(cursor.position, window.height(), window.width());
                // info!("New Box Position for {:?}: {}", entity, positions.0);
            }
        }
    }

    fn update_positions(mut query: Query<(&mut Transform, &Position), With<Box>>) {
        for (mut transform, position) in &mut query {
            transform.translation = position.0.extend(0.);
        }
    }

    fn start_drag(
        mut next_state: ResMut<NextState<BoxDragState>>,
        mut hover_reader: EventReader<BoxHoverEvent>,
    ) {
        if hover_reader
            .read()
            .last()
            .and_then(|&BoxHoverEvent(event)| event)
            .is_some()
        {
            next_state.set(BoxDragState::DragStarted);
        }
    }

    fn end_drag(mut next_state: ResMut<NextState<BoxDragState>>) {
        next_state.set(BoxDragState::DragEnded);
    }

    fn move_box_according_keyboard(
        active: Res<ActiveBox>,
        mut boxes: Query<&mut Position, With<Box>>,
        mut keyboard_events: EventReader<KeyboardInput>,
    ) {
        if let Some(active_entity) = active.entity {
            let mut position = boxes
                .get_mut(active_entity)
                .expect("Expected box to have a position");

            for keyboard_event in keyboard_events.read() {
                let vector = match keyboard_event.key_code {
                    KeyCode::ArrowUp => Vec2::Y,
                    KeyCode::ArrowDown => Vec2::NEG_Y,
                    KeyCode::ArrowLeft => Vec2::NEG_X,
                    KeyCode::ArrowRight => Vec2::X,
                    _ => Vec2::ZERO,
                };
                position.0 += vector
            }
        }
    }

    fn delete_box(
        active: Res<ActiveBox>,
        lines: Query<(Entity, &Line)>,
        mut delete_box_writer: EventWriter<DeleteBoxEvent>,
        mut delete_line_writer: EventWriter<DeleteLine>,
        mut remove_ast: EventWriter<RemoveFromAST>,
    ) {
        if let Some(box_entity) = active.entity {
            // Filter out all the lines that are connected to the currrent box
            let lines = lines.iter().filter_map(|(e, line)| {
                if line.from == box_entity || line.to.is_some_and(|to| to == box_entity) {
                    Some(DeleteLine(e))
                } else {
                    None
                }
            });
            // Delete all the lines connected to the box
            delete_line_writer.send_batch(lines);

            // Delete the box itself
            delete_box_writer.send(DeleteBoxEvent(box_entity));

            // Remove the box from the AST
            remove_ast.send(RemoveFromAST {
                parent: None,
                child: Some(box_entity),
                connection_type: ConnectionType::Flow,
            });
        }
    }

    fn handle_delete_box(
        mut commands: Commands,
        mut reader: EventReader<DeleteBoxEvent>,
        mut active: ResMut<ActiveBox>,
    ) {
        for &DeleteBoxEvent(box_entity) in reader.read() {
            let entity_commands = commands.get_entity(box_entity);
            if let Some(command) = entity_commands {
                command.despawn_recursive();
                // Set the active Entity to None
                active.entity = None;
            }
        }
    }
}

impl Plugin for BoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChangedActiveBoxEvent>()
            .add_event::<SpawnBoxEvent>()
            .add_event::<BoxHoverEvent>()
            .add_event::<DeleteBoxEvent>()
            .init_state::<BoxDragState>()
            .insert_resource(ActiveBox::default())
            // .add_systems(Startup, Self::spawn_initial_box)
            .add_systems(
                Update,
                (
                    Self::spawn_box,
                    (
                        Self::check_if_hover_on_box,
                        Self::check_if_box_clicked.run_if(in_state(LineState::NotDrawing)),
                        Self::change_box_color_for_hover,
                        Self::change_box_color_for_active.run_if(in_state(LineState::NotDrawing)),
                        Self::change_active.run_if(in_state(LineState::NotDrawing)),
                        Self::move_box_according_keyboard,
                    )
                        .run_if(in_state(BoxDragState::DragEnded))
                        .chain(),
                    Self::start_drag.run_if(
                        in_state(LineState::NotDrawing)
                            .and_then(input_just_pressed(MouseButton::Left)),
                    ),
                    Self::move_box_according_to_mouse.run_if(in_state(BoxDragState::DragStarted)),
                    Self::end_drag.run_if(input_just_released(MouseButton::Left)),
                    Self::update_positions,
                    // Self::spawn_new_box,
                    Self::delete_box.run_if(
                        in_state(LineState::NotDrawing)
                            .and_then(input_just_released(KeyCode::Backspace)),
                    ),
                    // Self::print_tree.run_if(
                    //     in_state(LineState::NotDrawing)
                    //         .and_then(input_just_released(KeyCode::KeyP)),
                    // ),
                    // print_events::<BoxHoverEvent>,
                )
                    .chain(),
            )
            .add_systems(Last, Self::handle_delete_box);
    }
}
