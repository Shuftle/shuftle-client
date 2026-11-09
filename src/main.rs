// Hides the console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;

/// The smallest the window gets: below this the hand's cards get too small to
/// read.
#[cfg(not(target_arch = "wasm32"))]
const MIN_WINDOW: Vec2 = Vec2::new(640., 400.);

fn main() {
    shuftle_client_core::app(Window {
        title: "Shuftle".into(),
        name: Some("bevy.app".into()),
        // Tells Wasm not to override default event handling, like F5, Ctrl+R etc.
        prevent_default_event_handling: false,
        // On the web, follow the page's size, so the layout gets the real
        // screen of a phone's browser.
        fit_canvas_to_parent: true,
        // Not on the web, where the window is the page, and phone browsers are
        // narrower.
        #[cfg(not(target_arch = "wasm32"))]
        resize_constraints: bevy::window::WindowResizeConstraints {
            min_width: MIN_WINDOW.x,
            min_height: MIN_WINDOW.y,
            ..default()
        },
        ..default()
    })
    .run();
}
