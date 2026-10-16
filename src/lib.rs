pub mod camera;
pub mod game_logic;

use bevy::prelude::*;
use camera::CameraPlugin;
use game_logic::GameLogic;

/// Builds the game with `window` as its primary window, ready to `run`.
pub fn app(window: Window) -> App {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(window),
                ..default()
            })
            .set(AssetPlugin {
                #[cfg(target_arch = "wasm32")]
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            }),
        GameLogic,
        CameraPlugin,
    ));
    app
}
