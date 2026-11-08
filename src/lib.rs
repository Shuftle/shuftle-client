pub mod assets;
pub mod camera;
pub mod catalog;
pub mod game;
pub mod settings;
pub mod state;
pub mod ui;

use std::time::Duration;

use assets::LoadingPlugin;
use bevy::{
    prelude::*,
    settings::SettingsPlugin,
    winit::{UpdateMode, WinitSettings},
};
use camera::CameraPlugin;
use game::{layout::LayoutPlugin, logic::GameLogic, table::TablePlugin};
use settings::{APP_NAME, SavingPlugin};
use state::StatePlugin;
use ui::{hud::HudPlugin, screens::MenuPlugin};

/// The most time a frame moves the game on by. The game's clock comes from the
/// renderer, so after the game sat idle the wait can show up a frame or two
/// late, and would make whatever started moving meanwhile jump.
const MAX_STEP: Duration = Duration::from_millis(50);

/// The longest the game goes without drawing a frame while it waits on the
/// player.
const IDLE_WAKE: Duration = Duration::from_secs(60);

/// Draws a frame only on taps, clicks and resizes, or when something asks for
/// one with `RequestRedraw`, so phones don't heat up while the game waits on
/// the player. Unlike `UpdateMode::reactive`, the mouse moving outside the
/// window doesn't count.
fn power_saving() -> WinitSettings {
    let mode = UpdateMode::reactive_low_power(IDLE_WAKE);
    WinitSettings {
        focused_mode: mode,
        unfocused_mode: mode,
    }
}

/// Builds the game with `window` as its primary window, ready to `run`.
pub fn app(window: Window) -> App {
    let mut app = App::new();
    app.add_plugins(
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
    );
    // Loads the settings, before the plugins that read them.
    settings::register(&mut app);
    app.add_plugins(SettingsPlugin::new(APP_NAME));
    app.add_plugins((
        SavingPlugin,
        StatePlugin,
        LayoutPlugin,
        MenuPlugin,
        LoadingPlugin,
        TablePlugin,
        GameLogic,
        HudPlugin,
        CameraPlugin,
    ))
    .insert_resource(power_saving())
    .insert_resource(Time::<Virtual>::from_max_delta(MAX_STEP));
    app
}
