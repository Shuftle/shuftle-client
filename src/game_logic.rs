use std::f32::consts::PI;

use bevy::{
    ecs::{schedule::common_conditions::any_with_component, system::SystemId},
    prelude::*,
    ui_widgets::{Activate, Button},
};
use shuftlib::{
    cards::italian::{ItalianRank, Suit},
    engine::{Session, Status, UpdateOf},
    tressette::{Action, Event, PerTeam, Seat, Team, Tressette, TressetteCard},
};

use crate::camera::CANVAS_SIZE;

/// The seat of the player in front of the screen.
const POV: Seat = Seat::South;

/// The game, and what the POV player saw of its last step.
#[derive(Resource)]
struct GameState {
    session: Session<Tressette>,
    /// The events of the last step that the POV player may receive.
    last_events: Vec<Event>,
}

impl GameState {
    fn new() -> Self {
        Self {
            session: Session::new(()),
            last_events: Vec::new(),
        }
    }

    /// Keeps the events of `update` that the POV player may receive.
    fn record(&mut self, update: UpdateOf<Tressette>) {
        self.last_events = update.for_seat(POV).cloned().collect();
    }

    /// Returns the seat the game waits on, if any.
    fn to_act(&self) -> Option<Seat> {
        self.session.status().to_act().first().copied()
    }

    /// Returns whether the game is over.
    fn is_over(&self) -> bool {
        matches!(self.session.status(), Status::Over(_))
    }

    /// Returns the seat that took the trick completed by the last step, if
    /// any.
    fn trick_taker(&self) -> Option<Seat> {
        self.last_events.iter().find_map(|event| match event {
            Event::TrickTaken { trick } => Some(trick.taker()),
            _ => None,
        })
    }

    /// Returns the score of the game, if the last step ended a hand.
    fn hand_score(&self) -> Option<PerTeam<u8>> {
        self.last_events.iter().find_map(|event| match event {
            Event::HandScored { score, .. } => Some(*score),
            _ => None,
        })
    }
}

/// Returns where `seat` is drawn: 0 is the bottom of the screen (the POV
/// player), then 1 right, 2 top and 3 left, in play order.
fn screen_slot(seat: Seat) -> usize {
    match seat {
        Seat::South => 0,
        Seat::East => 1,
        Seat::North => 2,
        Seat::West => 3,
    }
}

/// Returns the letter the card assets use for `suit`: they're named after
/// the French suits.
fn asset_letter(suit: Suit) -> &'static str {
    match suit {
        Suit::Cups => "H",
        Suit::Coins => "D",
        Suit::Clubs => "C",
        Suit::Swords => "S",
    }
}

/// Positions for played cards in the trick, by screen slot (center of table,
/// counter-clockwise diamond)
const TRICK_POSITIONS: [(f32, f32); 4] = [
    (0.0, -CARD_SIZE.y), // Player 0 (bottom)
    (CARD_SIZE.y, 0.0),  // Player 1 (right)
    (0.0, CARD_SIZE.y),  // Player 2 (top)
    (-CARD_SIZE.y, 0.0), // Player 3 (left)
];

/// Distance between the visual representation of the player and the edge of the screen
const EDGE_MARGIN: f32 = 10.;

/// Size of the card sprite.
const CARD_SIZE: Vec2 = Vec2::new(32., 48.);

/// Number of cards per player
const CARDS_PER_PLAYER: usize = 10;

pub struct GameLogic;

impl Plugin for GameLogic {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, init_scene)
            .add_systems(
                Update,
                move_to_target.run_if(any_with_component::<MovingTo>),
            )
            .add_systems(Last, despawn_marked.run_if(any_with_component::<ToDespawn>))
            .init_resource::<SetupGameId>()
            .init_resource::<NonPovPlayId>()
            .init_resource::<EnablePovId>()
            .init_resource::<HandleEffectId>()
            .init_resource::<CollectCardsId>()
            .init_resource::<MarkForDespawnAndContinueId>()
            .init_resource::<CardsBeingCollected>()
            .insert_resource(GameState::new())
            .init_state::<Turn>();
    }
}

/// System called at the beginning of the game to load assets and spawn players.
fn init_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    setup_game_sys: Res<SetupGameId>,
) {
    // Load Italian assets
    let mut italian_assets = ItalianAssets(Vec::with_capacity(4));
    for suit in Suit::ALL {
        let mut cards_in_suit = Vec::with_capacity(10);
        for rank in ItalianRank::ALL {
            let sprite_handle = asset_server.load(format!(
                "cards/italian/card-{}-{}.png",
                asset_letter(suit),
                rank as u8
            ));
            cards_in_suit.push(sprite_handle);
        }
        italian_assets.0.push(cards_in_suit);
    }
    commands.insert_resource(italian_assets);

    // Load card back.
    let sprite_handle = asset_server.load("cards/card-back1.png");
    commands.insert_resource(CardBack(sprite_handle));

    // Spawn the players, the POV one at the bottom.
    for seat in Seat::ALL {
        commands.spawn((
            Name::new(seat.to_string()),
            Transform {
                translation: player_position(CANVAS_SIZE.x, CANVAS_SIZE.y, screen_slot(seat)),
                ..default()
            },
            Player {
                seat,
                cards_counter: 0,
            },
            Visibility::default(),
        ));
    }

    // Spawn score display
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: px(10),
            left: px(10),
            ..default()
        },
        Text::new("Score: 0 - 0"),
        TextFont::from_font_size(24.0),
        TextColor(Color::WHITE),
        ScoreText,
    ));

    // Spawn the restart button, hidden until the game is over.
    commands
        .spawn((
            Button,
            Node {
                position_type: PositionType::Absolute,
                top: px(50),
                left: px(10),
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
            children![(
                Text::new("Restart Game"),
                TextFont::from_font_size(24.0),
                TextColor(Color::WHITE),
            )],
            RestartButton,
        ))
        .observe(restart_game);

    // Call system to setup game.
    commands.run_system(setup_game_sys.0);
}

fn player_position(width: f32, height: f32, slot: usize) -> Vec3 {
    match slot {
        0 => Vec3 {
            x: 0.0,
            y: -height * 0.5 + EDGE_MARGIN + CARD_SIZE.y * 0.5,
            z: 0.0,
        },
        1 => Vec3 {
            x: width * 0.5 - EDGE_MARGIN - CARD_SIZE.y * 0.5,
            y: 0.0,
            z: 0.0,
        },
        2 => Vec3 {
            x: 0.0,
            y: height * 0.5 - EDGE_MARGIN - CARD_SIZE.y * 0.5,
            z: 0.0,
        },
        3 => Vec3 {
            x: -width * 0.5 + EDGE_MARGIN + CARD_SIZE.y * 0.5,
            y: 0.0,
            z: 0.0,
        },
        _ => panic!("Invalid screen slot"),
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
/// One shot system that gets called after initial setup is done and every
/// time a new hand has to be dealt.
fn setup_game(
    mut commands: Commands,
    mut game: ResMut<GameState>,
    italian_assets: Res<ItalianAssets>,
    card_back: Res<CardBack>,
    mut query: Query<(Entity, &mut Player)>,
    non_pov_play_id: Res<NonPovPlayId>,
    enable_pov_id: Res<EnablePovId>,
) {
    if matches!(game.session.status(), Status::Chance) {
        match game.session.resolve_chance(&mut rand::rng()) {
            Ok(update) => game.record(update),
            Err(e) => warn!("Couldn't deal: {e}"),
        }
    }

    // Only the POV player's cards are known: the others are dealt face down.
    let view = game.session.observe(POV);
    for (entity, mut player) in query.iter_mut() {
        let seat = player.seat;
        player.cards_counter = 0;
        if seat == POV {
            let mut cards = view.hand().to_vec();
            cards.sort();
            distribute_to_pov(
                &mut commands,
                &italian_assets,
                entity,
                cards,
                &mut player.cards_counter,
            );
        } else {
            distribute_to_other(
                &mut commands,
                &card_back,
                entity,
                view.hand_sizes()[seat],
                &mut player.cards_counter,
                screen_slot(seat),
            );
        }
    }

    players_play(&game, &mut commands, &non_pov_play_id, &enable_pov_id);
}

/// Spawns card entities for the POV player.
fn distribute_to_pov(
    commands: &mut Commands,
    italian_assets: &Res<ItalianAssets>,
    entity: Entity,
    cards: Vec<TressetteCard>,
    card_counter: &mut usize,
) {
    let spacing = CARD_SIZE.x * 0.5;
    let num_cards = CARDS_PER_PLAYER;
    let total_width = (num_cards - 1) as f32 * spacing;
    let center_offset = total_width / 2.0;

    let cards_ids: Vec<_> = cards
        .iter()
        .map(|card| {
            let image_handle = italian_assets.face(*card);
            let id = commands
                .spawn((
                    Card(*card),
                    Sprite {
                        custom_size: Some(CARD_SIZE),
                        image: image_handle,
                        ..default()
                    },
                    Transform::from_xyz(
                        spacing * *card_counter as f32 - center_offset,
                        0.,
                        *card_counter as f32,
                    ),
                    PovCard,
                ))
                .observe(select_play_card)
                .id();
            *card_counter += 1;
            id
        })
        .collect();
    commands.entity(entity).add_children(&cards_ids);
}

/// Spawns `count` face-down card entities for a non POV player, drawn at
/// screen `slot`.
fn distribute_to_other(
    commands: &mut Commands,
    card_back: &Res<CardBack>,
    entity: Entity,
    count: usize,
    card_counter: &mut usize,
    slot: usize,
) {
    let spacing = CARD_SIZE.x * 0.5;
    let num_cards = CARDS_PER_PLAYER;
    let total_width = (num_cards - 1) as f32 * spacing;
    let center_offset = total_width / 2.0;

    let cards_ids: Vec<_> = (0..count)
        .map(|_| {
            let card_pos = spacing * *card_counter as f32;
            let (rotation, translation) = match slot {
                1 => (
                    Quat::from_rotation_z(PI * 0.5),
                    Vec3 {
                        x: 0.,
                        y: card_pos - center_offset,
                        z: *card_counter as f32,
                    },
                ),
                2 => (
                    Quat::IDENTITY,
                    Vec3 {
                        x: -card_pos + center_offset,
                        y: 0.,
                        z: *card_counter as f32,
                    },
                ),
                3 => (
                    Quat::from_rotation_z(-PI * 0.5),
                    Vec3 {
                        x: 0.,
                        y: -card_pos + center_offset,
                        z: *card_counter as f32,
                    },
                ),
                _ => panic!("This should never happen"),
            };
            let id = commands
                .spawn((
                    FaceDown,
                    Transform {
                        translation,
                        rotation,
                        ..default()
                    },
                    Sprite {
                        custom_size: Some(CARD_SIZE),
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
const CARD_SPEED: f32 = 1000.0;
const COLLECTION_DELAY: f32 = 2.;
/// Seconds a bot "thinks" before playing.
const BOT_PLAY_DELAY: f32 = 0.4;
const SELECTION_OFFSET: f32 = 20.;

/// This is called when the POV player clicks on one of their cards.
fn select_play_card(
    click: On<PointerClick>,
    mut game: ResMut<GameState>,
    mut selected_card_query: Query<
        (Entity, &mut Transform, &Card),
        (With<Card>, With<Playable>, With<Selected>),
    >,
    handle_effect_id: Res<HandleEffectId>,
    mut unselected_card_query: Query<(&mut Transform, &Card), (With<Playable>, Without<Selected>)>,
    mut commands: Commands,
    state: Res<State<Turn>>,
) {
    if !matches!(state.into_inner().get(), Turn::PovTurn) {
        return;
    }

    let clicked_card = click.entity;

    for (selected_entity, mut selected_transform, card) in selected_card_query.iter_mut() {
        if selected_entity != clicked_card {
            selected_transform.translation.y -= SELECTION_OFFSET;
            commands.entity(selected_entity).remove::<Selected>();
        } else {
            match game.session.act(POV, Action::Play(card.0)) {
                Ok(update) => {
                    game.record(update);
                    // Move to trick position
                    let (x, y) = TRICK_POSITIONS[screen_slot(POV)];
                    commands
                        .entity(clicked_card)
                        .remove::<Playable>()
                        .remove::<Selected>()
                        .remove::<PovCard>()
                        .insert(CardInPlay)
                        .remove_parent_in_place()
                        .insert(MovingTo {
                            target: Vec3::new(x, y, 10.0),
                            speed: CARD_SPEED,
                            on_arrival: Some(handle_effect_id.0),
                        });
                    commands.set_state(Turn::NonPovTurn);
                }
                Err(e) => {
                    warn!("Invalid play: {e}");
                }
            }
        }
    }
    if let Ok((mut transform, _card)) = unselected_card_query.get_mut(clicked_card) {
        transform.translation.y += SELECTION_OFFSET;
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
            transform.translation = moving.target;
            commands.entity(entity).remove::<MovingTo>();
            if let Some(system_id) = moving.on_arrival {
                commands.run_system(system_id);
            }
        } else {
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
    enable_pov_id: Res<EnablePovId>,
    collect_cards_id: Res<CollectCardsId>,
    mut commands: Commands,
    game: Res<GameState>,
    mut score_text_query: Query<&mut Text, With<ScoreText>>,
) {
    if game.trick_taker().is_none() {
        players_play(&game, &mut commands, &non_pov_play_id, &enable_pov_id);
        return;
    }

    commands
        .delayed()
        .secs(COLLECTION_DELAY)
        .run_system(collect_cards_id.0);
    if let Some(score) = game.hand_score()
        && let Ok(mut text) = score_text_query.single_mut()
    {
        let label = if game.is_over() {
            "Final Score"
        } else {
            "Score"
        };
        *text = Text::new(format!(
            "{label}: {} - {}",
            score[Team::NorthSouth],
            score[Team::EastWest]
        ));
    }
}

#[derive(Component)]
struct ToDespawn;

#[derive(Resource)]
struct CollectCardsId(SystemId);
impl FromWorld for CollectCardsId {
    fn from_world(world: &mut World) -> Self {
        let id = world.register_system(collect_cards);
        CollectCardsId(id)
    }
}

fn collect_cards(
    query: Query<Entity, With<CardInPlay>>,
    game: Res<GameState>,
    mark_for_despawn_and_continue_id: Res<MarkForDespawnAndContinueId>,
    mut cards_being_collected: ResMut<CardsBeingCollected>,
    mut commands: Commands,
) {
    if let Some(taker) = game.trick_taker() {
        let mut count = 0;
        for card in query.iter() {
            commands.entity(card).insert(MovingTo {
                target: player_position(CANVAS_SIZE.x, CANVAS_SIZE.y, screen_slot(taker)),
                speed: CARD_SPEED,
                on_arrival: Some(mark_for_despawn_and_continue_id.0),
            });
            count += 1;
        }
        cards_being_collected.0 = count;
    }
}

#[derive(Resource, Default)]
struct CardsBeingCollected(usize);

#[derive(Resource)]
struct MarkForDespawnAndContinueId(SystemId);
impl FromWorld for MarkForDespawnAndContinueId {
    fn from_world(world: &mut World) -> Self {
        let id = world.register_system(mark_for_despawn_and_continue);
        MarkForDespawnAndContinueId(id)
    }
}
fn mark_for_despawn_and_continue(
    card_query: Query<Entity, (With<CardInPlay>, Without<MovingTo>)>,
    non_pov_play_id: Res<NonPovPlayId>,
    enable_pov_id: Res<EnablePovId>,
    setup_game_id: Res<SetupGameId>,
    game: Res<GameState>,
    mut cards_being_collected: ResMut<CardsBeingCollected>,
    mut button_query: Query<&mut Node, With<RestartButton>>,
    mut commands: Commands,
) {
    // Decrement counter for the card that just arrived
    if cards_being_collected.0 > 0 {
        cards_being_collected.0 -= 1;
    }

    // If all cards have finished collecting
    if cards_being_collected.0 == 0 {
        // Mark all CardInPlay entities for despawn
        for entity in card_query.iter() {
            commands.entity(entity).insert(ToDespawn);
        }

        // After marking cards for despawn, check what to do next based on the
        // last step
        if game.is_over() {
            if let Ok(mut node) = button_query.single_mut() {
                node.display = Display::Flex;
            }
        } else if game.hand_score().is_some() {
            commands.run_system(setup_game_id.0);
        } else if game.trick_taker().is_some() {
            players_play(&game, &mut commands, &non_pov_play_id, &enable_pov_id);
        }
    }
}

fn despawn_marked(mut commands: Commands, query: Query<Entity, With<ToDespawn>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Called when the restart button, shown once the game is over, is activated.
fn restart_game(
    activate: On<Activate>,
    setup_game_id: Res<SetupGameId>,
    mut game: ResMut<GameState>,
    mut score_text_query: Query<&mut Text, With<ScoreText>>,
    card_query: Query<Entity, Or<(With<Card>, With<FaceDown>)>>,
    mut button_query: Query<&mut Node, With<RestartButton>>,
    mut commands: Commands,
) {
    // Mark all cards for despawn
    for card_entity in card_query.iter() {
        commands.entity(card_entity).insert(ToDespawn);
    }
    // Reset game state
    *game = GameState::new();
    // Update score text
    if let Ok(mut text) = score_text_query.single_mut() {
        *text = Text::new("Score: 0 - 0");
    }
    // Hide the button
    if let Ok(mut node) = button_query.get_mut(activate.entity) {
        node.display = Display::None;
    }
    // Restart game
    commands.run_system(setup_game_id.0);
}

/// One shot system that determines whether it's the POV player's turn or not
/// and calls the correct system to handle the play operations.
fn players_play(
    game: &GameState,
    commands: &mut Commands,
    non_pov_play_id: &NonPovPlayId,
    enable_pov_id: &EnablePovId,
) {
    match game.to_act() {
        Some(POV) => commands.run_system(enable_pov_id.0),
        // The delay also lets freshly dealt cards get their GlobalTransform,
        // which `non_pov_play` relies on to take them out of the hand.
        Some(_) => commands
            .delayed()
            .secs(BOT_PLAY_DELAY)
            .run_system(non_pov_play_id.0),
        None => {}
    }
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum Turn {
    #[default]
    PovTurn,
    NonPovTurn,
}

#[derive(Resource)]
struct EnablePovId(SystemId);
impl FromWorld for EnablePovId {
    fn from_world(world: &mut World) -> Self {
        let id = world.register_system(enable_pov);
        EnablePovId(id)
    }
}

fn enable_pov(
    game: Res<GameState>,
    mut commands: Commands,
    mut query: Query<(Entity, &Card, &mut Sprite), With<PovCard>>,
) {
    let playable = game.session.observe(POV).legal_cards();
    for (entity, card, mut sprite) in query.iter_mut() {
        if playable.contains(&card.0) {
            commands.entity(entity).insert(Playable);
            sprite.color = Color::linear_rgb(1., 1., 1.);
        } else {
            commands.entity(entity).remove::<Playable>();
            sprite.color = Color::linear_rgb(0.3, 0.3, 0.3);
        }
    }
    commands.set_state(Turn::PovTurn);
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
    players: Query<(&Player, &Children)>,
    mut face_down: Query<&mut Sprite, With<FaceDown>>,
) {
    let Some(seat) = game.to_act() else {
        return;
    };
    // A bot only knows what its seat may see.
    let Some(card) = game.session.observe(seat).legal_cards().first().copied() else {
        return;
    };
    match game.session.act(seat, Action::Play(card)) {
        Ok(update) => {
            game.record(update);
            // The bot's cards are face down and carry no identity: reveal the
            // last one as the card played, and move it to trick position.
            let Some(entity) = players
                .iter()
                .find(|(player, _)| player.seat == seat)
                .and_then(|(_, children)| {
                    children
                        .iter()
                        .rev()
                        .find(|&child| face_down.contains(child))
                })
            else {
                return;
            };
            if let Ok(mut sprite) = face_down.get_mut(entity) {
                sprite.image = italian_assets.face(card);
            }
            let (x, y) = TRICK_POSITIONS[screen_slot(seat)];
            commands
                .entity(entity)
                .remove::<FaceDown>()
                .insert(Card(card))
                .remove_parent_in_place()
                .insert(CardInPlay)
                .insert(MovingTo {
                    target: Vec3::new(x, y, 10.0),
                    speed: CARD_SPEED,
                    on_arrival: Some(handle_effect_id.0),
                });
        }
        Err(e) => {
            warn!("AI invalid play: {e}");
        }
    }
}

/// A card whose face is shown.
#[derive(Component)]
pub struct Card(pub TressetteCard);

/// A card of a non POV player's hand: shown by its back, and with no
/// identity until it's played.
#[derive(Component)]
struct FaceDown;

#[derive(Component, Default)]
struct Playable;

/// A card in the POV player's hand, which they can click to select and play.
#[derive(Component, Default)]
#[require(Pickable)]
struct PovCard;

#[derive(Component, Default)]
struct Selected;

#[derive(Component, Default)]
struct CardInPlay;

#[derive(Component)]
struct ScoreText;

/// The restart button, shown only once the game is over.
#[derive(Component)]
struct RestartButton;

#[derive(Component)]
struct MovingTo {
    target: Vec3,
    speed: f32,
    on_arrival: Option<SystemId>,
}

#[derive(Component)]
struct Player {
    seat: Seat,
    cards_counter: usize,
}

/// The card faces, by suit in [`Suit::ALL`] order, then by rank in
/// [`ItalianRank::ALL`] order.
#[derive(Resource)]
pub struct ItalianAssets(pub Vec<Vec<Handle<Image>>>);

impl ItalianAssets {
    /// Returns the face of `card`.
    fn face(&self, card: TressetteCard) -> Handle<Image> {
        self.0[card.suit() as usize][card.rank() as usize - 1].clone()
    }
}

#[derive(Resource)]
pub struct CardBack(pub Handle<Image>);
