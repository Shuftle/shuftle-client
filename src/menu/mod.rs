//! The menus shown before and between games.

mod main_menu;

use bevy::prelude::*;

use crate::state::Screen;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Main), main_menu::scene.spawn());
    }
}
