use bevy::{input::common_conditions::input_just_pressed, prelude::*, ui::FocusPolicy};
use bevy_simple_text_input::{
    TextInputBundle, TextInputInactive, TextInputSettings, TextInputSubmitEvent, TextInputValue,
};

use crate::{
    ast::UpdateAst,
    focus::{ActiveEntity, FocusBundle, InteractionFocusBundle, SelectEvent},
    ui_box::{BackgroundBox, BlockBundle, SpawnUIBox},
    utils::{BlockType, HoleType, Language},
    ErrorEvent, GameSets,
};

#[derive(Debug, Component, Clone, Copy)]
pub struct TextInput {
    pub owner: Entity,
}

#[derive(Bundle)]
pub struct CustomTextInputBundle {
    text_input_bundle: TextInputBundle,
    text_input: TextInput,
    focusable: FocusBundle,
    node_bundle: NodeBundle,
    entity_label: crate::EntityLabel,
}

impl CustomTextInputBundle {
    pub fn new(text_input_bundle: TextInputBundle, owner: Entity) -> Self {
        Self {
            node_bundle: NodeBundle {
                style: Style {
                    min_width: Val::Px(30.),
                    border: UiRect::bottom(Val::Px(2.)),
                    height: Val::Px(20.),
                    ..default()
                },
                border_color: Color::BLACK.into(),
                focus_policy: FocusPolicy::Block,
                ..default()
            },
            text_input_bundle,
            text_input: TextInput { owner },
            focusable: FocusBundle::new(Color::RED, Color::GREEN, Color::BLACK),
            entity_label: crate::EntityLabel::new("Text Box"),
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct SearchBox;

#[derive(Component, Debug, Clone, Copy)]
pub struct SearchContainer;

#[derive(Bundle)]
pub struct SearchBoxBundle {
    node: NodeBundle,
    marker: SearchBox,
    focus: FocusBundle,
    text_input: TextInputBundle,
    entity_label: crate::EntityLabel,
}

impl SearchBoxBundle {
    pub fn new() -> Self {
        Self {
            node: NodeBundle {
                style: Style {
                    padding: UiRect::all(Val::Px(2.)),
                    width: Val::Percent(100.),
                    border: UiRect::bottom(Val::Px(1.)),
                    min_height: Val::Px(20.),
                    ..default()
                },
                border_color: Color::BLACK.into(),
                focus_policy: FocusPolicy::Block,
                ..default()
            },
            text_input: TextInputBundle::default()
                .with_placeholder(
                    "Search for block",
                    Some(TextStyle {
                        color: Color::BLACK,
                        ..default()
                    }),
                )
                .with_text_style(TextStyle {
                    color: Color::BLACK,
                    ..default()
                })
                .with_settings(TextInputSettings {
                    retain_on_submit: false,
                    ..default()
                })
                .with_inactive(true),
            marker: SearchBox,
            focus: FocusBundle::new(Color::RED, Color::GREEN, Color::BLACK),
            entity_label: crate::EntityLabel::new("Search Box"),
        }
    }
}

#[derive(Resource, Default, Debug, Clone, Copy)]
struct IsSearchVisible(bool);

pub struct CustomTextInputPlugin;

impl CustomTextInputPlugin {
    fn handle_text_focus(
        active_entity: Res<ActiveEntity>,
        mut query: Query<(Entity, &mut TextInputInactive)>,
    ) {
        if let Some(active_entity) = active_entity.entity {
            for (text_entity, mut text_inactive) in &mut query {
                text_inactive.0 = text_entity != active_entity;
            }
        }
    }

    fn send_update_ast(
        block_types: Query<&BlockType>,
        text_query: Query<&TextInput, Changed<TextInputValue>>,
        mut update_writer: EventWriter<UpdateAst>,
    ) {
        for text_input in &text_query {
            if block_types.contains(text_input.owner) {
                update_writer.send_default();
            }
        }
    }

    fn set_text_block_type(
        mut block_types: Query<&mut BlockType>,
        text_query: Query<(&TextInput, &TextInputValue), Changed<TextInputInactive>>,
        mut reader: EventReader<SelectEvent>,
    ) {
        if reader.read().next().is_some() {
            for (text_input, value) in &text_query {
                if let Ok(mut block_type) = block_types.get_mut(text_input.owner) {
                    if block_type.name == "Text" {
                        block_type.value = HoleType::get_derived_type(value.0.as_str());
                    }
                }
            }
        }
    }

    fn spawn_search_box(mut commands: Commands, background: Query<Entity, With<BackgroundBox>>) {
        commands
            .entity(background.single())
            .with_children(|parent| {
                parent
                    .spawn((
                        NodeBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                bottom: Val::Px(0.),
                                width: Val::Percent(100.),
                                flex_direction: FlexDirection::Column,
                                min_height: Val::Px(20.),
                                padding: UiRect::all(Val::Px(5.)),
                                ..default()
                            },
                            visibility: Visibility::Hidden,
                            focus_policy: FocusPolicy::Pass,
                            ..default()
                        },
                        SearchContainer,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            TextBundle::from_section(
                                "Search Box",
                                TextStyle {
                                    color: Color::BLACK,
                                    ..default()
                                },
                            )
                            .with_text_justify(JustifyText::Left),
                            Label,
                        ));
                        parent.spawn(SearchBoxBundle::new());
                    });
            });
    }

    fn handle_visiblity(
        mut query: Query<&mut Visibility, With<SearchContainer>>,
        is_visible: Res<IsSearchVisible>,
    ) {
        for mut visibility in &mut query {
            *visibility = if is_visible.0 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }

    fn toggle_visibility(
        mut is_visible: ResMut<IsSearchVisible>,
        mut value_query: Query<(Entity, &mut TextInputValue), With<SearchBox>>,
        mut writer: EventWriter<SelectEvent>,
    ) {
        // Toggle the visibility
        is_visible.0 = !is_visible.0;
        // Clear the search box
        let (entity, mut value) = value_query.single_mut();
        value.0.clear();
        // Focus on the search box
        writer.send(SelectEvent(Some(entity)));
    }

    fn handle_search_box_submit(
        mut reader: EventReader<TextInputSubmitEvent>,
        mut error_writer: EventWriter<ErrorEvent>,
        mut spawn_box: EventWriter<SpawnUIBox>,
        language: Res<Language>,
        search_box: Query<&SearchBox>,
        background: Query<&Node, With<BackgroundBox>>,
    ) {
        for event in reader.read() {
            if search_box.get(event.entity).is_ok() {
                info!("Search Box Submit: {:?}", event.value);
                let Some(blocks) = language.blocks.iter().find(|block| {
                    block
                        .name
                        .to_lowercase()
                        .contains(event.value.to_lowercase().as_str())
                }) else {
                    error_writer.send(ErrorEvent(format!(
                        "Block with name {} not found",
                        event.value
                    )));
                    continue;
                };
                let coordinates = background.single().size() / 2.;
                spawn_box.send(SpawnUIBox {
                    marker: None,
                    bundle: BlockBundle::new(
                        coordinates.x,
                        coordinates.y,
                        40.,
                        40.,
                        InteractionFocusBundle::default(),
                        blocks.to_owned(),
                    ),
                });
            }
        }
    }
}

impl Plugin for CustomTextInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IsSearchVisible>()
            .add_systems(PostStartup, Self::spawn_search_box)
            .add_systems(
                Update,
                (
                    Self::handle_text_focus,
                    Self::set_text_block_type,
                    Self::handle_visiblity,
                    Self::toggle_visibility.run_if(input_just_pressed(KeyCode::Slash)),
                    Self::handle_search_box_submit,
                    Self::send_update_ast,
                )
                    .chain()
                    .in_set(GameSets::Running),
            );
    }
}
