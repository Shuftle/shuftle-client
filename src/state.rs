//! Where the app is: in the menus, or at the table, and how the player goes
//! back.

use bevy::{
    input::keyboard::{Key, KeyboardInput},
    input_focus::FocusedInput,
    prelude::*,
};

/// Whether the player is in the menus, waiting for a game to load, or playing.
#[derive(States, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    Menu,
    Loading,
    InGame,
}

/// The menu screen shown, only while in the menus.
#[derive(SubStates, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[source(AppState = AppState::Menu)]
pub enum Screen {
    #[default]
    Main,
    Settings,
    About,
}

impl Screen {
    /// Returns the screen going back leads to, if any.
    pub fn parent(self) -> Option<Screen> {
        match self {
            Screen::Main => None,
            Screen::Settings | Screen::About => Some(Screen::Main),
        }
    }
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>().add_sub_state::<Screen>();
    }
}

/// Returns whether `key` asks to go back: Esc, or Android's back gesture, once
/// it reaches the window, so nothing focused on the way kept it.
pub fn is_back(key: &On<FocusedInput<KeyboardInput>>, windows: &Query<(), With<Window>>) -> bool {
    let input = &key.input;
    windows.contains(key.event_target())
        && input.state.is_pressed()
        && !input.repeat
        && matches!(input.logical_key, Key::Escape | Key::BrowserBack)
}
