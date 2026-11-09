//! Shown while a game's images load.

use bevy::prelude::*;

use crate::{
    state::AppState,
    ui::{
        theme::{FONT_HEADING, TEXT},
        widgets::full_screen,
    },
};

pub fn scene() -> impl Scene {
    let scoped = DespawnOnExit(AppState::Loading);
    bsn! {
        @full_screen()
        Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        scoped
        Children [
            Text("Loading…")
            TextFont { font_size: FontSize::Px(FONT_HEADING) }
            TextColor(TEXT)
        ]
    }
}
