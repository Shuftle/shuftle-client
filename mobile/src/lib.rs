use bevy::{prelude::*, window::WindowMode};

// Generates the `android_main` entry point the activity calls.
#[bevy_main]
fn main() {
    shuftle_client_core::app(Window {
        mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
        resizable: false,
        ..default()
    })
    .run();
}
