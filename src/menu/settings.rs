//! What the player can change about the game.

use bevy::prelude::*;

use super::widgets::screen;
use crate::{state::Screen, theme::TEXT};

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
