use bevy::prelude::*;
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
    let hand_id = commands
        .spawn((
            Hand,
            SpriteBundle {
                transform: Transform {
                    translation: Vec3 {
                        x: -450.,
                        y: -250.,
                        ..default()
                    },
                    ..default()
                },
                ..default()
            },
        ))
        .id();
    let cards_ids: Vec<_> = (0..10)
        .map(|i| {
            commands
                .spawn((
                    Cardbundle::default().with_sprite(SpriteBundle {
                        texture: asset_server.load("cards/card-clubs-1.png"),
                        transform: Transform {
                            translation: Vec3 {
                                x: 100. * i as f32,
                                ..default()
                            },
                            ..default()
                        },
                        ..default()
                    }),
                    On::<Pointer<Click>>::run(select_play_card),
                ))
                .id()
        })
        .collect();
    commands.entity(hand_id).push_children(&cards_ids);
}

fn select_play_card(
    click: Listener<Pointer<Click>>,
    mut selected_card_query: Query<
        (Entity, &mut Transform),
        (With<Card>, With<Playable>, With<Selected>),
    >,
    mut unselected_card_query: Query<
        &mut Transform,
        (With<Card>, With<Playable>, Without<Selected>),
    >,
    mut commands: Commands,
) {
    let clicked_card = click.target;
    for (selected_entity, mut selected_transform) in selected_card_query.iter_mut() {
        if selected_entity != clicked_card {
            selected_transform.translation.y -= 50.;
            commands.entity(selected_entity).remove::<Selected>();
        } else {
            info!("Card was played");
            commands.entity(clicked_card).despawn();
        }
    }
    if let Ok(mut transform) = unselected_card_query.get_mut(clicked_card) {
        transform.translation.y += 50.;
        commands.entity(clicked_card).insert(Selected);
    }
}

#[derive(Component, Default)]
struct MainCamera;

#[derive(Component, Default)]
struct Card;

#[derive(Component, Default)]
struct Hand;

#[derive(Component, Default)]
struct Playable;

#[derive(Component, Default)]
struct Selected;

#[derive(Bundle, Default)]
struct Cardbundle {
    card: Card,
    sprite_bundle: SpriteBundle,
    playable: Playable,
}

impl Cardbundle {
    fn with_sprite(mut self, sprite_bundle: SpriteBundle) -> Cardbundle {
        self.sprite_bundle = sprite_bundle;
        self
    }
}
