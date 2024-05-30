use std::fmt::Display;

use bevy::{input::common_conditions::input_just_released, prelude::*, utils::HashMap};

use crate::{
    focus::DragState,
    utils::{BlockType, ConnectionType},
};

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

    fn print(&self, boxes: &Query<(Entity, &BlockType), With<crate::r#box::Box>>) -> String {
        let mut result = String::with_capacity(256);

        let (_, start_block_type) = boxes
            .get(self.entity)
            .expect("Expected box to be in the tree");

        result.push_str(format!("( {start_block_type} ").as_str());

        if let Some(left) = &self.left {
            result.push_str(left.print(boxes).as_str());
        }

        if let Some(right) = &self.right {
            result.push_str(right.print(boxes).as_str());
        }

        result.push_str(") ");

        if let Some(flow) = &self.flow {
            result.push_str(" -> ");
            result.push_str(flow.print(boxes).as_str());
        }
        result
    }

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

    fn print_ast(
        global_ast: Res<Ast>,
        boxes: Query<(Entity, &BlockType), With<crate::r#box::Box>>,
    ) {
        let Some(start) = boxes
            .iter()
            .find(|&(_, block_type)| *block_type == BlockType::Start)
            .map(|(e, _)| e)
        else {
            return;
        };
        let traverse = global_ast.traverse_tree(start);
        info!("{}", traverse.print(&boxes));
    }
}

impl Plugin for ASTPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<Ast>()
            .add_event::<AddToASTEvent>()
            .add_event::<RemoveFromAST>()
            .add_systems(
                Update,
                (
                    Self::handle_add_to_ast,
                    Self::handle_remove_from_ast,
                    Self::print_ast.run_if(
                        in_state(DragState::Ended).and_then(input_just_released(KeyCode::KeyP)),
                    ),
                )
                    .chain(),
            );
    }
}
