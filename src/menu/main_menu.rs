//! The first screen: the game's name, and what the player can do from there.

use bevy::{prelude::*, ui_widgets::Activate};

use super::widgets::{equal_share, full_screen, menu_button, screen_padding};
use crate::{
    layout::Orient,
    state::{AppState, Screen},
    theme::{FONT_TITLE, ITEM_GAP, TEXT},
};

/// Whether the platform lets apps quit: phones and browsers leave that to
/// the system.
const CAN_QUIT: bool = !cfg!(any(
    target_arch = "wasm32",
    target_os = "android",
    target_os = "ios"
));

/// Side by side on wide screens, the name above the buttons on tall ones,
/// where the buttons sit lower, in reach of the thumb. Each half takes the
/// same room either way.
pub fn scene() -> impl Scene {
    let scoped = DespawnOnExit(Screen::Main);
    bsn! {
        @full_screen()
        Node {
            padding: {screen_padding()},
        }
        Orient { portrait: FlexDirection::Column, landscape: FlexDirection::Row }
        scoped
        Children [
            @equal_share()
            Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
            }
            Children [
                Text("Shuftle")
                TextFont { font_size: FontSize::Px(FONT_TITLE) }
                TextColor(TEXT)
            ]
            --
            @equal_share()
            Node {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(ITEM_GAP),
            }
            Children [
                @menu_button("Play") on(play)
                --
                @menu_button("Settings") on(go_to(Screen::Settings))
                --
                @menu_button("About") on(go_to(Screen::About))
                --
                {CAN_QUIT.then(|| bsn_list! { @menu_button("Quit") on(quit) })}
            ]
        ]
    }
}

fn play(_: On<Activate>, mut next: ResMut<NextState<AppState>>) {
    next.set(AppState::Loading);
}

/// Returns what opens `screen` when a button is activated.
fn go_to(screen: Screen) -> impl Fn(On<Activate>, ResMut<NextState<Screen>>) + Clone {
    move |_, mut next| next.set(screen)
}

fn quit(_: On<Activate>, mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}
