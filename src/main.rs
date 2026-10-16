// Hides the console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;

fn main() {
    shuftle_client_core::app(Window {
        title: "Shuftle".into(),
        name: Some("bevy.app".into()),
        // Tells Wasm not to override default event handling, like F5, Ctrl+R etc.
        prevent_default_event_handling: false,
        // On the web, follow the page's size, so the layout gets the real
        // screen of a phone's browser.
        fit_canvas_to_parent: true,
        // Below this the hand's cards get too small to read. Not on the web,
        // where the window is the page, and phone browsers are narrower.
        #[cfg(not(target_arch = "wasm32"))]
        resize_constraints: bevy::window::WindowResizeConstraints {
            min_width: 640.,
            min_height: 400.,
            ..default()
        },
        ..default()
    })
    .run();
}
