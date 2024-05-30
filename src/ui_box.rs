use bevy::{
    input::{common_conditions::input_just_pressed, keyboard::KeyboardInput, ButtonState},
    prelude::*,
    ui::FocusPolicy,
};

use crate::{
    connectors::{ConnectionDirection, Connector, SpawnConnector},
    focus::{ActiveEntity, DragEntity, DragState, FocusBundle, FocusColor},
    utils::{BlockType, ConnectionType, Position, Size},
    DeleteEvent,
};

#[derive(Component, Clone, Copy)]
pub struct UIBox;

#[derive(Component, Clone, Copy)]
pub struct BackgroundBox;

#[derive(Bundle)]
struct BackgroundBoxBundle {
    marker: (BackgroundBox, UIBox),
    node: NodeBundle,
    label: crate::Label,
    focus_bundle: FocusBundle,
}

impl BackgroundBoxBundle {
    fn new() -> Self {
        Self {
            marker: (BackgroundBox, UIBox),
            label: crate::Label("Background Box".into()),
            node: NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    ..default()
                },
                focus_policy: bevy::ui::FocusPolicy::Block,
                background_color: BackgroundColor(Color::rgba(0., 0., 0., 0.)),
                ..default()
            },
            focus_bundle: FocusBundle::new(Color::RED, Color::GREEN, Color::WHITE),
            // focus: Focus::new(Color::RED, Color::WHITE, Color::GREEN),
            // focus_color: FocusColor(Color::WHITE),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct Block;

#[derive(Bundle, Clone)]
struct BlockBundle {
    marker: (Block, UIBox),
    position: Position,
    size: Size,
    block_type: BlockType,
    node: NodeBundle,
    label: crate::Label,
    focus_bundle: FocusBundle,
}

impl BlockBundle {
    fn new(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        focus_bundle: FocusBundle,
        block_type: BlockType,
    ) -> Self {
        Self {
            marker: (Block, UIBox),
            block_type,
            position: Position(Vec2::new(x, y)),
            size: Size(Vec2::new(w, h)),
            label: crate::Label("Block".into()),
            node: NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(y),
                    left: Val::Px(x),
                    min_width: Val::Px(w),
                    flex_direction: FlexDirection::Column,
                    min_height: Val::Px(h),
                    border: UiRect::all(Val::Px(1.)),
                    padding: UiRect::all(Val::Px(8.)),
                    ..default()
                },
                background_color: BackgroundColor(Color::rgba(0., 0., 0., 0.)),
                border_color: BorderColor(Color::WHITE),
                focus_policy: bevy::ui::FocusPolicy::Block,
                ..default()
            },
            focus_bundle,
        }
    }
}

#[derive(Component)]
struct HoleContainer;
#[derive(Bundle)]
pub struct HoleContainerBundle {
    node: NodeBundle,
    hole_container: HoleContainer,
}

impl HoleContainerBundle {
    fn new() -> Self {
        Self {
            hole_container: HoleContainer,
            node: NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.),
                    padding: UiRect::all(Val::Px(10.)),
                    margin: UiRect::all(Val::Px(10.)),
                    justify_content: JustifyContent::SpaceEvenly,
                    align_items: AlignItems::Center,

                    ..default()
                },
                focus_policy: bevy::ui::FocusPolicy::Pass,
                ..default()
            },
        }
    }
}

#[derive(Component)]
struct Hole;

#[derive(Bundle)]
struct HoleBundle {
    node: NodeBundle,
    hole: Hole,
    label: crate::Label,
    focus_bundle: FocusBundle,
}

impl HoleBundle {
    fn new() -> Self {
        Self {
            label: crate::Label("Hole".into()),
            hole: Hole,
            node: NodeBundle {
                style: Style {
                    padding: UiRect::all(Val::Px(10.)),
                    border: UiRect::all(Val::Px(1.)),
                    min_width: Val::Px(30.),

                    ..default()
                },
                focus_policy: bevy::ui::FocusPolicy::Block,
                border_color: BorderColor(Color::WHITE),
                ..default()
            },

            focus_bundle: FocusBundle::new(Color::WHITE, Color::GREEN, Color::WHITE),
        }
    }
}

#[derive(Debug, Component, Clone)]
struct Arg;

#[derive(Debug, Bundle, Clone)]
struct ArgBundle {
    arg: Arg,
    block_type: BlockType,
    node: NodeBundle,
    focus_bundle: FocusBundle,
}

impl ArgBundle {
    fn new(block_type: BlockType) -> Self {
        Self {
            block_type,
            arg: Arg,
            focus_bundle: FocusBundle::new(Color::RED, Color::GREEN, Color::WHITE),
            node: NodeBundle {
                style: Style {
                    border: UiRect::all(Val::Px(1.)),
                    padding: UiRect::all(Val::Px(8.)),
                    ..default()
                },
                border_color: BorderColor(Color::WHITE),
                ..default()
            },
        }
    }
}

#[derive(Event, Clone, Debug)]
struct SpawnArg {
    bundle: ArgBundle,
    hole: Entity,
}

#[derive(Event, Clone)]
struct SpawnUIBox {
    bundle: BlockBundle,
    parent: Option<Entity>,
    // connections: [Option<ConnectionType>; 3],
}

/// This is a plugin for the UI Box element. It contains all the systems for the UI Box
pub struct UIBoxPlugin;

impl UIBoxPlugin {
    fn spawn_background_box(mut commands: Commands) {
        let bundle = BackgroundBoxBundle::new();
        commands.spawn(bundle);
    }

    fn spawn_box(
        mut keyboard_events: EventReader<KeyboardInput>,
        mut writer: EventWriter<SpawnUIBox>,
        background: Query<&Node, With<BackgroundBox>>,
    ) {
        let background = (background.single().size() / 2.) - 50.;
        for keyboard_event in keyboard_events.read() {
            if keyboard_event.state == ButtonState::Pressed {
                #[allow(clippy::single_match)]
                let block_type = match keyboard_event.key_code {
                    KeyCode::KeyS => BlockType::Declaration,
                    KeyCode::KeyD => BlockType::If,
                    KeyCode::KeyC => BlockType::Comparison,
                    _ => continue,
                };
                writer.send(SpawnUIBox {
                    parent: None,
                    bundle: BlockBundle::new(
                        background.x,
                        background.y,
                        40.,
                        40.,
                        FocusBundle::new(Color::RED, Color::GREEN, Color::WHITE),
                        block_type,
                    ),
                });
            }
        }
    }

    fn spawn_initial_box(mut writer: EventWriter<SpawnUIBox>) {
        writer.send(SpawnUIBox {
            parent: None,
            bundle: BlockBundle::new(
                0.,
                0.,
                40.,
                40.,
                FocusBundle::new(Color::RED, Color::GREEN, Color::WHITE),
                BlockType::Start,
            ),
            // connections: Default::default(),
        });
    }

    fn handle_spawn_ui_box(
        mut reader: EventReader<SpawnUIBox>,
        mut connector_writer: EventWriter<SpawnConnector>,
        mut commands: Commands,
        background: Query<Entity, With<BackgroundBox>>,
    ) {
        for SpawnUIBox {
            parent,
            bundle,
            // connections,
        } in reader.read().map(ToOwned::to_owned)
        {
            let mut container = if let Some(parent) = parent {
                commands
                    .get_entity(parent)
                    .expect("Expected parent to be in the world tree")
            } else {
                let Some(background) = commands.get_entity(background.single()) else {
                    info!("Background box was not found in the world tree");
                    return;
                };
                background
            };
            container.with_children(|parent_commands| {
                let text = bundle.block_type.to_string();
                let holes = bundle.block_type.get_holes();
                let connections = bundle.block_type.get_connectors();
                let mut ui_box = parent_commands.spawn(bundle);
                ui_box.with_children(|parent| {
                    // Spawn Text
                    parent.spawn((
                        TextBundle::from(text).with_text_justify(JustifyText::Left),
                        Label,
                    ));

                    if holes > 0 {
                        // Spawn Hole Container
                        let mut hole_container = parent.spawn(HoleContainerBundle::new());
                        hole_container.with_children(|parent| {
                            for _ in 0..holes {
                                parent.spawn(HoleBundle::new());
                            }
                        });
                    }
                });
                if parent.is_none() {
                    for index in 0..connections {
                        let Some(direction) = ConnectionDirection::get_direction(index) else {
                            error!("Direction index {index} is not handled");
                            continue;
                        };
                        connector_writer.send(SpawnConnector {
                            connector: Connector {
                                fixture: ui_box.id(),
                                direction,
                                connection_type: ConnectionType::Flow,
                                connected: false,
                            },
                            radius: 10.,
                        });
                    }
                }
            });
        }
    }

    fn handle_holes(mut reader: EventReader<SpawnArg>, mut commands: Commands) {}

    fn update_size(mut query: Query<(&mut Size, &Node), With<Block>>) {
        for (mut box_size, box_node) in &mut query {
            box_size.0 = box_node.size();
        }
    }

    fn translate_position(mut query: Query<(&Position, &mut Style), With<UIBox>>) {
        for (box_pos, mut box_style) in &mut query {
            box_style.top = Val::Px(box_pos.0.y);
            box_style.left = Val::Px(box_pos.0.x);
        }
    }

    fn handle_color_change(
        query: Query<(Entity, &FocusColor), (Changed<FocusColor>, Or<(With<UIBox>, With<Hole>)>)>,
        mut style_query: Query<&mut BorderColor, Or<(With<UIBox>, With<Hole>)>>,
    ) {
        for (entity, focus_color) in &query {
            let Ok(mut border_color) = style_query.get_mut(entity) else {
                error!("Couldn't find the color");
                continue;
            };
            border_color.0 = focus_color.0;
        }
    }

    fn move_according_to_keyboard(
        active: Res<ActiveEntity>,
        mut boxes: Query<&mut Position, With<Block>>,
        mut keyboard_events: EventReader<KeyboardInput>,
    ) {
        if let Some(active) = active.entity.filter(|&entity| boxes.get(entity).is_ok()) {
            let mut position = boxes
                .get_mut(active)
                .expect("Expected the active box to be in the world tree ");
            for keyboard_event in keyboard_events.read() {
                let vector = match keyboard_event.key_code {
                    KeyCode::ArrowUp => Vec2::NEG_Y,
                    KeyCode::ArrowDown => Vec2::Y,
                    KeyCode::ArrowLeft => Vec2::NEG_X,
                    KeyCode::ArrowRight => Vec2::X,
                    _ => Vec2::ZERO,
                };
                position.0 += vector * 10.;
            }
        }
    }

    fn move_active_box_according_to_mouse(
        mut mouse_motion_event: EventReader<CursorMoved>,
        active: Res<ActiveEntity>,
        mut boxes: Query<&mut Position, With<Block>>,
    ) {
        if let Some(active) = active.entity.filter(|&entity| boxes.get(entity).is_ok()) {
            for delta in mouse_motion_event
                .read()
                .map(|motion| motion.delta.unwrap_or_default())
            {
                let mut pos = boxes
                    .get_mut(active)
                    .expect("Expected box to be in the world tree");
                pos.0 += delta;
            }
        }
    }

    fn delete_block(
        active: Res<ActiveEntity>,
        boxes: Query<&Block>,
        mut writer: EventWriter<DeleteEvent>,
    ) {
        // If the active box is a block
        if let Some(active) = active.entity.filter(|&entity| boxes.get(entity).is_ok()) {
            writer.send(DeleteEvent(active));
        }
    }

    fn make_focus_passable(
        drag_entity: Res<DragEntity>,
        mut focus_block: Query<&mut FocusPolicy, With<Block>>,
    ) {
        if let Some(entity) = drag_entity.entity {
            let Ok(mut policy) = focus_block.get_mut(entity) else {
                return;
            };
            *policy = FocusPolicy::Pass;
        }
    }
    fn make_focus_unpassable(
        drag_entity: Res<DragEntity>,
        mut focus_block: Query<&mut FocusPolicy, With<Block>>,
    ) {
        if let Some(entity) = drag_entity.entity {
            let Ok(mut policy) = focus_block.get_mut(entity) else {
                return;
            };
            *policy = FocusPolicy::Block;
        } else {
            error!("There's no drag entity no more");
        }
    }

    fn handle_hole_hovered_on_top(
        mut delete_writer: EventWriter<DeleteEvent>,
        mut spawn_writer: EventWriter<SpawnUIBox>,
    ) {
    }
}

impl Plugin for UIBoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnUIBox>()
            .add_systems(
                Startup,
                (Self::spawn_background_box, Self::spawn_initial_box),
            )
            .add_systems(OnEnter(DragState::Started), Self::make_focus_passable)
            .add_systems(OnExit(DragState::Started), Self::make_focus_unpassable)
            .add_systems(
                Update,
                (
                    Self::handle_spawn_ui_box,
                    // Self::handle_color_change.run_if(in_state(DragState::Ended)),
                    Self::handle_color_change,
                    Self::move_active_box_according_to_mouse.run_if(in_state(DragState::Started)),
                    Self::move_according_to_keyboard,
                    Self::spawn_box,
                    Self::translate_position,
                    Self::update_size,
                    Self::delete_block.run_if(input_just_pressed(KeyCode::Backspace)),
                )
                    .chain(),
            );
    }
}
