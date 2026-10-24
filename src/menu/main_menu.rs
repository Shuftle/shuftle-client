//! The first screen: the game's name, and what the player can do from there.

use bevy::{
    prelude::*,
    ui_widgets::{Activate, Button},
};

use crate::state::{AppState, Screen};

pub fn scene() -> impl Scene {
    let scoped = DespawnOnExit(Screen::Main);
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: px(32),
        }
        scoped
        Children [
            Text("Shuftle")
            TextFont { font_size: FontSize::Px(64.) }
            --
            Button
            Node {
                padding: {UiRect::axes(px(48), px(12))},
            }
            BackgroundColor({Color::srgb(0.5, 0.5, 0.5)})
            on(play)
            Children [
                Text("Play")
                TextFont { font_size: FontSize::Px(32.) }
            ]
        ]
    }
}

fn play(_: On<Activate>, mut next: ResMut<NextState<AppState>>) {
    next.set(AppState::Loading);
}
