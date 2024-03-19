use crate::utils::{Position, Size};
use bevy::{input::mouse::MouseButtonInput, prelude::*};
use rand::Rng;

/// This is a marker component that is used to identify any
/// box element
#[derive(Component)]
struct Box;

/// This is a marker component that is used to identify any
/// UI box element
#[derive(Component)]
struct UIBox;

/// This is a marker component that is used to identify the
/// background box that contains the whole app
#[derive(Component)]
struct BackgroundBox;

#[derive(Bundle)]
struct UIBoxBundle {
    r#box: Box,
    marker: UIBox,
    position: Position,
    size: Size,
    node: ButtonBundle,
}

impl UIBoxBundle {
    fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            marker: UIBox,
            position: Position(Vec2::new(x, y)),
            size: Size(Vec2::new(w, h)),
            node: ButtonBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(y),
                    left: Val::Px(x),
                    width: Val::Px(w),
                    height: Val::Px(h),
                    border: UiRect::all(Val::Px(1.)),
                    ..Default::default()
                },
                background_color: Color::WHITE.into(),
                ..Default::default()
            },
            r#box: Box,
        }
    }
}

#[derive(Resource, Default)]
struct ActiveUIBox {
    entity: Option<Entity>,
}

#[derive(Event, Default)]
struct ChangedActiveBoxEvent(pub Option<Entity>);

#[derive(Event, Default)]
pub struct BoxHoverEvent(pub Option<Entity>);

/// This is a plugin for the UI Box element. It contains all the systems for the UI Box
pub struct UIBoxPlugin;

/// This is a list of all the systems that are contained in the UIBox Entity
impl UIBoxPlugin {
    fn spawn_ui_box(mut commands: Commands) {
        let background_container = ButtonBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                position_type: PositionType::Relative,
                ..default()
            },
            background_color: Color::rgba(0., 0., 0., 0.).into(),
            ..Default::default()
        };

        commands
            .spawn((Box, BackgroundBox, background_container))
            .with_children(|parent| {
                for _ in 0..3 {
                    let mut random = rand::thread_rng();
                    let random_x = random.gen_range(0. ..200.);
                    let random_y = random.gen_range(0. ..200.);
                    parent.spawn(UIBoxBundle::new(random_x, random_y, 50., 100.));
                }
            });
    }

    fn handle_box_hover(
        mut reader: EventReader<BoxHoverEvent>,
        mut boxes: Query<(Entity, &mut BorderColor), With<UIBox>>,
    ) {
        for &BoxHoverEvent(entity) in reader.read() {
            for (box_entity, mut border_color) in &mut boxes {
                let value = *border_color;

                if value.0 == Color::RED {
                    continue;
                }
                if entity.is_some_and(|entity| entity == box_entity) {
                    *border_color = Color::GREEN.into();
                } else {
                    *border_color = BorderColor::default()
                }
            }
        }
    }

    fn handle_changed_active_ui_box(
        previous_active: Res<ActiveUIBox>,
        mut reader: EventReader<ChangedActiveBoxEvent>,
        mut boxes: Query<&mut BorderColor, With<UIBox>>,
    ) {
        for &ChangedActiveBoxEvent(entity) in reader.read() {
            // Clear the border of the previous one
            match previous_active.entity {
                Some(entity) => {
                    let mut border_color = boxes.get_mut(entity).unwrap();
                    *border_color = BorderColor::default();
                }
                _ => {}
            }

            // Change the border color for the new active
            match entity {
                Some(entity) => {
                    let mut border_color = boxes.get_mut(entity).unwrap();
                    *border_color = Color::RED.into()
                }
                _ => {}
            }
        }
    }

    fn change_active(
        mut reader: EventReader<ChangedActiveBoxEvent>,
        mut active: ResMut<ActiveUIBox>,
    ) {
        for &ChangedActiveBoxEvent(new_active) in reader.read() {
            active.entity = new_active;
        }
    }

    fn select_background(
        background: Query<&Interaction, With<BackgroundBox>>,
        mut changed_active_writer: EventWriter<ChangedActiveBoxEvent>,
        mut hover_box_writer: EventWriter<BoxHoverEvent>,
    ) -> () {
        let interaction = background.single();
        // let interaction = match background.get_single() {
        //     Ok((_, interaction)) => interaction,
        //     Err(error) => {
        //         println!("Error: {error}");
        //         &Interaction::None
        //     }
        // };

        match interaction {
            Interaction::Pressed => {
                // This should set the active box to none
                changed_active_writer.send_default();
            }
            Interaction::Hovered => {
                hover_box_writer.send_default();
            }
            _ => {}
        }
    }

    /// This function selects the box and triggers a `ChangedActiveBoxEvent` event<br>
    /// It also fires an evenet when the button is hovered on
    fn select_ui_box(
        ui_boxes: Query<(Entity, &Interaction), (Changed<Interaction>, With<UIBox>)>,
        active: Res<ActiveUIBox>,
        mut changed_active_writer: EventWriter<ChangedActiveBoxEvent>,
        mut hover_box_writer: EventWriter<BoxHoverEvent>,
    ) {
        for (box_entity, interaction) in &ui_boxes {
            match interaction {
                Interaction::Pressed => {
                    if active.entity.is_none()
                        || active.entity.is_some_and(|entity| entity != box_entity)
                    {
                        changed_active_writer.send(ChangedActiveBoxEvent(Some(box_entity)));
                    }
                    return;
                }
                Interaction::Hovered => {
                    if active.entity.is_none()
                        || (&active.entity).is_some_and(|entity| entity != box_entity)
                    {
                        hover_box_writer.send(BoxHoverEvent(Some(box_entity)));
                    }
                    return;
                }
                Interaction::None => (),
            };
        }
    }

    /// This function moves the box accourding to the mouse position if the mouse is pressing the UI box downn
    fn move_according_to_mouse(
        mut cursor_motions: EventReader<CursorMoved>,
        mut boxes: Query<&mut Position, (Changed<Interaction>, With<UIBox>)>,
        mouse_button_input: Res<ButtonInput<MouseButton>>,
        active: Res<ActiveUIBox>,
    ) {
        if let Some(active_entity) = active.entity {
            for cursor in cursor_motions.read() {
                if let Ok(mut position) = boxes.get_mut(active_entity) {
                    position.0 = cursor.position;

                    // match interaction {
                    //     Interaction::Pressed => {
                    //         position.0 = cursor.position;
                    //         info!("Moved Cursor")
                    //     }
                    //     _ => (),
                    // }
                }
            }
        }
    }

    fn update_box_properties(mut query: Query<(&mut Style, &Position, &Size), With<UIBox>>) {
        for (mut style, position, size) in &mut query {
            style.width = Val::Px(size.0.x);
            style.height = Val::Px(size.0.y);
            style.top = Val::Px(position.0.y);
            style.left = Val::Px(position.0.x);
        }
    }
}

impl Plugin for UIBoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChangedActiveBoxEvent>()
            .add_event::<BoxHoverEvent>()
            .insert_resource(ActiveUIBox::default())
            .add_systems(Startup, Self::spawn_ui_box)
            .add_systems(
                Update,
                (
                    (Self::select_ui_box, Self::select_background),
                    // Self::select_ui_box,
                    Self::handle_box_hover,
                    Self::handle_changed_active_ui_box,
                    Self::change_active,
                    Self::move_according_to_mouse,
                    Self::update_box_properties,
                )
                    .chain(),
            );
    }
}
