//! What the player can change about the game.

use bevy::prelude::*;

use crate::{
    state::Screen,
    ui::{theme::TEXT, widgets::screen},
};

pub fn scene() -> impl Scene {
    screen(
        Screen::Settings,
        "Settings",
        bsn_list! {
            Text("Nothing to set yet.")
            TextColor(TEXT)
        },
    )
}
