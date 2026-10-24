//! The menus shown before and between games.

mod about;
mod main_menu;
mod settings;
pub mod widgets;

use bevy::{input::keyboard::KeyboardInput, input_focus::FocusedInput, prelude::*};

use crate::state::{AppState, Screen, is_back};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Main), main_menu::scene.spawn())
            .add_systems(OnEnter(Screen::Settings), settings::scene.spawn())
            .add_systems(OnEnter(Screen::About), about::scene.spawn())
            .add_observer(back.run_if(in_state(AppState::Menu)))
            .add_observer(widgets::button_over)
            .add_observer(widgets::button_out)
            .add_observer(widgets::button_press)
            .add_observer(widgets::button_release);
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
