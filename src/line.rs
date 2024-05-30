use bevy::{
    input::common_conditions::{input_just_pressed, input_just_released},
    prelude::*,
    utils::info,
    window::PrimaryWindow,
};

use crate::{
    ast::{AddToASTEvent, Ast, RemoveFromAST},
    r#box::{ActiveBox, Box, BoxHoverEvent},
    translate_vec_to_world,
    utils::{
        self, ConnectionDirection, ConnectionType, Position, RelativePosition, Size, TempLine,
    },
};

type LineTarget = crate::r#box::Box;

const ACTIVE_LINE_COLOR: Color = Color::RED;

// This is a marker for a line
#[derive(Component, Clone, Copy)]
pub struct Line {
    // This should the box that we are drawing from
    pub from: Entity,
    // This should be the box that we are drawing to
    pub to: Option<Entity>,
    // This shows if a line is connected
    pub connected: bool,
    // This is the color of the line
    pub color: Color,
    pub connection_type: ConnectionType,
}

#[derive(Debug, States, Hash, PartialEq, Eq, PartialOrd, Ord, Default, Clone, Copy)]
pub enum LineState {
    Drawing,
    #[default]
    NotDrawing,
}

#[derive(Resource, Default)]
pub struct ActivelyDrawingLine {
    entity: Option<Entity>,
}

#[derive(Resource, Default)]
pub struct SelectedLine {
    entity: Option<Entity>,
}

#[derive(Event)]
pub struct ConnectLine(pub Entity);

#[derive(Event)]
pub struct ConnectLineHover(pub Option<Entity>);

#[derive(Event)]
pub struct ChangeLineState(pub LineState);

#[derive(Event, Debug, Default)]
pub struct SelectLine(pub Option<Entity>);

#[derive(Event, Debug)]
pub struct DeleteLine(pub Entity);

#[derive(Debug, Reflect, Default, GizmoConfigGroup)]
pub struct LineGizmos;

impl Line {
    pub const fn spawn_default(from: Entity, connection_type: ConnectionType) -> Self {
        Self {
            from,
            to: None,
            connected: false,
            color: connection_type.get_color(),
            connection_type,
        }
    }
}

pub struct LinePlugin;

impl LinePlugin {
    fn spawn_line(
        global_ast: Res<Ast>,
        mut command: Commands,
        active: Res<ActiveBox>,
        mut active_line: ResMut<ActivelyDrawingLine>,
    ) {
        if let Some(entity) = active.entity {
            // Get the connections it already has if it has it
            let connections = global_ast.map.get(&entity);

            // Get the connection type. It should be the first value with none in the array
            let connection_type = if let Some(connections) = connections {
                if let Some(connection_count) = connections
                    .iter()
                    .enumerate()
                    .find_map(|(index, val)| (if val.is_none() { Some(index) } else { None }))
                {
                    connection_count
                } else {
                    info!(
                        "{entity:?} already has 3 connections which is the maximum that we can go"
                    );
                    return;
                }
            } else {
                0
            };

            if let Some(connection_type) = ConnectionType::from_usize(connection_type) {
                let active_entity = command
                    .spawn(Line::spawn_default(entity, connection_type))
                    .id();
                active_line.entity = Some(active_entity);
            }
        }
    }

    fn configure_line(mut gizmos_store: ResMut<GizmoConfigStore>) {
        let (line_config, _) = gizmos_store.config_mut::<LineGizmos>();
        line_config.line_width = 1.;
    }

    fn draw_direct_line(
        mut gizmos: Gizmos<LineGizmos>,
        boxes: Query<&Position, With<LineTarget>>,
        lines: Query<&Line>,
        window: Query<&Window, With<PrimaryWindow>>,
    ) {
        let window = if let Some(window) = window.iter().next() {
            window
        } else {
            info!("No window found");
            return;
        };

        for line in &lines {
            let from_position = boxes
                .get(line.from)
                .expect("Expected the `from` entity to be  the world tree")
                .0;

            let to_position = line.to.map_or_else(
                || {
                    window
                        .cursor_position()
                        .map_or(from_position, |cursor_pos| {
                            translate_vec_to_world(cursor_pos, window.height(), window.width())
                        })
                },
                |to_entity| {
                    let to_position = boxes
                        .get(to_entity)
                        .expect("Expected the `to` entity to be in the world tree");
                    to_position.0
                },
            );

            gizmos
                .arrow_2d(from_position, to_position, line.color)
                .with_tip_length(5.);
        }
    }

    fn draw_line(
        mut gizmos: Gizmos<LineGizmos>,
        boxes: Query<(&Position, &Size, &ConnectionDirection), With<LineTarget>>,
        lines: Query<&Line>,
        window: Query<&Window, With<PrimaryWindow>>,
    ) {
        let window = window.single();
        for line in &lines {
            let (from_position, &from_size, &from_connection_dir) = boxes
                .get(line.from)
                .expect("Expected the `from` entity to be  the world tree");

            let from_position = from_position.0;

            let (to_position, to_size, to_connection_dir) = match line.to {
                None => {
                    if let Some(cursor_pos) = window.cursor_position() {
                        (
                            translate_vec_to_world(cursor_pos, window.height(), window.width()),
                            Size::default(),
                            ConnectionDirection::None,
                        )
                    } else {
                        (from_position, Size::default(), ConnectionDirection::None)
                    }
                }
                Some(to_entity) => {
                    let (to_position, &to_size, &to_connection_dir) = boxes
                        .get(to_entity)
                        .expect("Expected the `to` entity to be in the world tree");
                    (to_position.0, to_size, to_connection_dir)
                }
            };

            let relative_x_position =
                RelativePosition::get_relative_x_position((from_position, from_size), to_position);
            let relative_y_position =
                RelativePosition::get_relative_y_position((from_position, from_size), to_position);

            let x_diff = from_position.x - to_position.x;
            let y_diff = from_position.y - to_position.y;

            let mut starting_point = RelativePosition::get_box_point(
                from_position,
                from_size,
                from_connection_dir,
                (relative_x_position, relative_y_position),
                from_position - to_position,
            );

            // let mut starting_point = match (relative_x_position, relative_y_position) {
            //     (RelativePosition::Left, _) => from_position - from_size.0 * Vec2::new(0.5, 0.),
            //     (RelativePosition::Right, _) => from_position + from_size.0 * Vec2::new(0.5, 0.),
            //     (_, RelativePosition::Top) => from_position - from_size.0 * Vec2::new(0., 0.5),
            //     (_, RelativePosition::Bottom) => from_position + from_size.0 * Vec2::new(0., 0.5),
            //     _ => continue,
            // };

            let mut temp_lines: Vec<TempLine> = match (from_connection_dir, to_connection_dir) {
                (ConnectionDirection::All, ConnectionDirection::All) => {
                    match (relative_x_position, relative_y_position) {
                        (RelativePosition::Left, RelativePosition::Top)
                        | (RelativePosition::Left, RelativePosition::Bottom)
                        | (RelativePosition::Right, RelativePosition::Top)
                        | (RelativePosition::Right, RelativePosition::Bottom) => {
                            vec![TempLine::Horizontal(x_diff), TempLine::Vertical(y_diff)]
                        }
                        (RelativePosition::Left, RelativePosition::None)
                        | (RelativePosition::Right, RelativePosition::None) => {
                            if y_diff != 0. {
                                vec![
                                    TempLine::Horizontal(x_diff / 2.),
                                    TempLine::Vertical(y_diff),
                                    TempLine::Horizontal(x_diff / 2.),
                                ]
                            } else {
                                vec![TempLine::Horizontal(x_diff / 2.)]
                            }
                        }
                        (RelativePosition::None, RelativePosition::Top)
                        | (RelativePosition::None, RelativePosition::Bottom) => {
                            if x_diff != 0. {
                                vec![
                                    TempLine::Vertical(y_diff / 2.),
                                    TempLine::Horizontal(x_diff),
                                    TempLine::Vertical(y_diff / 2.),
                                ]
                            } else {
                                vec![TempLine::Vertical(y_diff / 2.)]
                            }
                        }
                        (RelativePosition::None, RelativePosition::None) => vec![],
                        x => {
                            info!("Not yet Implemented {x:?}");
                            continue;
                        }
                    }
                }
                (_, ConnectionDirection::None) => {
                    gizmos.arrow_2d(starting_point, to_position, line.color);
                    continue;
                }
                x => {
                    info!("Not yet Implemented {x:?}");
                    continue;
                }
            };

            let last = match temp_lines.pop() {
                Some(lines) => lines,
                // If there are no lines then we skip drawing the line
                None => continue,
            };

            let end_point_padding = match (relative_x_position, relative_y_position) {
                (RelativePosition::Left, _) => to_size.0 * Vec2::new(0.5, 0.),
                (RelativePosition::Right, _) => to_size.0 * Vec2::new(-0.5, 0.),
                (_, RelativePosition::Top) => to_size.0 * Vec2::new(0., 0.5),
                (_, RelativePosition::Bottom) => to_size.0 * Vec2::new(0., -0.5),
                _ => continue,
            };

            // Draw all the other lines
            for temp_line in temp_lines.into_iter() {
                let end_point = match temp_line {
                    TempLine::Horizontal(width) => starting_point + Vec2::new(width, 0.),
                    TempLine::Vertical(height) => starting_point + Vec2::new(0., height),
                    TempLine::None => continue,
                };

                gizmos.line_2d(starting_point, end_point, line.color);
                starting_point = end_point
            }

            // Draw all the end line
            let end_point = match last {
                TempLine::Horizontal(width) => {
                    starting_point + Vec2::new(width, 0.) + end_point_padding
                }
                TempLine::Vertical(height) => {
                    starting_point + Vec2::new(0., height) + end_point_padding
                }
                TempLine::None => continue,
            };

            gizmos
                .arrow_2d(starting_point, end_point, line.color)
                .with_tip_length(3.);
        }
    }

    fn handle_line_state_change(
        mut line_state_reader: EventReader<ChangeLineState>,
        mut next_state: ResMut<NextState<LineState>>,
    ) {
        for &ChangeLineState(line_state) in line_state_reader.read() {
            next_state.set(line_state);
        }
    }

    fn toggle_line_state(
        mut line_state_write: EventWriter<ChangeLineState>,
        line_state: Res<State<LineState>>,
        active_box: Res<ActiveBox>,
    ) {
        let line_state = line_state.get();

        if active_box.entity.is_some() {
            let new_state = match &line_state {
                LineState::Drawing => LineState::NotDrawing,
                LineState::NotDrawing => LineState::Drawing,
            };

            line_state_write.send(ChangeLineState(new_state));
        } else {
            info!("Cannot change line state because there's no active box")
        }
    }

    fn despawn_actively_drawing_line(
        mut line: ResMut<ActivelyDrawingLine>,
        mut commands: Commands,
    ) {
        if let Some(active_line) = line.entity {
            commands.entity(active_line).despawn_recursive();
            line.entity = None;
        }
    }

    fn connect_to_hovered_box(
        active_line: Res<ActivelyDrawingLine>,
        mut lines: Query<&mut Line>,
        mut hovered_reader: EventReader<BoxHoverEvent>,
        mut writer: EventWriter<ConnectLineHover>,
    ) {
        let active_box = hovered_reader
            .read()
            .map(|&BoxHoverEvent(entity)| entity)
            .last()
            .flatten();
        // info!("Active Box: {active_box:?}");

        // This unwrap should be safe because this will only run when we enter the Line drawing
        // start
        if let Some(active_line) = active_line.entity {
            if let Ok(line_entity) = lines.get_mut(active_line) {
                // If we are not hovering on the same box that we are drawing the line from
                if active_box.is_some_and(|entity| {
                    entity != line_entity.from
                        || line_entity
                            .to
                            .is_some_and(|line_entity| entity != line_entity)
                }) || active_box.is_none()
                {
                    writer.send(ConnectLineHover(active_box));
                }
            }
        }
    }

    fn connect_two_boxes(
        active_line: ResMut<ActivelyDrawingLine>,
        lines: Query<&Line>,
        mut writer: EventWriter<ConnectLine>,
    ) {
        if let Some(active_line_entity) = active_line.entity {
            if let Ok(line) = lines.get(active_line_entity) {
                if line.to.is_some() {
                    writer.send(ConnectLine(line.to.unwrap()));
                }
            }
        }
    }

    fn add_boxes_to_ast(
        active_line: Res<ActivelyDrawingLine>,
        lines: Query<&Line>,
        mut connect_line_reader: EventReader<ConnectLine>,
        mut ast_writer: EventWriter<AddToASTEvent>,
    ) {
        if let Some(active_line_entity) = active_line.entity {
            for &ConnectLine(to) in connect_line_reader.read() {
                let active_line = lines
                    .get(active_line_entity)
                    .expect("Active Line should be in the world tree");
                ast_writer.send(AddToASTEvent {
                    parent: Some(active_line.from),
                    child: to,
                    connection_type: active_line.connection_type,
                });
            }
        }
    }

    fn handle_active_line_changes(
        mut active_line: ResMut<ActivelyDrawingLine>,
        mut lines: Query<&mut Line>,
        active_box: Res<ActiveBox>,
        mut connect_line_reader: EventReader<ConnectLine>,
        mut connect_hover_reader: EventReader<ConnectLineHover>,
        mut line_state_write: EventWriter<ChangeLineState>,
    ) {
        if let Some(active_line_entity) = active_line.entity {
            let last_connected_event = connect_line_reader.read().last();

            let last_hovered_event = connect_hover_reader.read().last();

            if let Some(&ConnectLineHover(hovered_target)) = last_hovered_event {
                if hovered_target.is_some()
                    && lines.iter().any(|&line| {
                        line.connected
                            && line.from == active_box.entity.expect("Expected Active box")
                            && line.to == hovered_target
                    })
                {
                    return;
                }
            }

            let mut active_line_entity = lines
                .get_mut(active_line_entity)
                .expect("Expected the active line to be in the world tree");

            if !active_line_entity.connected {
                if let Some(&ConnectLine(box_entity)) = last_connected_event {
                    active_line_entity.to = Some(box_entity);
                    active_line.entity = None;
                    active_line_entity.connected = true;
                    line_state_write.send(ChangeLineState(LineState::NotDrawing));
                } else if let Some(&ConnectLineHover(box_entity)) = last_hovered_event {
                    active_line_entity.to = box_entity;
                }
            }
        }
    }

    fn handle_select_line(
        mut lines: Query<&mut Line>,
        mut selected_event_reader: EventReader<SelectLine>,
        selected_line: Res<SelectedLine>,
    ) {
        // Only modify the colors if there's a new event fired
        for &SelectLine(next_selected) in selected_event_reader.read() {
            if next_selected == selected_line.entity {
                continue;
            }
            // Change the new selected line
            if let Some(next_selected) = next_selected {
                let mut next_line = lines
                    .get_mut(next_selected)
                    .expect("Couldn't get a mutable reference to the line");
                info("Changing the line color");

                next_line.color = ACTIVE_LINE_COLOR;
            }

            // Change the old selected line
            if let Some(selected_entity) = selected_line.entity {
                let mut curr_line = lines
                    .get_mut(selected_entity)
                    .expect("Couldn't get a mutable reference to line");
                curr_line.color = curr_line.connection_type.get_color();
            }
        }
    }

    fn change_selected_line(
        mut selected_line: ResMut<SelectedLine>,
        mut selected_event_reader: EventReader<SelectLine>,
    ) {
        for &SelectLine(new_line) in selected_event_reader.read() {
            selected_line.entity = new_line;
        }
    }

    fn delete_selected_line(
        selected_line: Res<SelectedLine>,
        mut delete_line_writer: EventWriter<DeleteLine>,
    ) {
        if let Some(line_entity) = selected_line.entity {
            // Delete the line
            delete_line_writer.send(DeleteLine(line_entity));
        }
    }

    fn handle_delete_line(
        mut commands: Commands,
        mut delete_line_reader: EventReader<DeleteLine>,
        mut selected_line: ResMut<SelectedLine>,
        lines: Query<&Line>,
        mut remove_ast_writer: EventWriter<RemoveFromAST>,
    ) {
        for &DeleteLine(line) in delete_line_reader.read() {
            // Remove from AST
            let l = lines.get(line).expect("Line should be in the entity tree");
            if l.connected {
                remove_ast_writer.send(RemoveFromAST {
                    parent: Some(l.from),
                    child: None,
                    connection_type: l.connection_type,
                });
            }

            // Remove line
            if let Some(line_commands) = commands.get_entity(line) {
                line_commands.despawn_recursive();
                selected_line.entity = None;
            }
        }
    }

    fn select_line(
        positions: Query<&Position, With<Box>>,
        lines: Query<(Entity, &Line)>,
        window: Query<&Window, With<PrimaryWindow>>,
        mut hover_boxes: EventReader<BoxHoverEvent>,
        mut selected_event: EventWriter<SelectLine>,
    ) {
        let is_hovering = hover_boxes
            .read()
            .any(|BoxHoverEvent(event)| event.is_some());
        if let Ok(window) = window.get_single() {
            let cursor_pos = window
                .cursor_position()
                .expect("Expected mouse to be in the screen");
            for (line_entity, line) in lines.iter().filter(|(_, line)| line.connected) {
                // Unwrap is okay here because you cannot have a `to` that is none for a line that is
                // connnected
                let from_pos = positions.get(line.from).unwrap();
                let to_pos = positions.get(line.to.unwrap()).unwrap();
                if utils::point_line_collision(
                    (from_pos.0, to_pos.0),
                    translate_vec_to_world(cursor_pos, window.height(), window.width()),
                    Some(1.),
                ) && !is_hovering
                {
                    selected_event.send(SelectLine(Some(line_entity)));

                    return;
                }
            }
        }
        // This would send that there's no line selected
        selected_event.send_default();
    }
}

impl Plugin for LinePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<LineState>()
            .add_event::<ConnectLineHover>()
            .add_event::<ConnectLine>()
            .add_event::<ChangeLineState>()
            .add_event::<SelectLine>()
            .add_event::<DeleteLine>()
            .init_resource::<SelectedLine>()
            .init_resource::<ActivelyDrawingLine>()
            .init_gizmo_group::<LineGizmos>()
            .add_systems(Startup, Self::configure_line)
            .add_systems(OnEnter(LineState::Drawing), Self::spawn_line)
            .add_systems(
                OnExit(LineState::Drawing),
                Self::despawn_actively_drawing_line,
            )
            .add_systems(
                Update,
                (
                    Self::toggle_line_state.run_if(input_just_released(KeyCode::Space)),
                    (
                        Self::connect_to_hovered_box,
                        Self::connect_two_boxes.run_if(input_just_pressed(MouseButton::Left)),
                        Self::add_boxes_to_ast,
                        Self::handle_active_line_changes,
                    )
                        .run_if(in_state(LineState::Drawing))
                        .chain(),
                    Self::handle_line_state_change,
                    Self::select_line.run_if(
                        in_state(LineState::NotDrawing)
                            .and_then(input_just_pressed(MouseButton::Left)),
                    ),
                    Self::handle_select_line,
                    Self::change_selected_line,
                    Self::delete_selected_line.run_if(input_just_released(KeyCode::Backspace)),
                    // utils::print_events::<SelectLine>,
                    utils::log_transitions::<LineState>,
                )
                    .chain(),
            )
            .add_systems(
                Last,
                (Self::draw_direct_line, Self::handle_delete_line).chain(),
            );
    }
}
