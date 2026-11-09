//! What the game is, and who made it.

use bevy::prelude::*;

use crate::{
    state::Screen,
    ui::{theme::TEXT, widgets::screen},
};

pub fn scene() -> impl Scene {
    screen(
        Screen::About,
        "About",
        bsn_list! {
            Text("Shuftle")
            TextColor(TEXT)
        },
    )
}
