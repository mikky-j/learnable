use bevy::{
    input::{common_conditions::input_just_pressed, keyboard::KeyboardInput, ButtonState},
    prelude::*,
    ui::FocusPolicy,
};
use bevy_simple_text_input::{TextInputBundle, TextInputPlugin};

use crate::{
    ast::{AddToAst, RemoveFromAst},
    connectors::{ConnectionDirection, Connector, SpawnConnector},
    focus::{
        ActiveEntity, DragEntity, DragState, Draggable, FocusColor, HoverEntity,
        InteractionFocusBundle,
    },
    text_input::CustomTextInputBundle,
    utils::{BlockType, Language, Position, Size},
    DeleteEvent, EntityLabel, GameSets,
};

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct UIBox;

#[derive(Component, Clone, Copy)]
pub struct BackgroundBox;

#[derive(Bundle)]
struct BackgroundBoxBundle {
    marker: (BackgroundBox, UIBox),
    node: NodeBundle,
    label: EntityLabel,
    focus_bundle: InteractionFocusBundle,
}

impl BackgroundBoxBundle {
    fn new() -> Self {
        Self {
            marker: (BackgroundBox, UIBox),
            label: EntityLabel("Background Box".into()),
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
            focus_bundle: InteractionFocusBundle::new(Color::RED, Color::GREEN, Color::WHITE),
        }
    }
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Block;

#[derive(Bundle, Debug, Clone, Default)]
struct BlockBundle {
    marker: (Block, UIBox),
    position: Position,
    size: Size,
    block_type: BlockType,
    node: NodeBundle,
    label: EntityLabel,
    focus_bundle: InteractionFocusBundle,
    draggable: Draggable,
}

impl BlockBundle {
    fn _new_with_parent(
        size: Size,
        focus_bundle: InteractionFocusBundle,
        block_type: BlockType,
    ) -> Self {
        Self {
            node: NodeBundle {
                style: Style {
                    min_width: Val::Px(size.width()),
                    flex_direction: FlexDirection::Column,
                    min_height: Val::Px(size.height()),
                    border: UiRect::all(Val::Px(1.)),
                    padding: UiRect::all(Val::Px(4.)),
                    ..default()
                },
                background_color: BackgroundColor(Color::rgba(0., 0., 0., 0.)),
                border_color: BorderColor(Color::WHITE),
                focus_policy: bevy::ui::FocusPolicy::Block,
                ..default()
            },
            size,
            focus_bundle,
            block_type,
            label: EntityLabel::new("Block"),
            ..default()
        }
    }

    fn new(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        focus_bundle: InteractionFocusBundle,
        block_type: BlockType,
    ) -> Self {
        Self {
            draggable: Draggable,
            marker: (Block, UIBox),
            block_type,
            position: Position(Vec2::new(x, y)),
            size: Size(Vec2::new(w, h)),
            label: EntityLabel("Block".into()),
            node: NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(y),
                    left: Val::Px(x),
                    min_width: Val::Px(w),
                    flex_direction: FlexDirection::Column,
                    min_height: Val::Px(h),
                    border: UiRect::all(Val::Px(1.)),
                    padding: UiRect::all(Val::Px(4.)),
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
                    column_gap: Val::Px(4.),
                    padding: UiRect::all(Val::Px(4.)),
                    margin: UiRect::all(Val::Px(4.)),
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
pub struct Hole {
    pub owner: Entity,
    pub order: usize,
}

#[derive(Bundle)]
struct HoleBundle {
    node: NodeBundle,
    hole: Hole,
    label: EntityLabel,
    focus_bundle: InteractionFocusBundle,
}

impl HoleBundle {
    fn new(owner: Entity, order: usize) -> Self {
        Self {
            label: EntityLabel::new("Hole"),
            hole: Hole { owner, order },
            node: NodeBundle {
                style: Style {
                    padding: UiRect::all(Val::Px(4.)),
                    border: UiRect::all(Val::Px(1.)),
                    min_width: Val::Px(10.),
                    ..default()
                },
                focus_policy: bevy::ui::FocusPolicy::Block,
                border_color: BorderColor(Color::WHITE),
                ..default()
            },

            focus_bundle: InteractionFocusBundle::new(Color::WHITE, Color::GREEN, Color::WHITE),
        }
    }
}

#[derive(Debug, Component, Clone)]
struct Arg;

#[derive(Event, Debug, Clone)]
struct SpawnUIBox {
    bundle: BlockBundle,
    parent: Option<Entity>,
    // connections: [Option<ConnectionType>; 3],
}

#[derive(Event, Debug, Clone, Copy)]
struct SpawnArg {
    arg: Entity,
    parent: Entity,
}

// #[derive(Resource, Debug, Default)]
// pub struct ActiveArgSpawn {
//     spawn_arg: Option<SpawnUIBox>,
//     owner_block: Option<Entity>,
// }

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
        background: Query<(Entity, &Node), With<BackgroundBox>>,
        active: Res<ActiveEntity>,
    ) {
        let (background_entity, background) = background.single();
        // if the active entity is the background entity then we can spawn a box
        if active
            .entity
            .is_some_and(|entity| entity == background_entity)
        {
            let background_size = (background.size() / 2.) - 50.;
            for keyboard_event in keyboard_events.read() {
                if keyboard_event.state == ButtonState::Pressed {
                    let block_type = match keyboard_event.key_code {
                        KeyCode::KeyS => BlockType::Declaration,
                        KeyCode::KeyD => BlockType::If,
                        KeyCode::KeyC => BlockType::Comparison,
                        KeyCode::KeyT => BlockType::Text,
                        KeyCode::KeyV => BlockType::Variable,
                        KeyCode::KeyB => BlockType::Print,
                        _ => continue,
                    };
                    writer.send(SpawnUIBox {
                        parent: None,
                        bundle: BlockBundle::new(
                            background_size.x,
                            background_size.y,
                            40.,
                            40.,
                            InteractionFocusBundle::new(Color::RED, Color::GREEN, Color::WHITE),
                            block_type,
                        ),
                    });
                }
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
                InteractionFocusBundle::new(Color::RED, Color::GREEN, Color::WHITE),
                BlockType::Start,
            ),
        });
    }

    fn handle_spawn_ui_box(
        mut reader: EventReader<SpawnUIBox>,
        mut connector_writer: EventWriter<SpawnConnector>,
        mut add_ast_writer: EventWriter<AddToAst>,
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
                let mut commands = commands
                    .get_entity(parent)
                    .expect("Expected parent to be in the world tree");
                commands.despawn_descendants();
                commands
            } else {
                let background = commands
                    .get_entity(background.single())
                    .expect("Should never fail");
                background
            };

            container.with_children(|parent_commands| {
                let block_type = bundle.block_type;
                let text = bundle.block_type.to_string();
                let holes = bundle.block_type.get_holes();
                let connections = bundle.block_type.get_connectors();

                let mut ui_box = if parent.is_some() {
                    parent_commands.spawn((bundle, Arg))
                } else {
                    parent_commands.spawn(bundle)
                };

                let ui_box_id = ui_box.id();

                add_ast_writer.send(AddToAst {
                    parent: None,
                    child: (ui_box_id, block_type),
                });

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
                            match block_type {
                                BlockType::Text => {
                                    let text_bundle = TextInputBundle::default();
                                    parent
                                        .spawn(CustomTextInputBundle::new(text_bundle, ui_box_id));
                                }
                                _ => {
                                    for order in 0..holes {
                                        parent
                                            .spawn(HoleBundle::new(ui_box_id, order))
                                            .with_children(|parent| {
                                                parent.spawn(
                                                    TextBundle::from(order.to_string())
                                                        .with_text_justify(JustifyText::Center),
                                                );
                                            });
                                    }
                                }
                            };
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
                                // connection_type: ConnectionType::Flow,
                                connected: false,
                            },
                            radius: 10.,
                        });
                    }
                }
            });
        }
    }

    fn update_size(mut query: Query<(&mut Size, &Node), With<Block>>) {
        for (mut box_size, box_node) in &mut query {
            box_size.0 = box_node.size();
        }
    }

    fn translate_position(
        mut query: Query<(&Position, &mut Style), (With<UIBox>, Changed<Position>)>,
    ) {
        for (box_pos, mut box_style) in &mut query {
            if box_style.position_type == PositionType::Absolute {
                box_style.top = Val::Px(box_pos.0.y);
                box_style.left = Val::Px(box_pos.0.x);
            }
        }
    }

    fn translate_position_args(
        mut query: Query<(&mut Position, &GlobalTransform), (With<Arg>, Changed<Position>)>,
    ) {
        for (mut position, transform) in &mut query {
            position.0 = transform.translation().xy();
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
        active: Res<DragEntity>,
        mut boxes: Query<&mut Position, (Without<Arg>, With<Block>)>,
    ) {
        if let Some(mut pos) = active.entity.and_then(|entity| boxes.get_mut(entity).ok()) {
            for delta in mouse_motion_event
                .read()
                .map(|motion| motion.delta.unwrap_or_default())
            {
                pos.0 += delta;
            }
        }
    }

    fn _move_arg_according_to_mouse(
        mut mouse_motion_event: EventReader<CursorMoved>,
        active: Res<DragEntity>,
        mut boxes: Query<&mut Transform, With<Arg>>,
    ) {
        if let Some(mut global_transform) =
            active.entity.and_then(|entity| boxes.get_mut(entity).ok())
        {
            for delta in mouse_motion_event
                .read()
                .map(|motion| motion.delta.unwrap_or_default())
            {
                let new_transform = Transform::default().with_translation(delta.extend(0.));
                *global_transform = global_transform.mul_transform(new_transform);
            }
        }
    }

    fn delete_block(
        active: Res<ActiveEntity>,
        boxes: Query<&Block>,
        mut delete_writer: EventWriter<DeleteEvent>,
        mut remove_ast_writer: EventWriter<RemoveFromAst>,
    ) {
        // If the active box is a block
        if let Some(active) = active.entity.filter(|&entity| boxes.get(entity).is_ok()) {
            delete_writer.send(DeleteEvent(active));
            remove_ast_writer.send(RemoveFromAst {
                parent: None,
                child: active,
            });
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
            info!("Made focus unpassable");
            *policy = FocusPolicy::Block;
        } else {
            error!("There's no drag entity no more");
        }
    }

    fn handle_spawn_active_arg(
        mut arg_reader: EventReader<SpawnArg>,
        mut commands: Commands,
        children: Query<&Children>,
        connectors: Query<&Connector>,
        mut style: Query<&mut Style>,
    ) {
        for event in arg_reader.read() {
            info!("Running the spawning of args");
            let Ok(mut style) = style.get_mut(event.arg) else {
                info!("Couldn't get the style for the argument");
                continue;
            };
            style.position_type = PositionType::Relative;
            style.top = Val::Px(0.);
            style.left = Val::Px(0.);

            let Some(mut parent_commands) = commands.get_entity(event.parent) else {
                info!("Couldn't get the commands for the parent");
                continue;
            };
            parent_commands.despawn_descendants();

            let Some(mut arg_commands) = commands.get_entity(event.arg) else {
                info!("Couldn't get the commands for the arguments");
                continue;
            };

            let children_connectors = children
                .get(event.arg)
                .unwrap()
                .iter()
                .filter(|&&x| connectors.get(x).is_ok())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>();

            // INFO: Add argument, set the parent to the new parent and then remove all the
            // connectors from it

            arg_commands
                .insert(Arg)
                .set_parent(event.parent)
                .remove_children(children_connectors.as_slice());

            for child in children_connectors {
                commands.entity(child).despawn_recursive();
            }
        }
    }

    fn _handle_outside_hole(
        drag_entity: Res<DragEntity>,
        hover_entity: Res<HoverEntity>,
        background: Query<&BackgroundBox>,
        mut position: Query<
            (Entity, &BlockType, &Size, &Position, &mut GlobalTransform),
            With<Arg>,
        >,
        mut delete_writer: EventWriter<DeleteEvent>,
        mut spawn_writer: EventWriter<SpawnUIBox>,
    ) {
        if let Some((entity, &drag_block_type, size, position, mut global_transform)) = drag_entity
            .entity
            .and_then(|entity| position.get_mut(entity).ok())
        {
            if hover_entity
                .entity
                .is_some_and(|entity| background.get(entity).is_ok())
            {
                delete_writer.send(DeleteEvent(entity));
                info!("Deleted the entity");
                let bundle = BlockBundle::new(
                    position.x(),
                    position.y(),
                    size.width(),
                    size.height(),
                    InteractionFocusBundle::default(),
                    drag_block_type,
                );
                spawn_writer.send(SpawnUIBox {
                    bundle,
                    parent: None,
                });
            } else {
                let old_pos = drag_entity.drag_start.unwrap().extend(0.);
                let new_transform =
                    Transform::default().with_translation(position.0.extend(0.) - old_pos);
                *global_transform = global_transform.mul_transform(new_transform);
            }
        }
    }

    fn handle_hover_on_hole(
        drag_entity: Res<DragEntity>,
        hover_entity: Res<HoverEntity>,
        hole_query: Query<(Entity, &Hole)>,
        arg_query: Query<&Arg>,
        boxes: Query<&BlockType, With<Block>>,
        mut arg_writer: EventWriter<SpawnArg>,
    ) {
        if let Some(drag_entity) = drag_entity
            .entity
            .filter(|&entity| {
                boxes
                    .get(entity)
                    .is_ok_and(|block_type| block_type.can_be_in_a_hole())
            })
            // If it is not already an arg
            .filter(|&entity| arg_query.get(entity).is_err())
        {
            if let Some(hover_entity) = hover_entity
                .entity
                .and_then(|entity| hole_query.get(entity).ok())
                .filter(|(_, hole)| hole.owner != drag_entity)
                .map(|(entity, _)| entity)
            {
                arg_writer.send(SpawnArg {
                    arg: drag_entity,
                    parent: hover_entity,
                });
            }
        }
    }
    fn get_language() {
        info!("{:#?}", Language::new())
    }
}

impl Plugin for UIBoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnUIBox>()
            .add_event::<SpawnArg>()
            .add_systems(
                Startup,
                (
                    Self::spawn_background_box,
                    Self::spawn_initial_box,
                    Self::get_language,
                ),
            )
            .add_systems(OnEnter(DragState::Started), Self::make_focus_passable)
            .add_systems(
                OnExit(DragState::Started),
                (
                    Self::handle_hover_on_hole,
                    Self::handle_spawn_active_arg,
                    // Self::handle_outside_hole,
                    Self::make_focus_unpassable,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    (
                        Self::handle_spawn_ui_box,
                        Self::handle_color_change,
                        Self::move_active_box_according_to_mouse
                            .run_if(in_state(DragState::Started)),
                        // Self::move_arg_according_to_mouse.run_if(in_state(DragState::Started)),
                        Self::move_according_to_keyboard,
                        Self::spawn_box,
                        Self::translate_position,
                        Self::translate_position_args,
                        Self::update_size,
                    )
                        .chain()
                        .in_set(GameSets::Running),
                    Self::delete_block
                        .run_if(
                            input_just_pressed(KeyCode::Backspace)
                                .or_else(input_just_pressed(KeyCode::Delete)),
                        )
                        .in_set(GameSets::Despawn),
                ),
            )
            .add_plugins(TextInputPlugin);
    }
}
