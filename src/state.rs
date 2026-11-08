//! Where the app is: in the menus, or at the table, and how the player goes
//! back.

use bevy::{
    input::keyboard::{Key, KeyboardInput},
    input_focus::FocusedInput,
    prelude::*,
};

use crate::catalog::Game;

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
    /// The list of games to pick from.
    Games,
    /// How a game is played.
    Rules(Game),
    Settings,
    About,
}

/// Whether the player is asked to confirm leaving the game, only while
/// playing.
#[derive(SubStates, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[source(AppState = AppState::InGame)]
pub enum LeaveDialog {
    #[default]
    Closed,
    Open,
}

impl Screen {
    /// Returns the screen going back leads to, if any.
    pub fn parent(self) -> Option<Screen> {
        match self {
            Screen::Main => None,
            Screen::Games | Screen::Settings | Screen::About => Some(Screen::Main),
            Screen::Rules(_) => Some(Screen::Games),
        }
    }
}

pub struct StatePlugin;

/// A game, from loading to its end.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GameSession;

impl ComputedStates for GameSession {
    type SourceStates = AppState;
    // No exit and enter between loading and playing.
    const ALLOW_SAME_STATE_TRANSITIONS: bool = false;

    fn compute(state: AppState) -> Option<Self> {
        matches!(state, AppState::Loading | AppState::InGame).then_some(Self)
    }
}

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_sub_state::<Screen>()
            .add_sub_state::<LeaveDialog>()
            .add_computed_state::<GameSession>();
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
