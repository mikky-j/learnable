use bevy::{math::bounding::BoundingVolume, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{
    focus::{ActiveEntity, DragEntity, DragState, Draggable, FocusColor, InteractionFocusBundle},
    ui_box::Block,
    ui_line::{ConnectLine, TempConnectLine},
    utils::{get_aabb2d, get_relative_direction, Position, Size},
    GameSets,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
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

    pub const fn get_parse_order(&self) -> usize {
        match self {
            ConnectionDirection::Left => 0,
            ConnectionDirection::Right => 1,
            ConnectionDirection::Bottom => 2,
            ConnectionDirection::Top => 3,
            ConnectionDirection::Center => 4,
        }
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

    pub const fn get_vec(&self) -> Vec2 {
        Vec2::new(self.get_left(), self.get_top())
    }

    pub const fn get_center_top(&self) -> f32 {
        match self {
            ConnectionDirection::Left => 0.,
            ConnectionDirection::Right => 0.,
            ConnectionDirection::Top => 50.,
            ConnectionDirection::Bottom => -50.,
            ConnectionDirection::Center => 0.,
        }
    }

    pub const fn get_center_left(&self) -> f32 {
        match self {
            ConnectionDirection::Left => -50.,
            ConnectionDirection::Right => 50.,
            ConnectionDirection::Top => 0.,
            ConnectionDirection::Bottom => 0.,
            ConnectionDirection::Center => 0.,
        }
    }

    pub const fn get_center_vec(&self) -> Vec2 {
        Vec2::new(self.get_center_left(), self.get_center_top())
    }
}

#[derive(Bundle)]
pub struct ConnectorBundle {
    connector: Connector,
    node: NodeBundle,
    size: Size,
    label: crate::EntityLabel,
    position: Position,
    focus_bundle: InteractionFocusBundle,
    draggable: Draggable,
}

#[derive(Debug, Resource, Default)]
pub struct CollidedRect {
    pub entity: Option<Entity>,
}

impl ConnectorBundle {
    fn new(connector: Connector, radius: f32, position: Position) -> Self {
        let top = connector.direction.get_top();
        let left = connector.direction.get_left();

        Self {
            focus_bundle: InteractionFocusBundle::new(Color::RED, Color::GREEN, Color::WHITE),
            connector,
            position,
            draggable: Draggable,
            label: crate::EntityLabel("Connector".into()),
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
                transform: Transform::default()
                    .with_translation(Vec2::new(-radius, -radius).extend(0.) / 2.),
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
    // pub connection_type: ConnectionType,
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
        active: Res<DragEntity>,
        mut connectors: Query<&mut GlobalTransform, With<Connector>>,
    ) {
        if let Some(mut global_connector) = active
            .entity
            .and_then(|entity| connectors.get_mut(entity).ok())
        {
            info!("I ran in reset");
            let old_pos = active.drag_start.unwrap().extend(0.);
            let new_pos = old_pos - global_connector.translation();

            *global_connector =
                global_connector.mul_transform(Transform::default().with_translation(new_pos));
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
                )
                    .in_set(GameSets::Running),
            )
            .add_systems(PostUpdate, Self::handle_spawn_connector)
            .add_systems(OnExit(DragState::Started), Self::reset_position);
    }
}
