#![allow(clippy::type_complexity)]

// TODO:
// - Fix focus to allow focusing on lines
// - Fix `ui_box` to allow the dragging outside of arguments

mod ast;
mod camera;
mod connectors;
mod focus;
// mod function;
mod text_input;
mod ui_box;
mod ui_line;
mod utils;

use std::fs;

use bevy::{
    app::PluginGroupBuilder, input::common_conditions::input_just_pressed, prelude::*,
    utils::HashMap, window::PresentMode,
};
use bevy_simple_text_input::TextInputValue;
use serde::{Deserialize, Serialize};
use ui_line::UiLinePlugin;

use crate::{
    ast::{AddToAst, Ast, BlockData, BlockDataMap},
    camera::CameraPlugin,
    focus::FocusPlugin,
    text_input::{CustomTextInputPlugin, TextInput},
    ui_box::{
        Arg, BackgroundBox, BlockBundle, ErrorBox, ErrorBoxBundle, Hole, SpawnArg, SpawnUIBox,
        UIBoxPlugin,
    },
    ui_line::UiLine,
    utils::{BlockType, Position, Size},
};
use ast::ASTPlugin;
use connectors::ConnectorPlugin;

pub const WINDOW_HEIGHT: f32 = 600.;
pub const WINDOW_WIDTH: f32 = 600.;

pub const WHITE: Color = Color::rgb(255., 255., 255.);
pub const RED: Color = Color::rgb(255., 0., 0.);

// pub fn input_just_pressed_with_modifier<T>(
//     input: T,
// ) -> impl FnMut(Res<ButtonInput<T>>) -> bool + Clone
// where
//     T: Copy + Eq + std::hash::Hash + Send + Sync + 'static,
// {
//     move |inputs: Res<ButtonInput<T>>| inputs.get_just_pressed(KeyCode::ShiftLeft)
// }

pub fn get_default_plugins() -> PluginGroupBuilder {
    DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Learnable test".into(),
            canvas: Some("#game-canvas".into()),
            resolution: if cfg!(target_os = "web") {
                default()
            } else {
                (WINDOW_WIDTH, WINDOW_HEIGHT).into()
            },
            present_mode: PresentMode::AutoVsync,
            visible: true,
            ..default()
        }),
        ..default()
    })
}

pub fn translate_vec_to_world(mut vector: Vec2, window_height: f32, window_width: f32) -> Vec2 {
    vector.x -= window_width / 2.;
    vector.y = window_height / 2. - vector.y;
    vector
}

pub fn box_pos_collision(subject: Vec2, (target_pos, target_size): (Vec2, Vec2)) -> bool {
    subject.x >= target_pos.x
        && subject.x <= (target_pos.x + target_size.x)
        && subject.y >= target_pos.y
        && subject.y <= (target_pos.y + target_size.y)
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Default)]
pub struct EntityLabel(pub String);

impl EntityLabel {
    pub fn new(data: impl Into<String>) -> Self {
        Self(data.into())
    }
}

#[derive(SystemSet, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
pub enum GameSets {
    Running,
    Despawn,
}

pub struct GamePlugin;

#[derive(Debug, Event, Clone, Copy)]
pub struct DeleteEvent(pub Entity);

#[derive(Debug, Event, Clone)]
pub struct ErrorEvent(pub String);

#[derive(Debug, Component, Clone, Copy)]
pub struct Marker(pub Entity);

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct State {
    parent: Option<Entity>,
    order: Option<usize>,
    connections: [Option<(Entity, BlockType)>; 3],
    holes: Vec<BlockData>,
    block_type: BlockType,
    position: Position,
    size: Size,
    value: Option<String>,
}

#[derive(Resource, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GameState {
    map: HashMap<Entity, State>,
    lines: Vec<UiLine>,
}

impl GamePlugin {
    // Setup functions
    fn _spawn_camera(mut commands: Commands) {
        commands.spawn_empty().insert(Camera2dBundle::default());
    }

    fn handle_delete_block(mut reader: EventReader<DeleteEvent>, mut commands: Commands) {
        for &DeleteEvent(block) in reader.read() {
            let Some(commands) = commands.get_entity(block) else {
                error!("Cannot get the commands");
                continue;
            };
            commands.despawn_recursive();
        }
    }

    fn handle_errors(
        mut reader: EventReader<ErrorEvent>,
        mut commands: Commands,
        prev_error_message: Query<Entity, With<ErrorBox>>,
        background: Query<Entity, With<BackgroundBox>>,
    ) {
        for event in reader.read().map(ToOwned::to_owned) {
            if let Ok(previous) = prev_error_message.get_single() {
                commands.entity(previous).despawn_recursive();
            }

            let Some(mut command) = commands.get_entity(background.single()) else {
                error!("There was more than one background entity");
                continue;
            };
            command.with_children(|parent| {
                parent.spawn(ErrorBoxBundle::new(event.0));
            });
        }
    }

    fn store_state(
        mut game_state: ResMut<GameState>,
        text_value: Query<(&TextInput, &TextInputValue)>,
        block_query: Query<(Entity, &Position, &Size, &BlockType)>,
        arg_query: Query<&Arg>,
        ast: Res<Ast>,
        block_map: Res<BlockDataMap>,
        lines: Query<&UiLine>,
    ) {
        let mut app_state: HashMap<Entity, State> = HashMap::default();
        for (entity, &position, &size, block_type) in &block_query {
            let mut parent = None;
            let mut order = None;
            if let Ok(arg) = arg_query.get(entity) {
                parent = Some(arg.owner);
                order = Some(arg.order);
            }
            let connections = ast
                .map
                .get(&entity)
                .map(ToOwned::to_owned)
                .unwrap_or_default();

            let holes = block_map
                .map
                .get(&entity)
                .map(ToOwned::to_owned)
                .unwrap_or_default();

            let value = if block_type.name == "Text" {
                text_value.iter().find_map(|(text_input, text_value)| {
                    if text_input.owner == entity {
                        Some(text_value.0.clone())
                    } else {
                        None
                    }
                })
            } else {
                None
            };
            let state = State {
                parent,
                order,
                connections,
                holes,
                block_type: block_type.to_owned(),
                position,
                size,
                value,
            };

            app_state.insert(entity, state);
        }
        game_state.set_if_neq(GameState {
            map: app_state,
            lines: lines.into_iter().map(ToOwned::to_owned).collect(),
        });
        let text = serde_json::to_string(game_state.into_inner()).unwrap();
        fs::write("state.json", text).unwrap();
    }

    fn load_state(
        mut game_state: ResMut<GameState>,
        mut commands: Commands,
        background: Query<Entity, With<BackgroundBox>>,
        block_children: Query<&Children>,
        block_type: Query<&BlockType>,
    ) {
        let value: GameState =
            serde_json::from_str(fs::read_to_string("state.json").unwrap().as_str()).unwrap();
        game_state.set_if_neq(value);

        let background_entity = background.single();

        let children = block_children.get(background_entity).unwrap();

        for &child in children {
            if block_type.contains(child) {
                commands.entity(child).despawn_recursive();
            }
        }
    }

    fn spawn_entities_from_state(
        game_state: Res<GameState>,
        mut box_writer: EventWriter<SpawnUIBox>,
    ) {
        for (&entity, state) in &game_state.map {
            let spawn_box = SpawnUIBox {
                bundle: BlockBundle::new(
                    state.position.x(),
                    state.position.y(),
                    state.size.width(),
                    state.size.height(),
                    Default::default(),
                    state.block_type.clone(),
                ),
                marker: Some(Marker(entity)),
            };
            box_writer.send(spawn_box);
        }
    }

    fn load_entities(
        game_state: Res<GameState>,
        mut arg_writer: EventWriter<SpawnArg>,
        mut ast_writer: EventWriter<AddToAst>,
        markers: Query<(Entity, &Marker)>,
        holes: Query<(Entity, &Hole)>,
        mut text_value: Query<(&TextInput, &mut TextInputValue)>,
        mut commands: Commands,
    ) {
        let markers: HashMap<Entity, Entity> = markers
            .iter()
            .map(|(entity, &Marker(new_entity))| (entity, new_entity))
            .collect();

        // Spawn all lines again
        for line in &game_state.lines {
            let new_line = UiLine {
                from: markers.get(&line.from).unwrap().to_owned(),
                to_direction: line.to_direction,
                to: markers.get(&line.to).unwrap().to_owned(),
                from_direction: line.from_direction,
            };

            let block_type = game_state.map.get(&line.to).unwrap().block_type.clone();

            ast_writer.send(AddToAst {
                parent: Some((new_line.from, new_line.from_direction.get_parse_order())),
                child: (new_line.to, block_type),
            });

            commands.spawn(new_line);
        }

        // Spawn all args and load all text back into the block
        for (entity, state) in game_state
            .map
            .iter()
            .filter(|(_, state)| state.parent.is_some())
        {
            let &new_parent = state
                .parent
                .map(|parent| markers.get(&parent).unwrap())
                .unwrap();

            let &child_entity = markers.get(entity).unwrap();

            if state.block_type.name == "Text" {
                let mut text_input = text_value
                    .iter_mut()
                    .find_map(|(text_input, text_value)| {
                        if text_input.owner == child_entity {
                            Some(text_value)
                        } else {
                            None
                        }
                    })
                    .expect("Couldn't get the text");
                text_input.0 = state.value.clone().unwrap_or_default();
            }

            let mut holes = holes
                .iter()
                .filter(|(_, hole)| hole.owner == new_parent)
                .collect::<Vec<_>>();

            holes.sort_by(|(_, x), (_, other)| x.order.cmp(&other.order));

            if let Some(order) = state.order {
                let (hole, _) = holes[order];
                arg_writer.send(SpawnArg {
                    arg: child_entity,
                    parent: hole,
                });
            }
        }
    }
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameState>()
            .configure_sets(Update, (GameSets::Despawn, GameSets::Running).chain())
            .add_systems(
                Update,
                apply_deferred
                    .before(GameSets::Running)
                    .after(GameSets::Despawn),
            )
            .add_systems(
                Update,
                (
                    Self::handle_delete_block.in_set(GameSets::Despawn),
                    // Self::store_state.run_if(input_just_pressed(KeyCode::KeyZ)),
                    // Self::load_state.run_if(input_just_pressed(KeyCode::KeyL)),
                    // (
                    //     Self::spawn_entities_from_state,
                    //     apply_deferred,
                    //     Self::load_entities,
                    //     apply_deferred,
                    // )
                    //     .chain()
                    //     .run_if(input_just_pressed(KeyCode::KeyI)),
                ),
            )
            .add_systems(Last, Self::handle_errors)
            .add_event::<DeleteEvent>()
            .add_event::<ErrorEvent>()
            // .add_plugins(BoxPlugin)
            .add_plugins(FocusPlugin)
            .add_plugins(UiLinePlugin)
            // .add_plugins(LinePlugin)
            .add_plugins(ASTPlugin)
            .add_plugins(UIBoxPlugin)
            .add_plugins(CustomTextInputPlugin)
            .add_plugins(CameraPlugin)
            .add_plugins(ConnectorPlugin);
    }
}
