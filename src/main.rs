use bevy::prelude::*;
use shuftle_client_core::{camera::CameraPlugin, game_logic::GameLogic};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GameLogic)
        .add_plugins(CameraPlugin)
        .run();
}
