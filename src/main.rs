use bevy::prelude::*;
use shuftle_client_core::{
    camera::CameraPlugin, load_assets::LoadAssets, main_player::MainPlayer,
    other_player::OtherPlayer,
};

fn main() {
    App::new()
        .add_plugins(LoadAssets)
        .add_plugins(MainPlayer)
        .add_plugins(OtherPlayer)
        .add_plugins(CameraPlugin)
        .add_plugins(DefaultPlugins)
        .run();
}
