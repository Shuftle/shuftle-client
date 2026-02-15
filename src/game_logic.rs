use std::collections::HashMap;

use bevy::{
    ecs::schedule::common_conditions::any_with_component,
    ecs::system::SystemId,
    prelude::*,
    text::{Font, Justify, TextFont, TextLayout},
    ui::{Interaction, Node, PositionType, Val},
    window::PrimaryWindow,
};
use shuftlib::{
    core::{Suit, italian::ItalianRank},
    tressette::{Game, TressetteCard},
    trick_taking::{PLAYERS, PlayerId},
};
use strum::IntoEnumIterator;

#[derive(Resource)]
struct GameState(Game);

#[derive(Resource, Default)]
struct FontHandle(Handle<Font>);

// Positions for played cards in the trick (center of table, clockwise diamond)
const TRICK_POSITIONS: [(f32, f32); 4] = [
    (0.0, -50.0), // Player 0 (bottom)
    (50.0, 0.0),  // Player 1 (right)
    (0.0, 50.0),  // Player 2 (top)
    (-50.0, 0.0), // Player 3 (left)
];

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
            .add_systems(PostStartup, remove_name_from_text.after(init_scene))
            .add_systems(
                Update,
                (
                    update_player_positions,
                    move_to_target.run_if(any_with_component::<MovingTo>),
                    handle_restart_button,
                ),
            )
            .add_systems(Last, despawn_marked)
            .init_resource::<SetupGameId>()
            .init_resource::<NonPovPlayId>()
            .init_resource::<HandleEffectId>()
            .init_resource::<CollectCardsId>()
            .insert_resource(GameState(Game::new()));
    }
}

/// System called at the beginning of the game to load assets and spawn players.
fn init_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    setup_game_sys: Res<SetupGameId>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window.iter().next().unwrap();

    // Load default font
    let font_handle: Handle<Font> = Default::default();
    commands.insert_resource(FontHandle(font_handle.clone()));

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

    // Spawn POV player
    commands
        .spawn((
            Name::new("Player 0"),
            Transform {
                translation: Vec3 {
                    x: -window.width() * 0.18,
                    y: -window.height() * 0.35,
                    ..default()
                },
                ..default()
            },
            Player {
                id: PlayerId::PLAYER_0,
                cards_counter: 0,
            },
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text2d("Player 0".to_owned()),
                TextLayout::new_with_justify(Justify::Center),
                TextFont {
                    font: font_handle.clone(),
                    font_size: 24.0,
                    ..default()
                },
                Transform::from_xyz(0.0, 100.0, 1.0),
            ));
        });

    // Spawn player 1
    commands
        .spawn((
            Name::new("Player 1"),
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
                id: PlayerId::PLAYER_1,
                cards_counter: 0,
            },
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text2d("Player 1".to_owned()),
                TextLayout::new_with_justify(Justify::Center),
                TextFont {
                    font: font_handle.clone(),
                    font_size: 24.0,
                    ..default()
                },
                Transform {
                    translation: Vec3::new(-70.0, 60.0, 1.0),
                    rotation: Quat::from_rotation_z(-90f32.to_radians()),
                    ..default()
                },
            ));
        });

    // Spawn player 2
    commands
        .spawn((
            Name::new("Player 2"),
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
                id: PlayerId::PLAYER_2,
                cards_counter: 0,
            },
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text2d("Player 2".to_owned()),
                TextLayout::new_with_justify(Justify::Center),
                TextFont {
                    font: font_handle.clone(),
                    font_size: 24.0,
                    ..default()
                },
                Transform {
                    translation: Vec3::new(0.0, 100.0, 1.0),
                    rotation: Quat::from_rotation_z(180f32.to_radians()),
                    ..default()
                },
            ));
        });

    // Spawn player 3
    commands
        .spawn((
            Name::new("Player 3"),
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
                id: PlayerId::PLAYER_3,
                cards_counter: 0,
            },
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text2d("Player 3".to_owned()),
                TextLayout::new_with_justify(Justify::Center),
                TextFont {
                    font: font_handle.clone(),
                    font_size: 24.0,
                    ..default()
                },
                Transform {
                    translation: Vec3::new(-70.0, 60.0, 1.0),
                    rotation: Quat::from_rotation_z(90f32.to_radians()),
                    ..default()
                },
            ));
        });

    // Spawn score display
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        Text::new("Score: 0 - 0"),
        TextFont {
            font: font_handle.clone(),
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
        ScoreText,
    ));

    // Call system to setup game.
    commands.run_system(setup_game_sys.0);
}

/// System called every frame to position players at screen edges.
fn update_player_positions(
    window: Query<&Window, With<PrimaryWindow>>,
    mut players: Query<(&mut Transform, &Player)>,
) {
    let window = window.iter().next().unwrap();

    for (mut transform, player) in players.iter_mut() {
        match player.id.as_usize() {
            0 => {
                transform.translation.x = -window.width() * 0.18;
                transform.translation.y = -window.height() * 0.35;
            }
            1 => {
                transform.translation.x = window.width() * 0.44;
                transform.translation.y = -window.height() * 0.3;
            }
            2 => {
                transform.translation.x = window.width() * 0.18;
                transform.translation.y = window.height() * 0.35;
            }
            3 => {
                transform.translation.x = -window.width() * 0.44;
                transform.translation.y = window.height() * 0.3;
            }
            _ => panic!("This cannot happen"),
        }
    }
}

#[derive(Resource)]
struct SetupGameId(SystemId);
impl FromWorld for SetupGameId {
    fn from_world(world: &mut World) -> Self {
        let id = world.register_system(setup_game);
        SetupGameId(id)
    }
}
/// One shot system that gets called after initial setup is done and every time the game has to be started.
fn setup_game(
    mut commands: Commands,
    game: Res<GameState>,
    italian_assets: Res<ItalianAssets>,
    card_back: Res<CardBack>,
    mut query: Query<(Entity, &mut Player)>,
    non_pov_play_id: Res<NonPovPlayId>,
) {
    info!("Setting up game");

    // Distribute cards from Game hands.
    let mut players: HashMap<usize, _> = HashMap::new();
    for (entity, mut player) in query.iter_mut() {
        player.cards_counter = 0;
        players.insert(player.id.as_usize(), (entity, player));
    }

    for i in 0..PLAYERS {
        let mut cards: Vec<TressetteCard> = game.0.hand(PlayerId::try_from(i).unwrap()).to_vec();
        if i == 0 {
            cards.sort_by(|a, b| (a.suit() as u8).cmp(&(b.suit() as u8)).then(a.cmp(b)));
        }
        let (entity, player) = players.get_mut(&i).unwrap();
        if i == 0 {
            distribute_to_pov(
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

    if game.0.current_player() != PlayerId::PLAYER_0 {
        commands.run_system(non_pov_play_id.0);
    }
}

/// Spawns card entities for the POV player.
fn distribute_to_pov(
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

/// Spawn card entities for non POV players.
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

/// This is called when the POV player clicks on one of their cards.
fn select_play_card(
    click: On<Pointer<Click>>,
    mut game: ResMut<GameState>,
    mut selected_card_query: Query<
        (Entity, &mut Transform, &Card),
        (With<Card>, With<Playable>, With<Selected>),
    >,

    handle_effect_id: Res<HandleEffectId>,
    mut unselected_card_query: Query<(&mut Transform, &Card), (With<Playable>, Without<Selected>)>,
    moving_query: Query<(), With<MovingTo>>,
    mut commands: Commands,
) {
    // Only allow playing if it's Player 0's turn
    if game.0.current_player() != PlayerId::PLAYER_0 {
        info!("Not Player's turn");
        return;
    }

    // Prevent playing while animations are ongoing
    if !moving_query.is_empty() {
        info!("Cannot play card while animations are in progress");
        return;
    }

    let click_event = click.event();
    let clicked_card = click_event.entity;

    for (selected_entity, mut selected_transform, card) in selected_card_query.iter_mut() {
        if selected_entity != clicked_card {
            selected_transform.translation.y -= 50.;
            commands.entity(selected_entity).remove::<Selected>();
        } else {
            // Play the card
            let num_played = game
                .0
                .current_trick()
                .iter()
                .filter(|c| c.is_some())
                .count();
            let player_index = (game.0.trick_leader().as_usize() + num_played) % 4;
            match game.0.play_card(card.0) {
                Ok(effect) => {
                    info!("Played card: {:?}, effect: {:?}", card.0, effect);
                    // Move to trick position
                    let (x, y) = TRICK_POSITIONS[player_index];
                    commands
                        .entity(clicked_card)
                        .remove::<Playable>()
                        .remove::<Selected>()
                        .insert(CardInPlay)
                        .remove_parent_in_place()
                        .insert(MovingTo {
                            target: Vec3::new(x, y, 10.0),
                            speed: 200.0,
                            on_arrival: Some(handle_effect_id.0),
                        });
                }
                Err(e) => {
                    warn!("Invalid play: {:?}", e);
                }
            }
        }
    }
    if let Ok((mut transform, _card)) = unselected_card_query.get_mut(clicked_card) {
        transform.translation.y += 50.;
        commands.entity(clicked_card).insert(Selected);
    }
}

fn move_to_target(
    mut query: Query<(Entity, &mut Transform, &MovingTo)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut transform, moving) in query.iter_mut() {
        let direction = moving.target - transform.translation;
        let distance = direction.length();
        let move_amount = moving.speed * time.delta_secs();

        if move_amount >= distance {
            // Snap to target
            transform.translation = moving.target;

            // Run the callback system if provided
            if let Some(system_id) = moving.on_arrival {
                commands.run_system(system_id);
            }

            // Remove the component
            commands.entity(entity).remove::<MovingTo>();
        } else {
            // Continue moving
            transform.translation += direction.normalize() * move_amount;
        }
    }
}

#[derive(Resource)]
struct HandleEffectId(SystemId);
impl FromWorld for HandleEffectId {
    fn from_world(world: &mut World) -> Self {
        let id = world.register_system(handle_effect);
        HandleEffectId(id)
    }
}
fn handle_effect(
    non_pov_play_id: Res<NonPovPlayId>,
    setup_game_id: Res<SetupGameId>,
    collect_cards_id: Res<CollectCardsId>,
    font: Res<FontHandle>,
    mut commands: Commands,
    game: Res<GameState>,
    mut score_text_query: Query<&mut Text, With<ScoreText>>,
) {
    let effect = game.0.history().last().unwrap().1;
    match effect {
        shuftlib::tressette::MoveEffect::CardPlayed => {
            info!("Handling card played");
            if game.0.current_player() != PlayerId::PLAYER_0 {
                commands.run_system(non_pov_play_id.0)
            }
        }
        shuftlib::tressette::MoveEffect::TrickCompleted { winner } => {
            info!("Handling card played");
            commands.run_system(collect_cards_id.0);
            if winner != PlayerId::PLAYER_0 {
                commands.run_system(non_pov_play_id.0)
            }
        }
        shuftlib::tressette::MoveEffect::HandComplete {
            trick_winner: _,
            score,
        } => {
            info!("Handling hand complete");
            if let Ok(mut text) = score_text_query.single_mut() {
                *text = Text::new(format!("Score: {} - {}", score.0, score.1));
            }
            commands.run_system(collect_cards_id.0);
            commands.run_system(setup_game_id.0);
        }
        shuftlib::tressette::MoveEffect::GameOver {
            trick_winner: _,
            final_score,
        } => {
            info!("Handling game over");
            commands.run_system(collect_cards_id.0);
            if let Ok(mut text) = score_text_query.single_mut() {
                *text = Text::new(format!(
                    "Final Score: {} - {}",
                    final_score.0, final_score.1
                ));
            }
            // Spawn restart button
            commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(50.0),
                        left: Val::Px(10.0),
                        ..default()
                    },
                    Interaction::None,
                    BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
                    RestartButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Restart Game"),
                        TextFont {
                            font: font.0.clone(),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        }
    }
}

#[derive(Component)]
struct ToDespawn;

#[derive(Component)]
struct RestartButton;

#[derive(Resource)]
struct CollectCardsId(SystemId);
impl FromWorld for CollectCardsId {
    fn from_world(world: &mut World) -> Self {
        let id = world.register_system(mark_cards_for_despawn);
        CollectCardsId(id)
    }
}
fn mark_cards_for_despawn(mut query: Query<Entity, With<CardInPlay>>, mut commands: Commands) {
    info!("Marking cards for despawn");
    for card in query.iter_mut() {
        commands.entity(card).insert(ToDespawn);
    }
}

fn despawn_marked(mut commands: Commands, query: Query<Entity, With<ToDespawn>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn handle_restart_button(
    interaction_query: Query<(Entity, &Interaction), (Changed<Interaction>, With<RestartButton>)>,
    setup_game_id: Res<SetupGameId>,
    mut game: ResMut<GameState>,
    mut score_text_query: Query<&mut Text, With<ScoreText>>,
    card_query: Query<Entity, With<Card>>,
    mut commands: Commands,
) {
    for (entity, interaction) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Mark all cards for despawn
            for card_entity in card_query.iter() {
                commands.entity(card_entity).insert(ToDespawn);
            }
            // Reset game state
            *game = GameState(Game::new());
            // Update score text
            if let Ok(mut text) = score_text_query.single_mut() {
                *text = Text::new("Score: 0 - 0");
            }
            // Despawn the button
            commands.entity(entity).despawn();
            // Restart game
            commands.run_system(setup_game_id.0);
        }
    }
}

#[derive(Resource)]
struct NonPovPlayId(SystemId);
impl FromWorld for NonPovPlayId {
    fn from_world(world: &mut World) -> Self {
        let id = world.register_system(non_pov_play);
        NonPovPlayId(id)
    }
}
/// One shot system called for non POV players.
fn non_pov_play(
    mut game: ResMut<GameState>,
    mut commands: Commands,
    handle_effect_id: Res<HandleEffectId>,
    italian_assets: Res<ItalianAssets>,
    mut query: Query<(Entity, &mut Sprite, &Card)>,
) {
    if game.0.current_player() == PlayerId::PLAYER_0 {
        error!("It's the POV player's turn. This shouldn't have happened");
        return;
    }

    let legal_cards = game.0.legal_cards();
    if let Some(card) = legal_cards.first() {
        let num_played = game
            .0
            .current_trick()
            .iter()
            .filter(|c| c.is_some())
            .count();
        let player_index = (game.0.trick_leader().as_usize() + num_played) % 4;
        match game.0.play_card(*card) {
            Ok(effect) => {
                info!("AI played card: {:?}, effect: {:?}", card, effect);
                // Move to trick position and show face
                if let Some((entity, mut sprite, _)) =
                    query.iter_mut().find(|(_, _, c)| c.0 == *card)
                {
                    let (x, y) = TRICK_POSITIONS[player_index];
                    // Change to face-up sprite
                    sprite.image =
                        italian_assets.0[card.suit() as usize][card.rank() as usize - 1].clone();
                    commands
                        .entity(entity)
                        .remove_parent_in_place()
                        .insert(CardInPlay)
                        .insert(MovingTo {
                            target: Vec3::new(x, y, 10.0),
                            speed: 200.0,
                            on_arrival: Some(handle_effect_id.0),
                        });
                }
            }
            Err(e) => {
                warn!("AI invalid play: {:?}", e);
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

#[derive(Component, Default)]
struct CardInPlay;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct MovingTo {
    target: Vec3,
    speed: f32,
    on_arrival: Option<SystemId>,
}

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
    id: PlayerId,
    cards_counter: usize,
}

#[derive(Resource)]
pub struct ItalianAssets(pub Vec<Vec<Handle<Image>>>);

#[derive(Resource)]
pub struct CardBack(pub Handle<Image>);

fn remove_name_from_text(
    mut commands: Commands,
    query: Query<Entity, Or<(With<Text>, With<Sprite>)>>,
) {
    for entity in query.iter() {
        commands.entity(entity).remove::<Name>();
    }
}
