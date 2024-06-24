use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    ast::{AddToAst, RemoveFromAst},
    connectors::{ConnectionDirection, Connector},
    focus::{DragEntity, DragState, FocusColor, LineFocusBundle},
    translate_vec_to_world,
    ui_box::{Arg, BackgroundBox, Block},
    utils::{BlockType, Position, Size},
    DeleteEvent, GameSets,
};

#[derive(Component, Debug, Clone, Copy)]
pub struct Segment {
    pub from: Vec2,
    pub to: Vec2,
    pub owner: Entity,
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
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
    focus_bundle: LineFocusBundle,
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
            focus_bundle: LineFocusBundle::new(Color::RED, Color::GREEN, Color::WHITE),
        }
    }
}

#[derive(Debug, Bundle)]
pub struct SegmentBundle {
    segment: Segment,
    node: NodeBundle,
}

impl SegmentBundle {
    pub fn _new(segment: Segment) -> Self {
        let height = segment.from.distance(segment.to);
        let _angle = segment.from.angle_between(segment.to);
        Self {
            segment,
            node: NodeBundle {
                style: Style {
                    width: Val::Px(1.),
                    height: Val::Px(height),
                    ..default()
                },
                ..default()
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

#[derive(Debug, Event, Clone)]
pub struct SpawnSegments(pub Entity, pub Vec<Segment>);

pub struct UiLinePlugin;

// TODO: Implement focusable for lines
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
        if active_drawing.entity.is_none() {
            if let Some((curr_drag_entity, connector)) = curr_drag
                .entity
                .and_then(|entity| connectors.get(entity).ok())
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
        mut add_to_ast_writer: EventWriter<AddToAst>,

        // TODO: Once implemented the specialized AST event, remove this ASAP
        block_type: Query<&BlockType, Without<Arg>>,
    ) {
        if let Some(entity) = active_drawing.entity {
            let line = lines
                .get(entity)
                .expect("Expected the line to be in the world tree");
            if connectors.get(line.to).is_ok() {
                delete_writer.send(DeleteLine(entity));
            } else {
                connect_writer.send(ConnectLine(Some((line.to, line.to_direction))));
                let block_type = block_type.get(line.to).unwrap();
                add_to_ast_writer.send(AddToAst {
                    parent: Some((line.from, line.from_direction.get_parse_order())),
                    child: (line.to, block_type.to_owned()),
                });
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
        mut connectors: Query<(&Connector, &mut Visibility), With<Connector>>,
        mut active: ResMut<ActivelyDrawingLine>,
    ) {
        for &DeleteLine(deleted_entity) in reader.read() {
            let Some(entity_commands) = commands.get_entity(deleted_entity) else {
                error!("The deleted line has already been deleted");
                continue;
            };

            // INFO: Make connector visible if it has not despawned yet
            let line = lines
                .get(deleted_entity)
                .expect("Expected the line to be deleted");

            if let Ok((_, children)) = parent.get(line.from) {
                for &child in children {
                    if let Ok((connector, mut connector_visibility)) = connectors.get_mut(child) {
                        if connector.direction == line.from_direction {
                            *connector_visibility = Visibility::Visible;
                        }
                    }
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

    fn handle_segments(mut reader: EventReader<SpawnSegments>, mut commands: Commands) {
        for SpawnSegments(line_entity, segments) in reader.read().map(ToOwned::to_owned) {
            let mut line = commands.entity(line_entity);
            line.despawn_descendants();
            line.clear_children();
            line.with_children(|parent| {
                for segment in segments.into_iter() {
                    parent.spawn(segment);
                }
            });
        }
    }

    fn make_segments(
        mut writer: EventWriter<SpawnSegments>,
        lines: Query<(Entity, &UiLine)>,
        query: Query<(&Position, &Size)>,
        changed_query: Query<(&Position, &Size), Or<(Changed<Position>, Changed<Size>)>>,
    ) {
        for (entity, line) in &lines {
            let ((Position(from_pos), Size(from_size)), (Position(to_pos), Size(to_size))) =
                if let Ok((&from_entity_pos, &from_entity_size)) = changed_query.get(line.from) {
                    let (&to_pos, &to_size) = match changed_query.get(line.to).ok() {
                        Some(to) => to,
                        None => query.get(line.to).unwrap(),
                    };
                    ((from_entity_pos, from_entity_size), (to_pos, to_size))
                } else if let Ok((&to_entity_pos, &to_entity_size)) = changed_query.get(line.to) {
                    let (&from_pos, &from_size) = query.get(line.from).unwrap();
                    ((from_pos, from_size), (to_entity_pos, to_entity_size))
                } else {
                    continue;
                };

            let segment_from_pos = from_pos + (from_size * (line.from_direction.get_vec() / 100.));
            let segment_to_pos = to_pos + (to_size * (line.to_direction.get_vec() / 100.));

            writer.send(SpawnSegments(
                entity,
                vec![Segment {
                    from: segment_from_pos,
                    to: segment_to_pos,
                    owner: entity,
                }],
            ));
        }
    }

    fn draw_line(
        lines: Query<(&UiLine, &FocusColor, &Children)>,
        segments: Query<&Segment>,
        mut gizmos: Gizmos<LineGizmos>,
        background: Query<&Node, With<BackgroundBox>>,
    ) {
        let background_size = background.single().size();
        for (_, focus_color, children) in &lines {
            for &child in children {
                let Ok(segment) = segments.get(child) else {
                    continue;
                };
                let segment_from =
                    translate_vec_to_world(segment.from, background_size.y, background_size.x);
                let segment_to =
                    translate_vec_to_world(segment.to, background_size.y, background_size.x);

                gizmos.line_2d(segment_from, segment_to, focus_color.0);
            }
        }
    }

    fn handle_connected_delete(
        lines: Query<(Entity, &UiLine)>,
        mut delete_reader: EventReader<DeleteEvent>,
        mut delete_line_writer: EventWriter<DeleteLine>,
        mut remove_from_ast_writer: EventWriter<RemoveFromAst>,
    ) {
        for &DeleteEvent(deleted_entity) in delete_reader.read() {
            for (line_entity, line) in lines
                .iter()
                .filter(|(_, line)| line.from == deleted_entity || line.to == deleted_entity)
            {
                remove_from_ast_writer.send(RemoveFromAst {
                    parent: Some((line.from, line.from_direction.get_parse_order())),
                    child: line.to,
                });

                delete_line_writer.send(DeleteLine(line_entity));
            }
        }
    }

    fn draw_debug_make_segements(
        background: Query<&Node, With<BackgroundBox>>,
        mut gizmos: Gizmos<LineGizmos>,
        query: Query<(&Position, &Size)>,
        lines: Query<&UiLine>,
    ) {
        let background_size = background.single().size();
        for line in &lines {
            let (&Position(from_pos), &Size(from_size)) =
                query.get(line.from).expect("Expected to be in world tree");
            let (&Position(to_pos), &Size(to_size)) = query
                .get(line.to)
                .expect("Expected to be in the world tree");

            let from_pos = translate_vec_to_world(from_pos, background_size.y, background_size.x)
                + from_size * Vec2::new(0.5, -0.5);
            let to_pos = translate_vec_to_world(to_pos, background_size.y, background_size.x)
                + to_size * Vec2::new(0.5, -0.5);
            let from_size = from_size + 40.;
            let to_size = to_size + 40.;

            let from_connection_point =
                from_pos + from_size * line.from_direction.get_center_vec() / 100.;
            let to_connection_point = to_pos + to_size * line.to_direction.get_center_vec() / 100.;

            // Step 1
            // Outer boundary rect
            gizmos.rect_2d(from_pos, 0., from_size, Color::ALICE_BLUE);
            gizmos.rect_2d(to_pos, 0., to_size, Color::ORANGE_RED);
            gizmos.circle_2d(from_connection_point, 5., Color::ALICE_BLUE);
            gizmos.circle_2d(to_connection_point, 5., Color::ORANGE_RED);

            // Step 2
            // Draw grid lines
            // let grid
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
            .add_event::<SpawnSegments>()
            .add_systems(Startup, Self::configure_line)
            .add_systems(OnEnter(DragState::Started), Self::spawn_new_line)
            .add_systems(
                Update,
                (
                    (
                        Self::handle_spawn_line,
                        Self::handle_delete_line,
                        Self::handle_temp_connect_line,
                        Self::make_segments,
                        Self::handle_segments,
                        Self::draw_line,
                    )
                        .chain()
                        .in_set(GameSets::Running),
                    Self::handle_connected_delete.in_set(GameSets::Despawn),
                ),
            )
            .add_systems(OnExit(DragState::Started), Self::handle_mouse_release);
        // .add_systems(Last, Self::draw_debug_make_segements);
    }
}
