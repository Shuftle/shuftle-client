use bevy::prelude::*;
use shuftle_client_core::{camera::CameraPlugin, game_logic::GameLogic};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Shuftle".into(),
                name: Some("bevy.app".into()),
                resolution: (1280, 720).into(),
                // Tells Wasm to resize the window according to the available canvas
                fit_canvas_to_parent: true,
                // Tells Wasm not to override default event handling, like F5, Ctrl+R etc.
                prevent_default_event_handling: false,
                enabled_buttons: bevy::window::EnabledButtons {
                    maximize: false,
                    ..Default::default()
                },
                // This will spawn an invisible window
                // The window will be made visible in the make_visible() system after 3 frames.
                // This is useful when you want to avoid the white window that shows up before the GPU is ready to render the app.
                visible: false,
                ..default()
            }),
            ..default()
        }),))
        .add_plugins(GameLogic)
        .add_plugins(CameraPlugin)
        .run();
}
