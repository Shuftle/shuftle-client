use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin, prelude::*, sprite::MaterialMesh2dBundle,
    window::PrimaryWindow,
};
use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, spawn_hand)
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

fn spawn_hand(mut commands: Commands, asset_server: Res<AssetServer>) {
    (0..10).into_iter().for_each(|_| {
        commands.spawn((
            Cardbundle::default().with_sprite(SpriteBundle {
                texture: asset_server.load("cards/card-clubs-1.png"),
                ..default()
            }),
            // Disable picking
            On::<Pointer<DragStart>>::target_insert(Pickable::IGNORE),
            // Re-enable picking
            On::<Pointer<DragEnd>>::target_insert(Pickable::default()),
            On::<Pointer<Drag>>::target_component_mut::<Transform>(|drag, transform| {
                // Make the square follow the mouse
                transform.translation.x += drag.delta.x;
                transform.translation.y -= drag.delta.y;
            }),
        ));
    })
}

fn drag_drop(
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if let Some(cursor_position) = cursor_world_position(camera_query, window_query) {}
}

fn cursor_world_position(
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) -> Option<Vec2> {
    let (camera, camera_transform) = camera_query.single();
    let window = window_query.single();

    window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
}

#[derive(Component, Default)]
struct MainCamera;

#[derive(Component, Default)]
struct Card;

#[derive(Component, Default)]
struct Draggable;

#[derive(Component, Default)]
struct Hoverable;

#[derive(Resource, Default)]
struct MousePosition(Vec2);

#[derive(Event, Default)]
struct StartHovering;

#[derive(Event, Default)]
struct StopHovering;

#[derive(Event, Default)]
struct StartDragging;

#[derive(Event, Default)]
struct StopDragging;

#[derive(Bundle, Default)]
struct Cardbundle {
    draggable: PickableBundle,
    hoverable: Hoverable,
    card: Card,
    sprite_bundle: SpriteBundle,
}

impl Cardbundle {
    fn with_sprite(mut self, sprite_bundle: SpriteBundle) -> Cardbundle {
        self.sprite_bundle = sprite_bundle;
        self
    }
}
