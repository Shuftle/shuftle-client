use std::collections::HashMap;

use bevy::{
    ecs::system::SystemId,
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};
use shuftlib::{
    IntoEnumIterator,
    common::{
        cards::{Deck, ItalianRank, Suit},
        hands::{PlayerId, TrickTakingGame},
    },
    tressette::{CARDS_AT_TIME, TressetteCard, TressetteRules},
};

#[derive(States, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Setup,
    Play,
    CollectCards,
}

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum PlayerTurn {
    #[default]
    Player0,
    Player1,
    Player2,
    Player3,
}

pub struct GameLogic;

impl Plugin for GameLogic {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, init_scene)
            .add_systems(Update, anchor_players)
            .init_resource::<SetupGameId>()
            .init_state::<GameState>();
    }
}

fn init_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    setup_game_sys: Res<SetupGameId>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window.iter().next().unwrap();
    // Load Italian assets
    let mut italian_assets = ItalianAssets(Vec::with_capacity(4));
    for suit in Suit::iter() {
        let mut cards_in_suit = Vec::with_capacity(10);
        for rank in ItalianRank::iter() {
            let sprite_handle =
                asset_server.load(format!("cards/italian/card-{}-{}.png", suit, rank as u8));
            cards_in_suit.push(sprite_handle);
        }
        italian_assets.0.push(cards_in_suit);
    }
    commands.insert_resource(italian_assets);

    // Load card back.
    let sprite_handle = asset_server.load("cards/card-back1.png");
    commands.insert_resource(CardBack(sprite_handle));

    // Spawn main player.
    commands.spawn((
        Transform {
            translation: Vec3 {
                x: -window.width() * 0.18,
                y: -window.height() * 0.35,
                ..default()
            },
            ..default()
        },
        Player {
            id: PlayerId::new(0).unwrap(),
            cards_counter: 0,
        },
        InheritedVisibility::default(),
        GlobalTransform::default(),
    ));

    // Spawn player 2 (in front of the main player)
    commands.spawn((
        Transform {
            translation: Vec3 {
                x: window.width() * 0.18,
                y: window.height() * 0.35,
                ..default()
            },
            rotation: Quat::from_rotation_z((180f32).to_radians()),
            ..default()
        },
        Player {
            id: PlayerId::new(2).unwrap(),
            cards_counter: 0,
        },
    ));

    // Spawn player 1 (to the left of the main player)
    commands.spawn((
        Transform {
            translation: Vec3 {
                x: -window.width() * 0.44,
                y: window.height() * 0.3,
                ..default()
            },
            rotation: Quat::from_rotation_z((-90f32).to_radians()),
            ..default()
        },
        Player {
            id: PlayerId::new(1).unwrap(),
            cards_counter: 0,
        },
    ));

    // Spawn player 3 (to the right of the main player)
    commands.spawn((
        Transform {
            translation: Vec3 {
                x: window.width() * 0.44,
                y: -window.height() * 0.3,
                ..default()
            },
            rotation: Quat::from_rotation_z((90f32).to_radians()),
            ..default()
        },
        Player {
            id: PlayerId::new(3).unwrap(),
            cards_counter: 0,
        },
    ));

    // Create deck.
    let deck = Deck::italian().into();
    commands.insert_resource(DeckResource(deck));

    // Call system to setup game.
    commands.run_system(setup_game_sys.into_inner().0);
}

#[derive(Resource)]
struct SetupGameId(SystemId);

impl FromWorld for SetupGameId {
    fn from_world(world: &mut World) -> Self {
        let id = world.register_system(setup_game);
        SetupGameId(id)
    }
}

fn setup_game(
    mut commands: Commands,
    mut deck: ResMut<DeckResource>,
    mut next_state: ResMut<NextState<GameState>>,
    italian_assets: Res<ItalianAssets>,
    card_back: Res<CardBack>,
    mut query: Query<(Entity, &mut Player)>,
) {
    info!("Setting up game");
    // Shuffle deck.
    deck.0.shuffle();

    // Distribute cards.
    let mut players = HashMap::new();
    for (entity, player) in query.iter_mut() {
        players.insert(*player.id, (entity, player));
    }

    for _ in 0..2 {
        for i in 0..TressetteRules::PLAYERS {
            let cards = deck.0.draw_n(CARDS_AT_TIME);
            let (entity, player) = players.get_mut(&i).unwrap();
            if i == 0 {
                distribute_to_main(
                    &mut commands,
                    &italian_assets,
                    *entity,
                    cards,
                    &mut player.cards_counter,
                );
            } else {
                distribute_to_other(
                    &mut commands,
                    &card_back,
                    *entity,
                    cards,
                    &mut player.cards_counter,
                );
            }
        }
    }
    next_state.set(GameState::Play);

    // Start game loop.
}

fn distribute_to_main(
    commands: &mut Commands,
    italian_assets: &Res<ItalianAssets>,
    entity: Entity,
    cards: Vec<TressetteCard>,
    card_counter: &mut usize,
) {
    let cards_ids: Vec<_> = cards
        .iter()
        .map(|card| {
            const CARD_SPACING: f32 = 50.;
            let id = commands
                .spawn(Cardbundle {
                    card: Card(*card),
                    sprite: Sprite {
                        image: italian_assets.0[card.suit() as usize][card.rank() as usize - 1]
                            .clone(),
                        ..default()
                    },
                    transform: Transform {
                        translation: Vec3 {
                            x: CARD_SPACING * *card_counter as f32,
                            z: *card_counter as f32,
                            ..default()
                        },
                        ..default()
                    },
                    playable: Playable,
                    pickable: Pickable::default(),
                })
                .observe(select_play_card)
                .id();
            *card_counter += 1;
            id
        })
        .collect();
    commands.entity(entity).add_children(&cards_ids);
}

fn distribute_to_other(
    commands: &mut Commands,
    card_back: &Res<CardBack>,
    entity: Entity,
    cards: Vec<TressetteCard>,
    card_counter: &mut usize,
) {
    let cards_ids: Vec<_> = cards
        .iter()
        .map(|card| {
            const CARD_SPACING: f32 = 50.;
            let id = commands
                .spawn((
                    Card(*card),
                    Transform {
                        translation: Vec3 {
                            x: CARD_SPACING * *card_counter as f32,
                            z: *card_counter as f32,
                            ..default()
                        },
                        ..default()
                    },
                    Sprite {
                        image: card_back.0.clone(),
                        ..default()
                    },
                ))
                .id();
            *card_counter += 1;
            id
        })
        .collect();
    commands.entity(entity).add_children(&cards_ids);
}

fn select_play_card(
    mut trigger: On<Pointer<Click>>,
    mut selected_card_query: Query<
        (Entity, &mut Transform),
        (With<Card>, With<Playable>, With<Selected>),
    >,
    mut unselected_card_query: Query<(&mut Transform, &Card), (With<Playable>, Without<Selected>)>,
    mut commands: Commands,
) {
    trigger.propagate(false);
    let click_event = trigger.event();
    let clicked_card = click_event.entity;
    for (selected_entity, mut selected_transform) in selected_card_query.iter_mut() {
        if selected_entity != clicked_card {
            selected_transform.translation.y -= 50.;
            commands.entity(selected_entity).remove::<Selected>();
        } else {
            commands.entity(clicked_card).despawn();
        }
    }
    if let Ok((mut transform, _card)) = unselected_card_query.get_mut(clicked_card) {
        transform.translation.y += 50.;
        commands.entity(clicked_card).insert(Selected);
    }
}

fn anchor_players(
    window: Query<&Window, With<PrimaryWindow>>,
    mut players: Query<(&mut Transform, &Player)>,
    mut resize_reader: MessageReader<WindowResized>,
) {
    let window = window.iter().next().unwrap();

    for _event in resize_reader.read() {
        for (mut transform, player) in players.iter_mut() {
            match *player.id {
                0 => {
                    transform.translation.x = -window.width() * 0.18;
                    transform.translation.y = -window.height() * 0.35;
                }
                1 => {
                    transform.translation.x = -window.width() * 0.44;
                    transform.translation.y = window.height() * 0.3;
                }
                2 => {
                    transform.translation.x = window.width() * 0.18;
                    transform.translation.y = window.height() * 0.35;
                }
                3 => {
                    transform.translation.x = window.width() * 0.44;
                    transform.translation.y = -window.height() * 0.3;
                }
                _ => panic!("This cannot happen"),
            }
        }
    }
}

#[derive(Component, Default)]
pub struct Card(pub TressetteCard);

#[derive(Component, Default)]
struct Playable;

#[derive(Component, Default)]
struct Selected;

#[derive(Bundle, Default)]
struct Cardbundle {
    pub transform: Transform,
    pub card: Card,
    pub sprite: Sprite,
    pub playable: Playable,
    pub pickable: Pickable,
}

#[derive(Component)]
struct Player {
    id: PlayerId<{ TressetteRules::PLAYERS }>,
    cards_counter: usize,
}

#[derive(Resource)]
pub struct ItalianAssets(pub Vec<Vec<Handle<Image>>>);

#[derive(Resource)]
pub struct CardBack(pub Handle<Image>);

#[derive(Resource)]
struct DeckResource(Deck<TressetteCard>);
