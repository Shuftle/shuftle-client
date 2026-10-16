use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
    }
}

/// A 2D camera where a world unit is a logical pixel, so the table layout can
/// size things for the actual screen.
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
