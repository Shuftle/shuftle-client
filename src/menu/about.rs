//! What the game is, and who made it.

use bevy::prelude::*;

use super::widgets::screen;
use crate::{state::Screen, theme::TEXT};

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
