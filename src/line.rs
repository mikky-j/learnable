use bevy::{
    input::common_conditions::{input_just_pressed, input_just_released},
    prelude::*,
    window::PrimaryWindow,
};

use crate::{
    r#box::{ActiveBox, BoxHoverEvent},
    translate_vec_to_world,
    utils::Position,
};

type LineTarget = crate::r#box::Box;

// This is a marker for a line
#[derive(Component, Clone, Copy)]
pub struct Line {
    // This should the box that we are drawing from
    from: Entity,
    // This should be the box that we are drawing to
    to: Option<Entity>,
    // This shows if a line is connected
    connected: bool,
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

#[derive(Event)]
pub struct ConnectLine(pub Entity);

#[derive(Event)]
pub struct ConnectLineHover(pub Option<Entity>);

#[derive(Event)]
pub struct ChangeLineState(pub LineState);

impl Line {
    pub fn spawn_default(from: Entity) -> Self {
        Self {
            from,
            to: None,
            connected: false,
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
                .arrow_2d(from_position, to_position, Color::YELLOW)
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

    fn toggle_line(
        mut line_state_write: EventWriter<ChangeLineState>,
        line_state: Res<State<LineState>>,
    ) {
        let line_state = line_state.get();

        let new_state = match &line_state {
            LineState::Drawing => LineState::NotDrawing,
            LineState::NotDrawing => LineState::Drawing,
        };

        line_state_write.send(ChangeLineState(new_state));
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
        mut writer: EventWriter<ConnectLine>, // mut next_state: ResMut<NextState<LineState>>,
    ) {
        if let Some(active_line_entity) = active_line.entity {
            if let Ok(line) = lines.get(active_line_entity) {
                if line.to.is_some() {
                    writer.send(ConnectLine(line.to.unwrap()));
                }
            }
        }
    }

    fn log_transitions(mut transitions: EventReader<StateTransitionEvent<LineState>>) {
        for transition in transitions.read() {
            info!(
                "Moving from {:?} ==> {:?}",
                transition.before, transition.after
            )
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
}

impl Plugin for LinePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<LineState>()
            .add_event::<ConnectLineHover>()
            .add_event::<ConnectLine>()
            .add_event::<ChangeLineState>()
            .insert_resource(ActivelyDrawingLine::default())
            .add_systems(OnEnter(LineState::Drawing), Self::spawn_line)
            .add_systems(
                OnExit(LineState::Drawing),
                Self::despawn_actively_drawing_line,
            )
            .add_systems(
                Update,
                (
                    Self::toggle_line.run_if(input_just_released(KeyCode::Space)),
                    (
                        Self::connect_to_hovered_box,
                        Self::connect_two_boxes.run_if(input_just_pressed(MouseButton::Left)),
                        Self::handle_active_line_changes,
                    )
                        .run_if(in_state(LineState::Drawing))
                        .chain(),
                    Self::handle_line_state_change,
                    Self::log_transitions,
                )
                    .chain(),
            )
            .add_systems(Last, Self::draw_line);
    }
}
