//! How a game is played.

use bevy::prelude::*;

use crate::{
    catalog::Game,
    state::Screen,
    ui::{
        theme::{FONT_SMALL, FULL, TEXT},
        widgets::screen,
    },
};

pub fn scene(game: Game) -> impl Scene {
    screen(
        Screen::Rules(game),
        game.name(),
        bsn_list! {
            Text({game.rules()})
            TextFont { font_size: FontSize::Px(FONT_SMALL) }
            TextColor(TEXT)
            Node { width: FULL }
        },
    )
}
