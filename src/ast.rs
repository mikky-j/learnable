use std::fmt::Display;

use bevy::{
    input::common_conditions::{input_just_pressed, input_just_released},
    prelude::*,
    utils::HashMap,
};
use bevy_simple_text_input::TextInputValue;

use crate::{
    focus::DragState,
    text_input::TextInput,
    ui_box::Hole,
    utils::{BlockType, ConnectionType},
    GameSets,
};

#[derive(Debug)]
pub enum BlockData {
    Hole(Entity, usize),
    Value(String),
}

#[derive(Debug, Resource, Default)]
pub struct BlockDataMap {
    pub map: HashMap<Entity, Vec<BlockData>>,
}

fn get_block_data_hashmap(
    holes: Query<(Entity, &Hole)>,
    children: Query<&Children>,
    block_type: Query<(Entity, &BlockType)>,
    text_input: Query<(&TextInput, &TextInputValue)>,
    mut block_map: ResMut<BlockDataMap>,
) {
    let mut hashmap: HashMap<Entity, Vec<BlockData>> = HashMap::default();
    for (hole_entity, hole) in &holes {
        let value = hashmap.entry(hole.owner).or_default();
        let Some((child_entity, child_block)) =
            children.get(hole_entity).ok().and_then(|children| {
                children
                    .iter()
                    .find_map(|&child| block_type.get(child).ok())
            })
        else {
            info!("Hole {hole_entity:?} children were not block_types");
            continue;
        };

        match child_block {
            BlockType::Text => {
                let Some((_, text_value)) = text_input
                    .iter()
                    .find(|(text_input, _)| text_input.owner == child_entity)
                else {
                    info!("Entity {child_entity:?} had a BlockType::Text but no TextInputValue");
                    continue;
                };
                value.push(BlockData::Value(text_value.0.clone()));
            }
            _ => value.push(BlockData::Hole(child_entity, hole.order)),
        }
    }

    let block_key = hashmap
        .keys()
        .filter_map(|&key| block_type.get(key).map(|(_, block_type)| block_type).ok())
        .collect::<Vec<_>>();

    info!("Block types of keys: {:?}", block_key);
    info!("Hashmap: {hashmap:#?}");

    block_map.map = hashmap;
}

#[derive(Resource, Debug, Default)]
pub struct Ast {
    // pub map: HashMap<Entity, [Option<Entity>; 3]>,
    pub map: HashMap<Entity, Vec<Option<Entity>>>,
}

#[derive(Debug)]
pub struct Traverse {
    entity: Entity,

    flow: Option<Box<Traverse>>,
    left: Option<Box<Traverse>>,
    right: Option<Box<Traverse>>,
}

impl Traverse {
    const fn get_default(entity: Entity) -> Self {
        Self {
            entity,
            flow: None,
            left: None,
            right: None,
        }
    }

    // fn print(&self, boxes: &Query<(Entity, &BlockType), With<crate::r#box::Box>>) -> String {
    //     let mut result = String::with_capacity(256);
    //
    //     let (_, start_block_type) = boxes
    //         .get(self.entity)
    //         .expect("Expected box to be in the tree");
    //
    //     result.push_str(format!("( {start_block_type} ").as_str());
    //
    //     if let Some(left) = &self.left {
    //         result.push_str(left.print(boxes).as_str());
    //     }
    //
    //     if let Some(right) = &self.right {
    //         result.push_str(right.print(boxes).as_str());
    //     }
    //
    //     result.push_str(") ");
    //
    //     if let Some(flow) = &self.flow {
    //         result.push_str(" -> ");
    //         result.push_str(flow.print(boxes).as_str());
    //     }
    //     result
    // }
    //
    fn set_val(&mut self, connection_type: ConnectionType, traverse: Self) {
        match connection_type {
            ConnectionType::Flow => self.flow = Some(Box::new(traverse)),
            ConnectionType::Left => self.left = Some(Box::new(traverse)),
            ConnectionType::Right => self.right = Some(Box::new(traverse)),
        }
    }
}

impl Ast {
    pub fn traverse_tree(&self, start: Entity) -> Traverse {
        let mut result = Traverse::get_default(start);

        // Assume that all entities are in the tree
        let start_d = self.map.get(&start).unwrap();

        for (index, &entity) in start_d.iter().enumerate() {
            if let Some(entity) = entity {
                result.set_val(
                    ConnectionType::from_usize(index).expect("Index was not a connection type"),
                    self.traverse_tree(entity),
                );
            }
        }

        result
    }
}

#[derive(Event, Debug, Clone, Copy)]
pub struct AddToASTEvent {
    pub parent: Option<Entity>,
    pub child: Entity,
    pub connection_type: ConnectionType,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct RemoveFromAST {
    pub parent: Option<Entity>,
    pub child: Option<Entity>,
    pub connection_type: ConnectionType,
}

pub struct ASTPlugin;

impl ASTPlugin {
    fn handle_add_to_ast(mut global_ast: ResMut<Ast>, mut add_event: EventReader<AddToASTEvent>) {
        for &event in add_event.read() {
            // Add the parent to the AST if it doesn't exist and say this is it's child
            if let Some(parent) = event.parent {
                let parent = global_ast.map.entry(parent).or_default();
                parent[event.connection_type as usize] = Some(event.child);
                // parent.push(events.child);
            }
            // Add the child to the AST if it doesn't exist
            global_ast.map.entry(event.child).or_default();
        }
    }

    fn handle_remove_from_ast(
        mut global_ast: ResMut<Ast>,
        mut remove_event: EventReader<RemoveFromAST>,
    ) {
        for &event in remove_event.read() {
            // Delete the child from the AST
            if let Some(child) = event.child {
                global_ast.map.remove(&child);
            }

            // Delete the child from the parent
            if let Some(parent) = event.parent {
                let parent_connections = global_ast.map.entry(parent).or_default();
                parent_connections[event.connection_type as usize] = None;
            }
        }
    }

    // fn print_ast(
    //     global_ast: Res<Ast>,
    //     boxes: Query<(Entity, &BlockType), With<crate::r#box::Box>>,
    // ) {
    //     let Some(start) = boxes
    //         .iter()
    //         .find(|&(_, block_type)| *block_type == BlockType::Start)
    //         .map(|(e, _)| e)
    //     else {
    //         return;
    //     };
    //     let traverse = global_ast.traverse_tree(start);
    //     info!("{}", traverse.print(&boxes));
    // }
}

impl Plugin for ASTPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<Ast>()
            .init_resource::<BlockDataMap>()
            .add_event::<AddToASTEvent>()
            .add_event::<RemoveFromAST>()
            .add_systems(
                Update,
                (
                    get_block_data_hashmap.run_if(input_just_pressed(KeyCode::KeyP))
                    // Self::handle_add_to_ast,
                    // Self::handle_remove_from_ast,
                    // Self::print_ast.run_if(
                    //     in_state(DragState::Ended).and_then(input_just_released(KeyCode::KeyP)),
                    // ),
                )
                .chain()
                .in_set(GameSets::Running),
            );
    }
}
