use bevy::{input::common_conditions::input_just_released, prelude::*};

use crate::{
    connectors::{ConnectionDirection, Connector, SpawnConnector},
    focus::{ActiveEntity, DragEntity, DragState, HoverEntity},
    translate_vec_to_world,
    ui_box::{BackgroundBox, Block},
    utils::{log_transitions, print_events, Position, Size},
};

#[derive(Component, Debug, Clone, Copy)]
pub struct UiLine {
    pub from: Entity,
    /// The direction we are coming from
    pub from_direction: ConnectionDirection,
    /// By default this would be the connector entity
    pub to: Entity,
    /// The direction we are going to
    pub to_direction: ConnectionDirection,
}

#[derive(Bundle, Debug, Clone, Copy)]
pub struct LineBundle {
    line: UiLine,
}

impl LineBundle {
    pub fn new(from: Entity, from_direction: ConnectionDirection, to: Entity) -> Self {
        Self {
            line: UiLine {
                from,
                from_direction,
                to,
                to_direction: ConnectionDirection::Center,
            },
        }
    }
}

#[derive(Debug, Reflect, Default, GizmoConfigGroup)]
pub struct LineGizmos;

#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct ActivelyDrawingLine {
    pub entity: Option<Entity>,
}

#[derive(Debug, Event, Clone, Copy)]
pub struct SpawnLineEvent(pub LineBundle);

#[derive(Debug, Event, Clone, Copy)]
pub struct DeleteLine(pub Entity);

#[derive(Debug, Event, Clone, Copy, Default)]
pub struct TempConnectLine(pub Option<(Entity, ConnectionDirection)>);

#[derive(Debug, Event, Clone, Copy, Default)]
pub struct ConnectLine(pub Option<(Entity, ConnectionDirection)>);

pub struct UiLinePlugin;

impl UiLinePlugin {
    fn configure_line(mut gizmos_store: ResMut<GizmoConfigStore>) {
        let (line_config, _) = gizmos_store.config_mut::<LineGizmos>();
        line_config.line_width = 1.;
    }

    fn spawn_new_line(
        connectors: Query<(Entity, &Connector)>,
        active_drawing: Res<ActivelyDrawingLine>,
        curr_drag: Res<DragEntity>,
        mut writer: EventWriter<SpawnLineEvent>,
    ) {
        if matches!(active_drawing.entity, None) {
            if let Some((curr_drag_entity, connector)) = curr_drag
                .entity
                .map(|entity| connectors.get(entity).ok())
                .flatten()
            {
                let bundle =
                    LineBundle::new(connector.fixture, connector.direction, curr_drag_entity);
                writer.send(SpawnLineEvent(bundle));
            }
        }
    }

    fn handle_mouse_release(
        mut delete_writer: EventWriter<DeleteLine>,
        mut active_drawing: ResMut<ActivelyDrawingLine>,
        connectors: Query<&Connector>,
        lines: Query<&UiLine>,
        mut connect_writer: EventWriter<ConnectLine>,
    ) {
        if let Some(entity) = active_drawing.entity {
            let line = lines
                .get(entity)
                .expect("Expected the line to be in the world tree");
            if connectors.get(line.to).is_ok() {
                delete_writer.send(DeleteLine(entity));
            } else {
                connect_writer.send(ConnectLine(Some((line.to, line.to_direction))));
            }
            active_drawing.entity = None;
        }
    }

    fn handle_spawn_line(
        mut reader: EventReader<SpawnLineEvent>,
        mut active: ResMut<ActivelyDrawingLine>,
        mut commands: Commands,
    ) {
        for &SpawnLineEvent(bundle) in reader.read() {
            let entity = commands.spawn(bundle);
            active.entity = Some(entity.id());
        }
    }

    fn handle_delete_line(
        mut reader: EventReader<DeleteLine>,
        mut commands: Commands,
        parent: Query<(&Block, &Children)>,
        lines: Query<&UiLine>,
        mut connectors: Query<&mut Visibility, With<Connector>>,
        mut active: ResMut<ActivelyDrawingLine>,
    ) {
        for &DeleteLine(deleted_entity) in reader.read() {
            let Some(entity_commands) = commands.get_entity(deleted_entity) else {
                error!("The deleted line has already been deleted");
                continue;
            };

            // Make connector visibile
            let line = lines
                .get(deleted_entity)
                .expect("Expected the line to be deleted");

            let (_, children) = parent
                .get(line.from)
                .expect("Expected the block to have children");
            for &child in children.iter() {
                if let Ok(mut connector_visibility) = connectors.get_mut(child) {
                    *connector_visibility = Visibility::Visible;
                }
            }

            // Despawn the line
            entity_commands.despawn_recursive();

            if active.entity.is_some_and(|entity| entity == deleted_entity) {
                active.entity = None;
            }
        }
    }

    fn handle_temp_connect_line(
        mut reader: EventReader<TempConnectLine>,
        active_drawing: Res<ActivelyDrawingLine>,
        curr_drag: Res<DragEntity>,
        mut lines: Query<&mut UiLine>,
    ) {
        for &TempConnectLine(connected_entity) in reader.read() {
            if let Some(active) = active_drawing.entity {
                let mut line = lines.get_mut(active).expect("Expected a line element");
                if let Some((connected_entity, connection_direction)) = connected_entity {
                    line.to = connected_entity;
                    line.to_direction = connection_direction;
                } else {
                    line.to = curr_drag.entity.expect("Should be dragging the connector");
                    line.to_direction = ConnectionDirection::Center;
                }
            }
        }
    }

    fn draw_line(
        lines: Query<&UiLine>,
        mut gizmos: Gizmos<LineGizmos>,
        query: Query<(&Position, &Size)>,
        background: Query<&Node, With<BackgroundBox>>,
    ) {
        for line in &lines {
            let background_size = background
                .get_single()
                .expect("Expect there to be only one background")
                .size();

            let (parent_pos, parent_size) = query
                .get(line.from)
                .expect("Expected Fixture to be in the world tree");

            let (connector_pos, connector_size) = query
                .get(line.to)
                .expect("Expected Connector to be in the world tree");

            let mut from_pos = parent_pos.0
                + (parent_size.0
                    * Vec2::new(
                        line.from_direction.get_left(),
                        line.from_direction.get_top(),
                    )
                    / 100.);

            let mut to_pos = connector_pos.0
                + (connector_size.0
                    * Vec2::new(line.to_direction.get_left(), line.to_direction.get_top())
                    / 100.);

            from_pos = translate_vec_to_world(from_pos, background_size.y, background_size.x);
            to_pos = translate_vec_to_world(to_pos, background_size.y, background_size.x);
            gizmos.line_2d(from_pos, to_pos, Color::WHITE);
        }
    }
}

impl Plugin for UiLinePlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<LineGizmos>()
            .init_resource::<ActivelyDrawingLine>()
            .add_event::<SpawnLineEvent>()
            .add_event::<DeleteLine>()
            .add_event::<TempConnectLine>()
            .add_event::<ConnectLine>()
            .add_systems(Startup, Self::configure_line)
            .add_systems(OnEnter(DragState::Started), Self::spawn_new_line)
            .add_systems(
                Update,
                (
                    Self::handle_spawn_line,
                    Self::handle_delete_line,
                    Self::handle_temp_connect_line,
                ),
            )
            .add_systems(OnExit(DragState::Started), Self::handle_mouse_release)
            .add_systems(Last, Self::draw_line);
    }
}
