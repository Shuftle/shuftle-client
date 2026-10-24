pub mod camera;
pub mod game_logic;
pub mod layout;
pub mod loading;
pub mod menu;
pub mod state;
pub mod table;
pub mod theme;

use std::time::Duration;

use bevy::{
    prelude::*,
    winit::{UpdateMode, WinitSettings},
};
use camera::CameraPlugin;
use game_logic::GameLogic;
use layout::LayoutPlugin;
use loading::LoadingPlugin;
use menu::MenuPlugin;
use state::StatePlugin;
use table::TablePlugin;

/// The most time a frame moves the game on by. The game's clock comes from the
/// renderer, so after the game sat idle the wait can show up a frame or two
/// late, and would make whatever started moving meanwhile jump.
const MAX_STEP: Duration = Duration::from_millis(50);

/// Draws a frame only on taps, clicks and resizes, or when something asks for
/// one with `RequestRedraw`, so phones don't heat up while the game waits on
/// the player. Unlike `UpdateMode::reactive`, the mouse moving outside the
/// window doesn't count.
fn power_saving() -> WinitSettings {
    let mode = UpdateMode::reactive_low_power(Duration::from_secs(60));
    WinitSettings {
        focused_mode: mode,
        unfocused_mode: mode,
    }
}

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
        StatePlugin,
        LayoutPlugin,
        MenuPlugin,
        LoadingPlugin,
        TablePlugin,
        GameLogic,
        CameraPlugin,
    ))
    .insert_resource(power_saving())
    .insert_resource(Time::<Virtual>::from_max_delta(MAX_STEP));
    app
}
