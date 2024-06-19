use bevy::{input::common_conditions::input_just_pressed, prelude::*, utils::HashMap};
use bevy_simple_text_input::TextInputValue;

use crate::{text_input::TextInput, ui_box::Hole, utils::BlockType, GameSets};

#[derive(Debug, Clone)]
pub struct BlockData {
    block_type: BlockType,
    data_type: BlockDataType,
    position: usize,
}

#[derive(Debug, Clone)]
pub enum BlockDataType {
    Hole(Entity),
    Value(String),
}

#[derive(Debug, Resource, Default)]
pub struct BlockDataMap {
    pub map: HashMap<Entity, Vec<BlockData>>,
}

impl BlockDataMap {
    fn expand_holes(&self, block_entity: Entity, block_type: BlockType) -> String {
        let Some(data) = self.map.get(&block_entity) else {
            info!("Block {block_type:?} doesn't have an entry in the template string");
            return block_type.get_template().into();
        };
        let mut data = data.clone();

        // Make sure that the block is always sorted when we want to get the holes
        data.sort_by(|data1, data2| data1.position.cmp(&data2.position));

        let mut value = Vec::with_capacity(block_type.get_holes());

        for data in data.iter().cloned() {
            match data.data_type {
                BlockDataType::Value(val) => value.push(val),
                BlockDataType::Hole(entity) => {
                    value.push(self.expand_holes(entity, data.block_type))
                }
            }
        }

        let mut template_string = block_type.get_template().to_owned();
        for (index, value) in value.iter().enumerate() {
            let index = index + 1;
            template_string = template_string
                .replacen(format!("{{{{{index}}}}}").as_str(), value, 1)
                .to_owned();
        }
        template_string
    }
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

        let data_type = match child_block {
            BlockType::Text => {
                let Some((_, text_value)) = text_input
                    .iter()
                    .find(|(text_input, _)| text_input.owner == child_entity)
                else {
                    info!("Entity {child_entity:?} had a BlockType::Text but no TextInputValue");
                    continue;
                };
                BlockDataType::Value(text_value.0.clone())
            }

            _ => BlockDataType::Hole(child_entity),
        };
        let block_data = BlockData {
            block_type: child_block.to_owned(),
            data_type,
            position: hole.order,
        };
        value.push(block_data);
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
    pub map: HashMap<Entity, [Option<(Entity, BlockType)>; 3]>,
    // pub map: HashMap<Entity, Vec<Entity>>,
}

impl Ast {
    fn traverse_branch(
        &self,
        entity: Entity,
        block_type: BlockType,
        block_data_map: &BlockDataMap,
    ) -> String {
        // Expand the holes inside the block
        let mut full_string = block_data_map.expand_holes(entity, block_type);

        let hole = block_type.get_holes();
        let Some(branches) = self.map.get(&entity) else {
            return full_string;
        };

        // Expand the left and right branches
        for (index, branch) in branches
            .get(0..=1)
            .unwrap()
            .iter()
            .filter(|x| x.is_some())
            .enumerate()
        {
            match branch.to_owned() {
                Some((branch_entity, branch_block_type)) => {
                    let string =
                        self.traverse_branch(branch_entity, branch_block_type, block_data_map);
                    full_string = full_string.replacen(
                        format!("{{{{{}}}}}", hole + index + 1).as_str(),
                        string.as_str(),
                        1,
                    );
                }
                None => unreachable!("This should not be reachable"),
            }
        }

        // Expand the flow branch
        match branches.last().and_then(ToOwned::to_owned) {
            Some((branch_entity, branch_block_type)) => {
                let string = self.traverse_branch(branch_entity, branch_block_type, block_data_map);
                format!("{full_string}\n{string}")
            }
            None => full_string,
        }
    }
}

// TODO: Make specialized events for adding a child to a parent
// and removing a child from a parent

#[derive(Debug, Event, Clone, Copy)]
pub struct AddToAst {
    pub parent: Option<(Entity, usize)>,
    pub child: (Entity, BlockType),
}

#[derive(Debug, Event, Clone, Copy)]
pub struct RemoveFromAst {
    pub parent: Option<(Entity, usize)>,
    pub child: Entity,
}

pub struct ASTPlugin;

impl ASTPlugin {
    fn handle_add_to_ast(mut reader: EventReader<AddToAst>, mut ast: ResMut<Ast>) {
        for event in reader.read() {
            if let Some((parent, order)) = event.parent {
                let value = ast.map.entry(parent).or_default();
                value[order] = Some(event.child)
            } else {
                ast.map.entry(event.child.0).or_default();
            }
        }
    }

    fn handle_remove_from_ast(mut reader: EventReader<RemoveFromAst>, mut ast: ResMut<Ast>) {
        for event in reader.read() {
            if let Some((parent, order)) = event.parent {
                let value = ast.map.entry(parent).or_default();
                value[order] = None;
            } else {
                ast.map.remove_entry(&event.child);
            }
        }
    }

    fn print_ast(
        ast: Res<Ast>,
        block_data_map: Res<BlockDataMap>,
        block_type: Query<(Entity, &BlockType)>,
    ) {
        let Some((start_entity, &start_block)) = block_type
            .iter()
            .find(|(_, block_type)| matches!(block_type, BlockType::Start))
        else {
            info!("There is no start block in the world");
            return;
        };
        let code = ast.traverse_branch(start_entity, start_block, block_data_map.as_ref());
        info!("====== Outputed Code ======");
        info!("{code}");

        // Stage 1: Get the template string for the block
        // Stage 2: Explore any alternative branches. Repeat step 1 there too
        // Stage 3: Explore the main flow branch
    }
}

impl Plugin for ASTPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<Ast>()
            .init_resource::<BlockDataMap>()
            .add_event::<AddToAst>()
            .add_event::<RemoveFromAst>()
            .add_systems(
                Update,
                (
                    get_block_data_hashmap.run_if(input_just_pressed(KeyCode::KeyP)),
                    Self::handle_add_to_ast,
                    Self::handle_remove_from_ast,
                    Self::print_ast.run_if(input_just_pressed(KeyCode::KeyQ)),
                )
                    .chain()
                    .in_set(GameSets::Running),
            );
    }
}
