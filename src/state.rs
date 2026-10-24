//! Where the app is: in the menus, or at the table.

use bevy::prelude::*;

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
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>().add_sub_state::<Screen>();
    }
}
