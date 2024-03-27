use bevy::{
    input::common_conditions::{input_just_pressed, input_just_released},
    prelude::*,
    window::PrimaryWindow,
};

use crate::{
    r#box::{ActiveBox, Box, BoxHoverEvent},
    translate_vec_to_world,
    utils::{self, Position},
};

type LineTarget = crate::r#box::Box;

const LINE_COLOR: Color = Color::YELLOW;
const ACTIVE_LINE_COLOR: Color = Color::RED;

// This is a marker for a line
#[derive(Component, Clone, Copy)]
pub struct Line {
    // This should the box that we are drawing from
    from: Entity,
    // This should be the box that we are drawing to
    to: Option<Entity>,
    // This shows if a line is connected
    connected: bool,
    // This is the color of the line
    color: Color,
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

impl Line {
    pub fn spawn_default(from: Entity) -> Self {
        Self {
            from,
            to: None,
            connected: false,
            color: LINE_COLOR,
        }
    }
}

pub struct LinePlugin;

impl LinePlugin {
    fn spawn_line(
        mut command: Commands,
        active: Res<ActiveBox>,
        mut active_line: ResMut<ActivelyDrawingLine>,
    ) {
        if let Some(entity) = active.entity {
            let active_entity = command.spawn(Line::spawn_default(entity)).id();
            active_line.entity = Some(active_entity);
        }
    }

    fn draw_line(
        mut gizmos: Gizmos,
        boxes: Query<&Position, With<LineTarget>>,
        lines: Query<&Line>,
        window: Query<&Window, With<PrimaryWindow>>,
    ) {
        let window = window.single();
        for line in &lines {
            let from_position = boxes
                .get(line.from)
                .expect("Expected the `from` entity to be in the world tree")
                .0;

            let to_position = match line.to {
                None => {
                    if let Some(cursor_pos) = window.cursor_position() {
                        translate_vec_to_world(cursor_pos, window.height(), window.width())
                    } else {
                        from_position
                    }
                }
                Some(to_entity) => {
                    boxes
                        .get(to_entity)
                        .expect("Expected the `to` entity to be in the world tree")
                        .0
                }
            };

            gizmos
                .arrow_2d(from_position, to_position, line.color)
                .with_tip_length(10.);
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

            if let Some(&ConnectLineHover(hovered_target)) = last_hovered_event.clone() {
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
            // Change the new selected line
            if let Some(next_selected) = next_selected {
                let mut next_line = lines
                    .get_mut(next_selected)
                    .expect("Couldn't get a mutable reference to line");

                next_line.color = ACTIVE_LINE_COLOR;
            }

            // Change the old selected line
            if let Some(selected_entity) = selected_line.entity {
                let mut curr_line = lines
                    .get_mut(selected_entity)
                    .expect("Couldn't get a mutable reference to line");
                curr_line.color = LINE_COLOR;
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
            delete_line_writer.send(DeleteLine(line_entity));
        }
    }

    fn handle_delete_line(
        mut commands: Commands,
        mut delete_line_reader: EventReader<DeleteLine>,
        mut selected_line: ResMut<SelectedLine>,
    ) {
        for &DeleteLine(line) in delete_line_reader.read() {
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
        mut selected_event: EventWriter<SelectLine>,
    ) {
        if let Some(window) = window.get_single().ok() {
            let cursor_pos = window
                .cursor_position()
                .expect("Expected mouse to be in screen");
            for (line_entity, line) in lines.iter().filter(|(_, line)| line.connected) {
                // Unwrap is okay here because you cannot have a `to` that is none for a line that is
                // connnected
                let from_pos = positions.get(line.from).unwrap();
                let to_pos = positions.get(line.to.unwrap()).unwrap();
                if utils::point_line_collision(
                    (from_pos.0, to_pos.0),
                    translate_vec_to_world(cursor_pos, window.height(), window.width()),
                    Some(1.),
                ) {
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
                    Self::handle_delete_line,
                    utils::print_events::<SelectLine>,
                    utils::log_transitions::<LineState>,
                )
                    .chain(),
            )
            .add_systems(Last, Self::draw_line);
    }
}
