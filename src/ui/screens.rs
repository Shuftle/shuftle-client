//! The screens shown before and between games: the menus, and the wait while
//! a game loads.

mod about;
mod loading;
mod main_menu;
mod settings;

use bevy::{
    ecs::schedule::common_conditions::{any_component_removed, any_match_filter},
    input::keyboard::KeyboardInput,
    input_focus::FocusedInput,
    prelude::*,
    ui::{Checked, Pressed},
    ui_widgets::radio_self_update,
};

use super::widgets;
use crate::{
    settings::{BackId, DeckId, FeltColour},
    state::{AppState, Screen, is_back},
};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Main), main_menu::scene.spawn())
            .add_systems(OnEnter(Screen::Settings), settings::spawn)
            .add_systems(OnEnter(Screen::About), about::scene.spawn())
            .add_systems(OnEnter(AppState::Loading), loading::scene.spawn())
            .add_observer(back.run_if(in_state(AppState::Menu)))
            .add_systems(
                Update,
                widgets::shade_buttons.run_if(
                    any_match_filter::<widgets::ShadeChanged>
                        .or_else(any_component_removed::<Pressed>)
                        .or_else(any_component_removed::<Checked>),
                ),
            )
            .add_systems(
                Update,
                widgets::shade_thumbs.run_if(any_match_filter::<widgets::ThumbChanged>),
            )
            .add_observer(widgets::drag_scroll_start)
            .add_observer(widgets::drag_scroll)
            // Moves the check to the option picked in a row of choices.
            .add_observer(radio_self_update)
            .add_observer(settings::pick::<DeckId>)
            .add_observer(settings::pick::<BackId>)
            .add_observer(settings::pick::<FeltColour>);
    }
}

/// Goes to the screen above this one on Esc, or Android's back gesture.
fn back(
    key: On<FocusedInput<KeyboardInput>>,
    windows: Query<(), With<Window>>,
    screen: Res<State<Screen>>,
    mut next: ResMut<NextState<Screen>>,
) {
    if let Some(parent) = screen.get().parent()
        && is_back(&key, &windows)
    {
        next.set(parent);
    }
}
