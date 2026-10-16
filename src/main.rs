// Hides the console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{diagnostic::FrameCount, prelude::*};

fn main() {
    let mut app = shuftle_client_core::app(Window {
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
        // This will spawn an invisible window
        // The window will be made visible in the make_visible() system after 3 frames.
        // This is useful when you want to avoid the white window that shows up before the GPU is ready to render the app.
        visible: false,
        ..default()
    });
    app.add_systems(Update, make_visible).run();
}

/// Shows the window once the GPU has had a few frames to render something.
fn make_visible(mut window: Single<&mut Window>, frames: Res<FrameCount>) {
    if frames.0 == 3 {
        window.visible = true;
    }
}
