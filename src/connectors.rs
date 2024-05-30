use bevy::{math::bounding::BoundingVolume, prelude::*};

use crate::{
    focus::{ActiveEntity, DragEntity, DragState, FocusBundle, FocusColor},
    ui_box::Block,
    ui_line::{ActivelyDrawingLine, ConnectLine, TempConnectLine, UiLine},
    utils::{get_aabb2d, get_relative_direction, ConnectionType, Position, Size},
};

#[derive(Debug, Clone, Copy, Default)]
pub enum ConnectionDirection {
    Left,
    Right,
    #[allow(dead_code)]
    Top,
    #[default]
    Bottom,
    Center,
}

impl ConnectionDirection {
    pub const fn get_direction(index: usize) -> Option<Self> {
        let dir = match index {
            0 => ConnectionDirection::Bottom,
            1 => ConnectionDirection::Left,
            2 => ConnectionDirection::Right,
            _ => {
                return None;
            }
        };
        Some(dir)
    }

    pub const fn get_top(&self) -> f32 {
        match self {
            ConnectionDirection::Left
            | ConnectionDirection::Right
            | ConnectionDirection::Center => 50.,
            ConnectionDirection::Bottom => 100.,
            _ => 0.,
        }
    }

    pub const fn get_left(&self) -> f32 {
        match self {
            ConnectionDirection::Top
            | ConnectionDirection::Bottom
            | ConnectionDirection::Center => 50.,
            ConnectionDirection::Right => 100.,
            _ => 0.,
        }
    }
}

#[derive(Bundle)]
pub struct ConnectorBundle {
    connector: Connector,
    node: NodeBundle,
    size: Size,
    label: crate::Label,
    position: Position,
    focus_bundle: FocusBundle,
}

#[derive(Debug, Resource, Default)]
pub struct CollidedRect {
    pub entity: Option<Entity>,
}

impl ConnectorBundle {
    fn new(connector: Connector, radius: f32, position: Position) -> Self {
        let top = connector.direction.get_top();
        let left = connector.direction.get_left();

        // let offset = match connector.direction {
        //     ConnectionDirection::Top | ConnectionDirection::Left => 1.,
        //     ConnectionDirection::Right | ConnectionDirection::Bottom => -1.,
        //     _ => 0.,
        // };

        Self {
            focus_bundle: FocusBundle::new(Color::RED, Color::GREEN, Color::WHITE),
            connector,
            position,
            label: crate::Label("Connector".into()),
            size: Size(Vec2::new(radius, radius)),
            node: NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(top),
                    left: Val::Percent(left),
                    width: Val::Px(radius),
                    height: Val::Px(radius),
                    ..default()
                },
                background_color: BackgroundColor(Color::WHITE),
                focus_policy: bevy::ui::FocusPolicy::Block,
                ..default()
            },
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Connector {
    pub fixture: Entity,
    pub direction: ConnectionDirection,
    pub connection_type: ConnectionType,
    pub connected: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SpawnConnector {
    pub connector: Connector,
    pub radius: f32,
}

pub struct ConnectorPlugin;

impl ConnectorPlugin {
    fn handle_spawn_connector(
        mut reader: EventReader<SpawnConnector>,
        mut commands: Commands,
        query: Query<(&Position, &Size)>,
    ) {
        for &SpawnConnector { connector, radius } in reader.read() {
            let (parent_pos, &parent_size) = query
                .get(connector.fixture)
                .expect("Couldn't find the fixture's position");
            // let Ok((parent_pos, &parent_size)) = query.get(connector.fixture) else {
            //     error!("Couldn't find the fixture's position");
            //     continue;
            // };
            let mut size = parent_size.0;
            size.x *= connector.direction.get_left() / 100.;
            size.y *= connector.direction.get_top() / 100.;

            let new_pos = Position(parent_pos.0 + size);

            let bundle = ConnectorBundle::new(connector, radius, new_pos);

            let Some(mut parent) = commands.get_entity(connector.fixture) else {
                error!("The parent doesn't exist in the world tree");
                continue;
            };

            parent.with_children(|parent| {
                parent.spawn(bundle);
            });
        }
    }

    fn handle_color_change(
        query: Query<(Entity, &FocusColor), (Changed<FocusColor>, With<Connector>)>,
        mut color_query: Query<&mut BackgroundColor, With<Connector>>,
    ) {
        for (entity, focus_color) in &query {
            let Ok(mut background) = color_query.get_mut(entity) else {
                continue;
            };
            background.0 = focus_color.0;
        }
    }

    fn move_connector_according_to_mouse(
        active: Res<DragEntity>,
        mut query: Query<&mut GlobalTransform, With<Connector>>,
        mut mouse_motions: EventReader<CursorMoved>,
    ) {
        if let Some(active_entity) = active.entity.filter(|&entity| query.get(entity).is_ok()) {
            let mut global = query.get_mut(active_entity).unwrap();
            for delta in mouse_motions
                .read()
                .map(|motion| motion.delta.unwrap_or_default())
            {
                let transform = Transform::default().with_translation(delta.extend(0.));
                *global = global.mul_transform(transform);
            }
        }
    }

    fn translate_position(
        mut query: Query<
            (&mut Position, &GlobalTransform),
            (With<Connector>, Changed<GlobalTransform>),
        >,
    ) {
        for (mut pos, transform) in &mut query {
            pos.0 = transform.translation().xy();
        }
    }

    fn reset_position(
        active: Res<ActiveEntity>,
        mut connectors: Query<(&Connector, &Node, &mut GlobalTransform)>,
        styles: Query<(&Position, &Node), Without<Connector>>,
    ) {
        if let Some((connector, connector_node, mut global_connector)) = active
            .entity
            .map(|entity| connectors.get_mut(entity).ok())
            .flatten()
        {
            let (pos, node) = styles
                .get(connector.fixture)
                .expect("Expected parent to be in the world tree");
            let mut size = node.size();

            size.x *= connector.direction.get_left() / 100.;
            size.y *= connector.direction.get_top() / 100.;
            let target = pos.0 + size;
            let translation = target.extend(0.) - global_connector.translation();
            *global_connector =
                global_connector.mul_transform(Transform::default().with_translation(
                    translation + (connector_node.size().extend(0.) - Vec3::new(3., 3., 0.)),
                ));
            // if active.
        }
    }

    fn set_connect_line(
        mut reader: EventReader<TempConnectLine>,
        mut collided_rect: ResMut<CollidedRect>,
    ) {
        for &TempConnectLine(collided_entity) in reader.read() {
            collided_rect.entity = collided_entity.map(|(e, _)| e);
        }
    }

    fn check_collision(
        connectors: Query<(&Position, &Size, &Connector), (With<Connector>, Changed<Position>)>,
        positions: Query<(Entity, &Position, &Size), With<Block>>,
        collided_rect: Res<CollidedRect>,
        mut writer: EventWriter<TempConnectLine>,
    ) {
        for (connector_pos, connector_size, connector) in &connectors {
            let connector_aabb = get_aabb2d(connector_pos, connector_size);

            let mut collided_with = positions.iter().filter_map(|(entity, pos, size)| {
                if entity == connector.fixture {
                    return None;
                }
                let target_aab = get_aabb2d(pos, size);
                if target_aab.contains(&connector_aabb) {
                    let direction =
                        get_relative_direction((pos, size), (connector_pos, connector_size));
                    Some((entity, direction))
                } else {
                    None
                }
            });

            //? This could in theory product two boxes but we don't care
            if let Some(collided_with) = collided_with.next() {
                if collided_rect.entity.is_none()
                    || collided_rect
                        .entity
                        .is_some_and(|entity| entity != collided_with.0)
                {
                    writer.send(TempConnectLine(Some(collided_with)));
                }
            } else {
                writer.send_default();
            }
        }
    }

    fn hide_connector(
        mut reader: EventReader<ConnectLine>,
        mut connectors: Query<&mut Visibility, With<Connector>>,
        active: Res<ActiveEntity>,
    ) {
        for ConnectLine(_) in reader.read() {
            let Ok(mut visibility) = connectors.get_mut(
                active
                    .entity
                    .expect("Expected the active element to be a connector"),
            ) else {
                return;
            };

            *visibility = Visibility::Hidden;
        }
    }
}

impl Plugin for ConnectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnConnector>()
            .init_resource::<CollidedRect>()
            .add_systems(
                Update,
                (
                    Self::handle_color_change,
                    Self::move_connector_according_to_mouse,
                    Self::translate_position,
                    Self::check_collision.run_if(in_state(DragState::Started)),
                    Self::set_connect_line,
                    Self::hide_connector,
                ),
            )
            .add_systems(PostUpdate, Self::handle_spawn_connector)
            .add_systems(OnExit(DragState::Started), Self::reset_position);
    }
}
