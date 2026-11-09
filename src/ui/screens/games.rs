//! The list of games to pick from.

use bevy::{prelude::*, ui_widgets::Activate};

use super::go_to;
use crate::{
    catalog::Game,
    state::{AppState, Screen},
    ui::{
        theme::{BUTTON_MAX_WIDTH, FULL, ITEM_GAP},
        widgets::{icon_button, menu_button, screen},
    },
};

/// A row per game: its name, which starts it, and a button beside it opening
/// its rules.
pub fn scene() -> impl Scene {
    let rows: Vec<_> = Game::ALL
        .into_iter()
        .map(|game| {
            bsn! {
                Node {
                    width: FULL,
                    max_width: px(BUTTON_MAX_WIDTH),
                    align_items: AlignItems::Center,
                    column_gap: px(ITEM_GAP),
                }
                Children [
                    @menu_button(game.name()) on(start)
                    --
                    @icon_button("?") on(go_to(Screen::Rules(game)))
                ]
            }
        })
        .collect();
    screen(Screen::Games, "Games", bsn_list! { {rows} })
}

fn start(_: On<Activate>, mut next: ResMut<NextState<AppState>>) {
    next.set(AppState::Loading);
}
