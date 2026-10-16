// Hides the console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;

fn main() {
    shuftle_client_core::app(Window {
        title: "Shuftle".into(),
        name: Some("bevy.app".into()),
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
    })
    .run();
}
