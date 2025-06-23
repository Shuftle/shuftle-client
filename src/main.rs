use bevy::{
    dev_tools::picking_debug::{DebugPickingMode, DebugPickingPlugin},
    prelude::*,
};
use shuftle_client_core::{camera::CameraPlugin, game_logic::GameLogic};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GameLogic)
        .add_plugins(CameraPlugin)
        .add_plugins(DebugPickingPlugin)
        .insert_resource(DebugPickingMode::Normal)
        .run();
}
