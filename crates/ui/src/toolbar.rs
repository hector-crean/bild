use bevy::state::state::FreelyMutableState;
use bevy::{color::palettes, prelude::*, ui::widget::NodeImageMode};
use strum::EnumProperty;
use strum::IntoEnumIterator;

// Generic toolbar that works with any state enum
#[derive(Component)]
pub struct Toolbar<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Toolbar<T> {
    // Define button colors as constants
    const NORMAL_BUTTON: Color = Color::NONE;
    const HOVERED_BUTTON: Color = Color::srgb(0.475, 0.475, 0.475);
    const PRESSED_BUTTON: Color = Color::srgb(0.592, 0.694, 0.835);
    const SELECTED_BUTTON: Color = Color::srgb(0.4, 0.6, 0.8);
    const CONTAINER_COLOR: Color = Color::srgb(0.235, 0.235, 0.235);
}

// Trait that defines what a state enum needs to implement for the toolbar
pub trait ToolbarState:
    FreelyMutableState
    + IntoEnumIterator
    + Into<&'static str>
    + Copy
    + Component
    + EnumProperty
    + PartialEq
    + Eq
    + 'static
{
    fn get_icon(&self) -> &'static str;
}

impl<T: ToolbarState> Toolbar<T> {
    pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Auto,
                    bottom: Val::Px(8.0),
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Toolbar::<T> {
                    _phantom: std::marker::PhantomData,
                },
            ))
            .with_children(|parent| {
                parent
                    .spawn((
                        Node {
                            width: Val::Auto,
                            height: Val::Auto,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(8.0),
                            padding: UiRect::all(Val::Px(8.)),
                            ..default()
                        },
                        BackgroundColor(Self::CONTAINER_COLOR),
                        BorderRadius::new(
                            // top left
                            Val::Px(4.),
                            // top right
                            Val::Px(4.),
                            // bottom right
                            Val::Px(4.),
                            // bottom left
                            Val::Px(4.),
                        ),
                    ))
                    .with_children(|toolbar| {
                        for state in T::iter() {
                            toolbar
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(32.0),
                                        height: Val::Px(32.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        display: Display::Flex,
                                        flex_direction: FlexDirection::Column,
                                        column_gap: Val::Px(4.0),
                                        ..default()
                                    },
                                    state,
                                    BackgroundColor(Self::NORMAL_BUTTON),
                                    BorderColor::all(Color::BLACK),
                                    BorderRadius::new(
                                        // top left
                                        Val::Px(4.),
                                        // top right
                                        Val::Px(4.),
                                        // bottom right
                                        Val::Px(4.),
                                        // bottom left
                                        Val::Px(4.),
                                    ),
                                ))
                                .with_children(|button| {
                                    let tool_icon = state.get_icon();

                                    button.spawn((
                                        ImageNode {
                                            image: asset_server.load(tool_icon),
                                            color: Color::WHITE,
                                            flip_x: false,
                                            flip_y: false,
                                            image_mode: NodeImageMode::Auto,
                                            ..default()
                                        },
                                        Node {
                                            width: Val::Px(24.0),
                                            height: Val::Px(24.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            display: Display::Flex,
                                            flex_direction: FlexDirection::Column,
                                            column_gap: Val::Px(4.0),
                                            ..default()
                                        },
                                    ));
                                });
                        }
                    });
            });
    }

    pub fn handle_interaction_changes(
        mut next_state: ResMut<NextState<T>>,
        current_state: Res<State<T>>,
        mut query: Query<
            (&Interaction, &mut BackgroundColor, &mut BorderColor, &T),
            (Changed<Interaction>, With<Button>),
        >,
    ) {
        for (interaction, mut color, mut border_color, state) in &mut query {
            let active_state_label: &'static str = (*current_state.get()).into();
            let state_label: &'static str = (*state).into();
            let state_active = active_state_label == state_label;

            match (*interaction, state_active) {
                (Interaction::Pressed, _) => {
                    next_state.set(*state);
                    *color = Self::PRESSED_BUTTON.into();
                    border_color.set_all(Self::PRESSED_BUTTON);
                }
                (Interaction::Hovered, true) | (Interaction::None, true) => {
                    *color = Self::SELECTED_BUTTON.into();
                    border_color.set_all(Self::SELECTED_BUTTON);
                }
                (Interaction::Hovered, false) => {
                    *color = Self::HOVERED_BUTTON.into();
                    border_color.set_all(Self::HOVERED_BUTTON);
                }
                (Interaction::None, false) => {
                    *color = Self::NORMAL_BUTTON.into();
                    border_color.set_all(Self::NORMAL_BUTTON);
                }
            }
        }
    }

    pub fn update_state(
        current_state: Res<State<T>>,
        mut query: Query<(&mut BackgroundColor, &mut BorderColor, &T), With<Button>>,
    ) {
        for (mut color, mut border_color, state) in &mut query {
            let active_state_label: &'static str = (*current_state.get()).into();
            let state_label: &'static str = (*state).into();
            let state_active = active_state_label == state_label;

            match state_active {
                true => {
                    *color = Self::SELECTED_BUTTON.into();
                    border_color.set_all(Self::SELECTED_BUTTON)  ;
                }
                false => {
                    *color = Self::NORMAL_BUTTON.into();
                    border_color.set_all(Self::NORMAL_BUTTON);
                }
            }
        }
    }
}



// Plugin for the generic toolbar
pub struct ToolbarPlugin<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: ToolbarState> Default for ToolbarPlugin<T> {
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: ToolbarState> Plugin for ToolbarPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Toolbar::<T>::setup).add_systems(
            Update,
            (
                Toolbar::<T>::handle_interaction_changes,
                Toolbar::<T>::update_state.run_if(on_event::<StateTransitionEvent<T>>),
            ),
        );
    }
}
