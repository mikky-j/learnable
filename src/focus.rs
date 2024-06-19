use bevy::{
    input::common_conditions::{input_just_pressed, input_just_released},
    prelude::*,
    ui::FocusPolicy,
};

use crate::{ui_line::Segment, utils::point_line_collision, DeleteEvent, GameSets};

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Focus {
    pub active: Color,
    pub inactive: Color,
    pub hover: Color,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct LineFocus;

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Draggable;

#[derive(Debug, Resource, Default)]
pub struct ActiveEntity {
    pub entity: Option<Entity>,
}

#[derive(Debug, Resource, Default)]
pub struct HoverEntity {
    pub entity: Option<Entity>,
}

#[derive(Debug, Resource, Default)]
pub struct DragEntity {
    pub entity: Option<Entity>,
    /// This is the start Global translation of the object that is being dragged
    pub drag_start: Option<Vec2>,
}

#[derive(Debug, States, PartialEq, Eq, Hash, Clone, Copy, Default)]
pub enum DragState {
    Started,
    #[default]
    Ended,
}

#[derive(Clone, Copy, Debug, Event)]
pub struct HoverEvent(pub Option<Entity>);

#[derive(Debug, Clone, Copy, Event)]
pub struct SelectEvent(pub Option<Entity>);

#[derive(Debug, Clone, Copy, Component, Default)]
pub struct FocusColor(pub Color);

#[derive(Debug, Bundle, Clone, Copy, Default)]
pub struct FocusBundle {
    focus: Focus,
    focus_color: FocusColor,
}

#[derive(Debug, Bundle, Clone, Copy)]
pub struct InteractionFocusBundle {
    focus_bundle: FocusBundle,
    interaction: Interaction,
}

#[derive(Debug, Bundle, Clone, Copy)]
pub struct LineFocusBundle {
    focus_bundle: FocusBundle,
    interaction: LineFocus,
}

impl FocusBundle {
    pub fn new(active: Color, hover: Color, inactive: Color) -> Self {
        Self {
            focus: Focus {
                active,
                inactive,
                hover,
            },
            focus_color: FocusColor(inactive),
        }
    }
}

impl LineFocusBundle {
    pub fn new(active: Color, hover: Color, inactive: Color) -> Self {
        Self {
            interaction: LineFocus,
            focus_bundle: FocusBundle::new(active, hover, inactive),
        }
    }
}

impl InteractionFocusBundle {
    pub fn new(active: Color, hover: Color, inactive: Color) -> Self {
        Self {
            interaction: Interaction::default(),
            focus_bundle: FocusBundle::new(active, hover, inactive),
        }
    }
}

impl Default for InteractionFocusBundle {
    fn default() -> Self {
        Self {
            focus_bundle: FocusBundle::new(Color::RED, Color::GREEN, Color::WHITE),
            interaction: Default::default(),
        }
    }
}

pub struct FocusPlugin;

impl FocusPlugin {
    fn handle_interaction(
        focus_policy: Query<&FocusPolicy>,
        interactions: Query<(Entity, &Interaction), (Changed<Interaction>, With<Focus>)>,
        // labels: Query<&EntityLabel>,
        mut select_writer: EventWriter<SelectEvent>,
        mut hover_writer: EventWriter<HoverEvent>,
    ) {
        for (entity, &interaction) in &interactions {
            // let label = labels
            //     .get(entity)
            //     .map(ToOwned::to_owned)
            //     .unwrap_or_default();
            if focus_policy
                .get(entity)
                .ok()
                .is_some_and(|policy| matches!(policy, FocusPolicy::Pass))
            {
                info!("Encountered weird bevy bug");
                continue;
            }
            match interaction {
                Interaction::Pressed => {
                    // info!("{} was selected", label.0);
                    select_writer.send(SelectEvent(Some(entity)));
                }
                Interaction::Hovered => {
                    // info!("{} {entity:?} was hovered", label.0);
                    hover_writer.send(HoverEvent(Some(entity)));
                }
                Interaction::None => (),
            }
        }
    }

    fn handle_select_event(
        old_selected: Res<ActiveEntity>,
        mut color: Query<(&mut FocusColor, &Focus)>,
        mut reader: EventReader<SelectEvent>,
    ) {
        for &SelectEvent(entity) in reader.read() {
            if let Some(select_box_entity) = entity {
                let (mut focus_color, focus) = color
                    .get_mut(select_box_entity)
                    .expect("Expected selcted UI box to be in the world tree");
                focus_color.0 = focus.active;
            }

            if let Some(old_entity) = old_selected
                .entity
                .filter(|&old_entity| !entity.is_some_and(|new| new == old_entity))
            {
                let (mut focus_color, focus) = color
                    .get_mut(old_entity)
                    .expect("Expected the old box to be in the world tree");
                focus_color.0 = focus.inactive;
            }
        }
    }

    fn handle_hover_event(
        selected_box: Res<ActiveEntity>,
        // old_hover_box: Res<HoverEntity>,
        mut color: Query<(Entity, &mut FocusColor, &Focus)>,
        mut reader: EventReader<HoverEvent>,
    ) {
        for &HoverEvent(new_hover_box) in reader.read() {
            for (entity, mut focus_color, focus) in &mut color {
                if selected_box
                    .entity
                    .is_some_and(|selcted_entity| selcted_entity == entity)
                {
                    continue;
                }
                if new_hover_box.is_some_and(|new_hover| new_hover == entity) {
                    focus_color.0 = focus.hover;
                } else {
                    focus_color.0 = focus.inactive;
                }
            }
        }
    }

    fn set_active(mut reader: EventReader<SelectEvent>, mut active: ResMut<ActiveEntity>) {
        for &SelectEvent(event) in reader.read() {
            active.entity = event;
        }
    }

    fn set_hover(mut reader: EventReader<HoverEvent>, mut active: ResMut<HoverEntity>) {
        for &HoverEvent(event) in reader.read() {
            active.entity = event;
        }
    }

    fn start_drag_state(
        hover: Query<&Interaction, With<Draggable>>,
        active: Res<ActiveEntity>,
        mut next_state: ResMut<NextState<DragState>>,
        mut drag: ResMut<DragEntity>,
        global_translation: Query<&GlobalTransform>,
    ) {
        if let Some(entity) = active.entity.filter(|&entity| {
            // Check if the hovered entity has a draggable component
            hover
                .get(entity)
                .ok()
                .is_some_and(|interaction| matches!(interaction, Interaction::Pressed))
        }) {
            next_state.set(DragState::Started);
            drag.entity = Some(entity);
            drag.drag_start = Some(
                global_translation
                    .get(entity)
                    .expect("All entities should have a global translation right?")
                    .translation()
                    .xy(),
            );
        }
    }

    fn end_drag_state(mut next_state: ResMut<NextState<DragState>>) {
        next_state.set(DragState::Ended);
    }

    fn handle_drag_state_transitions(
        mut transitions: EventReader<StateTransitionEvent<DragState>>,
        hover: Query<&Interaction, With<Draggable>>,
        active: Res<ActiveEntity>,
        mut drag: ResMut<DragEntity>,
    ) {
        for new_state in transitions.read() {
            match new_state.after {
                DragState::Started => {
                    if let Some(entity) = active.entity {
                        if hover.get(entity).expect("Should never fail") == &Interaction::Pressed {
                            drag.entity = Some(entity);
                        }
                    }
                }
                DragState::Ended => {
                    drag.entity = None;
                    drag.drag_start = None;
                }
            }
        }
    }

    fn handle_delete(
        mut reader: EventReader<DeleteEvent>,
        mut active: ResMut<ActiveEntity>,
        mut hover: ResMut<HoverEntity>,
        mut drag: ResMut<DragEntity>,
        mut next_state: ResMut<NextState<DragState>>,
        // query: Query<&Focus>,
    ) {
        for &DeleteEvent(_) in reader.read() {
            active.entity = None;
            hover.entity = None;
            drag.entity = None;
            next_state.set(DragState::Ended);
        }
    }

    fn handle_focus_line(
        query: Query<&Segment>,
        mut cursor_motion: EventReader<CursorMoved>,
        mut hover_writer: EventWriter<HoverEvent>,
    ) {
        for segments in &query {
            for motion in cursor_motion.read() {
                if point_line_collision((segments.from, segments.to), motion.position, Some(1.)) {
                    info!("Hovering on {:?}", segments.owner);
                    hover_writer.send(HoverEvent(Some(segments.owner)));
                }
            }
        }
    }
}

impl Plugin for FocusPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HoverEvent>()
            .add_event::<SelectEvent>()
            .init_state::<DragState>()
            .init_resource::<ActiveEntity>()
            .init_resource::<DragEntity>()
            .init_resource::<HoverEntity>()
            .add_systems(
                Update,
                (
                    (
                        (
                            Self::handle_interaction,
                            Self::handle_focus_line.run_if(in_state(DragState::Ended)),
                            Self::handle_hover_event,
                            Self::set_hover,
                            Self::handle_select_event,
                            Self::set_active,
                            Self::start_drag_state.run_if(input_just_pressed(MouseButton::Left)),
                        )
                            .chain(),
                        // .run_if(in_state(DragState::Ended)),
                        Self::end_drag_state.run_if(
                            in_state(DragState::Started)
                                .and_then(input_just_released(MouseButton::Left)),
                        ),
                    )
                        .in_set(GameSets::Running),
                    Self::handle_delete.in_set(GameSets::Despawn),
                ),
            )
            .add_systems(Last, Self::handle_drag_state_transitions);
    }
}
