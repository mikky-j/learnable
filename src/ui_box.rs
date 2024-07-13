use bevy::{
    input::{common_conditions::input_just_pressed, keyboard::KeyboardInput, ButtonState},
    prelude::*,
    ui::FocusPolicy,
};
use bevy_simple_text_input::{TextInputBundle, TextInputPlugin};

use crate::{
    ast::{AddToAst, RemoveFromAst, UpdateAst},
    connectors::{Connector, SpawnConnector},
    focus::{
        ActiveEntity, DragEntity, DragState, Draggable, FocusColor, HoverEntity,
        InteractionFocusBundle,
    },
    text_input::CustomTextInputBundle,
    utils::{BlockType, HoleType, Language, Position, Size},
    wasm::{Message, WASMRequest},
    DeleteEvent, EntityLabel, ErrorEvent, GameSets,
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
pub struct BlockBundle {
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
    pub fn new(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        focus_bundle: InteractionFocusBundle,
        block_type: BlockType,
    ) -> Self {
        let color = block_type.concept_type.get_color();
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
                    // min_height: Val::Px(h),
                    border: UiRect::all(Val::Px(1.)),
                    padding: UiRect::all(Val::Px(4.)),
                    ..default()
                },
                background_color: BackgroundColor(color),
                border_color: BorderColor(Color::rgba_u8(0, 0, 0, 0)),
                focus_policy: bevy::ui::FocusPolicy::Block,
                ..default()
            },
            focus_bundle,
        }
    }
}

#[derive(Debug, Component)]
pub struct ErrorBox;

#[derive(Bundle)]
pub struct ErrorBoxBundle {
    node: TextBundle,
    marker: (UIBox, ErrorBox),
}

impl ErrorBoxBundle {
    pub fn new(error: String) -> Self {
        Self {
            node: TextBundle {
                text: Text::from_section(
                    error,
                    TextStyle {
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                style: Style {
                    position_type: PositionType::Absolute,
                    right: Val::Px(0.),
                    top: Val::Px(0.),
                    padding: UiRect::all(Val::Px(8.)),
                    ..default()
                },
                background_color: Color::RED.into(),
                ..default()
            },
            marker: (UIBox, ErrorBox),
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
    pub hole_type: HoleType,
}

#[derive(Bundle)]
struct HoleBundle {
    node: NodeBundle,
    hole: Hole,
    label: EntityLabel,
    focus_bundle: InteractionFocusBundle,
}

impl HoleBundle {
    fn new(owner: Entity, order: usize, hole_type: HoleType) -> Self {
        Self {
            label: EntityLabel::new("Hole"),
            hole: Hole {
                owner,
                order,
                hole_type,
            },
            node: NodeBundle {
                style: Style {
                    padding: UiRect::all(Val::Px(4.)),
                    border: UiRect::all(Val::Px(1.)),
                    min_width: Val::Px(10.),
                    ..default()
                },
                focus_policy: bevy::ui::FocusPolicy::Block,
                border_color: BorderColor(Color::BLACK),
                ..default()
            },

            focus_bundle: InteractionFocusBundle::new(Color::BLACK, Color::GREEN, Color::BLACK),
        }
    }
}

#[derive(Debug, Component, Clone)]
pub struct Arg {
    pub owner: Entity,
    pub order: usize,
}

#[derive(Event, Debug, Clone)]
pub struct SpawnUIBox {
    pub bundle: BlockBundle,
    pub marker: Option<crate::Marker>, // connections: [Option<ConnectionType>; 3],
}

#[derive(Event, Debug, Clone, Copy)]
pub struct SpawnArg {
    pub arg: Entity,
    pub parent: Entity,
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
        language: Res<Language>,
        mut error_writer: EventWriter<ErrorEvent>,
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
                        KeyCode::KeyS => "Declaration",
                        KeyCode::KeyD => "If",
                        KeyCode::KeyC => "Comparitor",
                        KeyCode::KeyT => "Text",
                        KeyCode::KeyV => "Variable",
                        KeyCode::KeyB => "Print",
                        _ => continue,
                    };
                    let Some(block_type) = language.get_block(block_type) else {
                        error_writer.send(ErrorEvent(format!(
                            "Couldn't spawn {block_type} for language"
                        )));
                        continue;
                    };
                    writer.send(SpawnUIBox {
                        bundle: BlockBundle::new(
                            background_size.x,
                            background_size.y,
                            50.,
                            60.,
                            InteractionFocusBundle::default(),
                            block_type,
                        ),
                        marker: None,
                    });
                }
            }
        }
    }

    fn spawn_initial_box(mut writer: EventWriter<SpawnUIBox>, language: Res<Language>) {
        let start_block = language.get_block("Start").unwrap();

        writer.send(SpawnUIBox {
            marker: None,
            bundle: BlockBundle::new(
                0.,
                0.,
                60.,
                60.,
                InteractionFocusBundle::default(),
                start_block,
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
            bundle,
            marker,
            // connections,
        } in reader.read().map(ToOwned::to_owned)
        {
            let mut container = commands
                .get_entity(background.single())
                .expect("Should never fail");

            container.with_children(|parent_commands| {
                let text = bundle.block_type.to_string();
                let holes = bundle.block_type.get_holes();
                let block_type = bundle.block_type.clone();
                let connections = block_type.connectors.clone();

                let mut ui_box = parent_commands.spawn(bundle);
                if let Some(marker) = marker {
                    ui_box.insert(marker);
                }

                let ui_box_id = ui_box.id();

                add_ast_writer.send(AddToAst {
                    parent: None,
                    child: (ui_box_id, block_type.clone()),
                });

                ui_box.with_children(|parent| {
                    // Spawn Text
                    parent.spawn((
                        TextBundle::from_section(
                            text,
                            TextStyle {
                                color: Color::BLACK,
                                font_size: 20.,
                                ..default()
                            },
                        )
                        .with_text_justify(JustifyText::Left),
                        Label,
                    ));

                    if holes > 0 {
                        // Spawn Hole Container
                        let mut hole_container = parent.spawn(HoleContainerBundle::new());
                        hole_container.with_children(|parent| {
                            match block_type {
                                block_type if block_type.has_text() => {
                                    let text_bundle =
                                        TextInputBundle::default().with_text_style(TextStyle {
                                            color: Color::BLACK,
                                            font_size: 15.,
                                            ..default()
                                        });
                                    parent
                                        .spawn(CustomTextInputBundle::new(text_bundle, ui_box_id));
                                }
                                _ => {
                                    for (order, hole_type) in
                                        block_type.holes.into_iter().enumerate()
                                    {
                                        parent
                                            .spawn(HoleBundle::new(ui_box_id, order, hole_type))
                                            .with_children(|parent| {
                                                parent.spawn(
                                                    TextBundle::from_section(
                                                        order.to_string(),
                                                        TextStyle {
                                                            color: Color::BLACK,
                                                            font_size: 15.,
                                                            ..Default::default()
                                                        },
                                                    )
                                                    .with_text_justify(JustifyText::Center),
                                                );
                                            });
                                    }
                                }
                            };
                        });
                    }
                });

                for direction in connections {
                    connector_writer.send(SpawnConnector {
                        connector: Connector {
                            fixture: ui_box.id(),
                            direction,
                            // connection_type: ConnectionType::Flow,
                            connected: false,
                        },
                        radius: 7.,
                    });
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
        mut query: Query<(&mut Position, &GlobalTransform), (With<Arg>, Changed<GlobalTransform>)>,
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
        mut focus_block: Query<
            (&mut FocusPolicy, &mut BackgroundColor, &mut Transform),
            With<Block>,
        >,
    ) {
        if let Some(entity) = drag_entity.entity {
            let Ok((mut policy, mut background_color, mut transform)) = focus_block.get_mut(entity)
            else {
                return;
            };
            background_color.0 = background_color.0.with_a(0.5);
            transform.translation.z = 100.;

            *policy = FocusPolicy::Pass;
        }
    }
    fn make_focus_unpassable(
        drag_entity: Res<DragEntity>,
        mut focus_block: Query<
            (&mut FocusPolicy, &mut BackgroundColor, &mut Transform),
            With<Block>,
        >,
    ) {
        if let Some(entity) = drag_entity.entity {
            let Ok((mut policy, mut background, mut transform)) = focus_block.get_mut(entity)
            else {
                return;
            };
            background.0 = background.0.with_a(1.);
            transform.translation.z = 100.;
            *policy = FocusPolicy::Block;
        } else {
            error!("There's no drag entity no more");
        }
    }

    fn handle_spawn_active_arg(
        mut arg_reader: EventReader<SpawnArg>,
        mut commands: Commands,
        mut update_writer: EventWriter<UpdateAst>,
        children: Query<&Children>,
        connectors: Query<&Connector>,
        mut style: Query<&mut Style>,
        hole: Query<&Hole>,
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
                .insert(Arg {
                    owner: event.parent,
                    order: hole.get(event.parent).unwrap().order,
                })
                .set_parent(event.parent)
                .remove_children(children_connectors.as_slice());

            for child in children_connectors {
                commands.entity(child).despawn_recursive();
            }
            update_writer.send(UpdateAst);
        }
    }

    fn handle_hover_on_hole(
        drag_entity: Res<DragEntity>,
        hover_entity: Res<HoverEntity>,
        hole_query: Query<(Entity, &Hole)>,
        arg_query: Query<&Arg>,
        boxes: Query<(Entity, &BlockType), With<Block>>,
        mut arg_writer: EventWriter<SpawnArg>,
        mut error_writer: EventWriter<ErrorEvent>,
    ) {
        if let Some((drag_entity, block_type)) = drag_entity
            .entity
            .and_then(|entity| {
                boxes
                    .get(entity)
                    .ok()
                    .filter(|(_, block_type)| block_type.can_be_in_a_hole())
            })
            .filter(|&(entity, _)| arg_query.get(entity).is_err())
        {
            if let Some((hover_entity, hole)) = hover_entity
                .entity
                .and_then(|entity| hole_query.get(entity).ok())
                .filter(|(_, hole)| hole.owner != drag_entity)
            {
                let block_type_value = &block_type.value;
                let hole_type_value = &hole.hole_type;
                if hole_type_value == &HoleType::Any || block_type_value == hole_type_value {
                    arg_writer.send(SpawnArg {
                        arg: drag_entity,
                        parent: hover_entity,
                    });
                } else {
                    error_writer.send(ErrorEvent(
                        format!("Block with type {block_type_value:?} was dropped in a hole that expected {hole_type_value:?}")
                    ));
                }
            }
        }
    }

    fn move_arg_according_to_mouse(
        curr_drag: Res<DragEntity>,
        mut arg_query: Query<&mut GlobalTransform, With<Arg>>,
        mut mouse_motions: EventReader<CursorMoved>,
    ) {
        if let Some(mut transform) = curr_drag
            .entity
            .and_then(|entity| arg_query.get_mut(entity).ok())
        {
            for delta in mouse_motions
                .read()
                .map(|motion| motion.delta.unwrap_or_default())
            {
                let new_transform = Transform::default().with_translation(delta.extend(0.));
                *transform = transform.mul_transform(new_transform);
            }
        }
    }

    fn handle_outside_hole(
        curr_drag: Res<DragEntity>,
        hover_entity: Res<HoverEntity>,
        background: Query<&BackgroundBox>,
        mut args: Query<(Entity, &GlobalTransform, &mut Style), With<Arg>>,
        mut commands: Commands,
        mut update_writer: EventWriter<UpdateAst>,
    ) {
        if let Some(hover_entity) = hover_entity
            .entity
            .filter(|&entity| background.get(entity).is_ok())
        {
            if let Some((entity, global_transform, mut styles)) = curr_drag
                .entity
                .and_then(|entity| args.get_mut(entity).ok())
            {
                commands.entity(entity).remove::<Arg>();

                let mut background = commands.entity(hover_entity);

                // INFO: Set the variable to be absolute and get the current global position and set it
                // the top and left to that position

                let curr_pos = global_transform.translation().xy();
                styles.position_type = PositionType::Absolute;
                styles.top = Val::Px(curr_pos.y);
                styles.left = Val::Px(curr_pos.x);

                background.push_children(&[entity]);
            }
        }
        update_writer.send_default();
    }

    fn print_block_type(active: Res<ActiveEntity>, blocks: Query<&BlockType>) {
        if let Some(active) = active.entity {
            if let Ok(block_type) = blocks.get(active) {
                info!("{block_type:?}");
            }
        }
    }

    fn send_language_list(language: Res<Language>, mut socket_writer: EventWriter<WASMRequest>) {
        let list = language.get_lang_data();
        socket_writer.send(WASMRequest(Message::LanguageList(list)));
    }
}

impl Plugin for UIBoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnUIBox>()
            .add_event::<SpawnArg>()
            .insert_resource(Language::new())
            .add_systems(
                Startup,
                (
                    Self::spawn_background_box,
                    Self::spawn_initial_box,
                    Self::send_language_list,
                ),
            )
            .add_systems(OnEnter(DragState::Started), Self::make_focus_passable)
            .add_systems(
                OnExit(DragState::Started),
                (
                    Self::handle_hover_on_hole,
                    Self::handle_spawn_active_arg,
                    Self::handle_outside_hole,
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
                        Self::move_according_to_keyboard,
                        Self::move_arg_according_to_mouse.run_if(in_state(DragState::Started)),
                        // Self::spawn_box,
                        Self::translate_position,
                        Self::translate_position_args,
                        Self::update_size,
                        Self::print_block_type.run_if(input_just_pressed(KeyCode::KeyH)),
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
