use bevy::{
    ecs::{schedule::common_conditions::any_with_component, system::SystemId},
    prelude::*,
    sprite::{BorderRect, TextureSlicer},
    time::DelayedCommandQueue,
    transform::TransformSystems,
    ui_widgets::{Activate, Button},
    window::RequestRedraw,
};
use shuftlib::{
    cards::italian::{ItalianRank, Suit},
    engine::{Session, Status, UpdateOf},
    tressette::{Action, Event, PerTeam, Seat, Team, Tressette, TressetteCard},
};

use crate::{
    layout::{BAR_HEIGHT, Place, SAFE_TOP, Side, TableLayout},
    table::{GLOW_FADE, Glow},
};

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

/// Returns the colour of the team `seat` plays for.
fn team_color(seat: Seat) -> Color {
    if seat.team() == POV.team() {
        US_COLOR
    } else {
        THEM_COLOR
    }
}

/// Returns the side of the screen `seat` sits at: the POV player at the
/// bottom, then the others in play order, counter-clockwise.
fn side(seat: Seat) -> Side {
    match seat {
        Seat::South => Side::Bottom,
        Seat::East => Side::Right,
        Seat::North => Side::Top,
        Seat::West => Side::Left,
    }
}

/// The deck the cards are drawn from.
const DECK: &str = "cards/italian/paris/svg-cards";

/// Returns the name the deck's files use for `suit`.
fn suit_name(suit: Suit) -> &'static str {
    match suit {
        Suit::Cups => "cups",
        Suit::Coins => "coins",
        Suit::Clubs => "clubs",
        Suit::Swords => "swords",
    }
}

/// Seconds a card takes to move between places.
const TRAVEL_SECS: f32 = 0.3;
const COLLECTION_DELAY: f32 = 2.;
/// Seconds a bot "thinks" before playing.
const BOT_PLAY_DELAY: f32 = 0.7;

/// Colour of what shows whose turn it is.
const TURN_COLOR: Color = Color::srgb(1., 0.84, 0.45);
/// Size of a turn glow, relative to its marker.
const GLOW_SCALE: f32 = 1.7;
/// How opaque the glow around the cards the POV player can play is, in the turn
/// colour, and how far it reaches past their edges.
const PLAYABLE_ALPHA: f32 = 0.85;
const PLAYABLE_REACH: f32 = 6.;
/// Depth of that glow, below every card in the hand.
const PLAYABLE_GLOW_Z: f32 = 0.5;
/// Colours of the POV player's team, and of the other one.
const US_COLOR: Color = Color::srgb(0.55, 0.78, 1.);
const THEM_COLOR: Color = Color::srgb(1., 0.6, 0.55);

/// Drawing order: hand cards are 1 and up, in hand order. Cards in the same
/// layer are ordered by [`card_order`], so overlapping ones don't flicker.
const MARKER_Z: f32 = 20.;
const TRICK_Z: f32 = 30.;
const TRAVEL_Z: f32 = 40.;

/// Returns a depth offset, below 1, that's different for every card.
fn card_order(card: TressetteCard) -> f32 {
    (card.suit() as usize * 10 + card.rank() as usize) as f32 * 0.01
}

pub struct GameLogic;

impl Plugin for GameLogic {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, init_scene)
            .add_systems(
                Update,
                (
                    place_cards,
                    place_markers,
                    show_turn,
                    show_score.run_if(resource_changed::<ShownScore>),
                ),
            )
            .add_systems(
                Last,
                (
                    despawn_marked.run_if(any_with_component::<ToDespawn>),
                    // Frames are only drawn on input, or while something here
                    // needs them: what moves, and what only gets on while
                    // frames are drawn.
                    redraw.run_if(
                        any_with_component::<Travel>
                            .or_else(any_with_component::<Pulse>)
                            .or_else(any_with_component::<DelayedCommandQueue>)
                            .or_else(images_loading),
                    ),
                ),
            )
            .add_systems(
                PostUpdate,
                fit_playable_glows.before(TransformSystems::Propagate),
            )
            .add_observer(glow_playable)
            .add_observer(unglow_playable)
            .init_resource::<SetupGameId>()
            .init_resource::<NonPovPlayId>()
            .init_resource::<EnablePovId>()
            .init_resource::<HandleEffectId>()
            .init_resource::<CollectCardsId>()
            .init_resource::<MarkForDespawnAndContinueId>()
            .init_resource::<CardsBeingCollected>()
            .init_resource::<ShownScore>()
            .insert_resource(GameState::new());
    }
}

/// System called at the beginning of the game to load assets and set the
/// table.
fn init_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    glow: Res<Glow>,
    setup_game_sys: Res<SetupGameId>,
) {
    // Load Italian assets
    let mut italian_assets = ItalianAssets(Vec::with_capacity(4));
    for suit in Suit::ALL {
        let mut cards_in_suit = Vec::with_capacity(10);
        for rank in ItalianRank::ALL {
            let sprite_handle =
                asset_server.load(format!("{DECK}/{}-{}.png", suit_name(suit), rank as u8));
            cards_in_suit.push(sprite_handle);
        }
        italian_assets.0.push(cards_in_suit);
    }
    commands.insert_resource(italian_assets);

    // The other players are shown by a card back with how many cards they
    // hold, named in their team's colour and glowing when it's their turn.
    let card_back: Handle<Image> = asset_server.load(format!("{DECK}/back.png"));
    for seat in Seat::ALL.into_iter().filter(|&seat| seat != POV) {
        let marker = Marker(seat);
        let name = seat.to_string();
        let color = team_color(seat);
        commands.spawn_scene(bsn! {
            marker
            Sprite { image: {card_back.clone()} }
            Children [
                MarkerCount
                Text2d
                Transform { translation: {Vec3::Z} }
                --
                MarkerName
                Text2d({name})
                TextColor({color})
                --
                TurnGlow
                Sprite { image: {glow.0.clone()} }
                Transform { translation: {Vec3::NEG_Z} }
                Visibility::Hidden
            ]
        });
    }

    // The top bar, below the room phones keep for the status bar and camera.
    commands.spawn_scene(bsn! {
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: {px(SAFE_TOP + BAR_HEIGHT)},
            padding: {UiRect { left: px(12), right: px(12), top: px(SAFE_TOP), ..default() }},
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
        }
        BackgroundColor({Color::srgba(0., 0., 0., 0.3)})
        Children [
            Text
            TextFont { font_size: FontSize::Px(24.) }
            ScoreText
            Children [
                TextSpan
                TextFont { font_size: FontSize::Px(24.) }
                TextColor({US_COLOR})
                UsScore
                --
                TextSpan("   ")
                TextFont { font_size: FontSize::Px(24.) }
                --
                TextSpan
                TextFont { font_size: FontSize::Px(24.) }
                TextColor({THEM_COLOR})
                ThemScore
            ]
            --
            Text
            TextFont { font_size: FontSize::Px(24.) }
            TextColor({TURN_COLOR})
            TurnText
        ]
    });

    // The restart button, centred on the table and hidden until the game is
    // over. What centres it covers the screen, so it lets taps through.
    let ignore = Pickable::IGNORE;
    commands.spawn_scene(bsn! {
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        ignore
        Children [
            Button
            Node {
                padding: {UiRect::all(px(12))},
                display: Display::None,
            }
            BackgroundColor({Color::srgb(0.5, 0.5, 0.5)})
            RestartButton
            on(restart_game)
            Children [
                Text("Restart Game")
                TextFont { font_size: FontSize::Px(24.) }
            ]
        ]
    });

    // Call system to setup game.
    commands.run_system(setup_game_sys.0);
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
    non_pov_play_id: Res<NonPovPlayId>,
    enable_pov_id: Res<EnablePovId>,
) {
    if matches!(game.session.status(), Status::Chance) {
        match game.session.resolve_chance(&mut rand::rng()) {
            Ok(update) => game.record(update),
            Err(e) => warn!("Couldn't deal: {e}"),
        }
    }

    // Only the POV player's cards are known: they come up into the hand from
    // below the screen, while the markers show how many the others got.
    for &card in game.session.observe(POV).hand() {
        let card = Card(card);
        let face = italian_assets.face(card.0);
        let spot = Spot::Hand;
        let travel = Travel::default().starting_at(Place::Marker(Side::Bottom));
        commands.spawn_scene(bsn! {
            card
            spot
            travel
            Sprite { image: {face} }
            // Sprites only get taps and clicks when they're pickable.
            Pickable
            on(play_card)
        });
    }

    players_play(&game, &mut commands, &non_pov_play_id, &enable_pov_id);
}

/// This is called when the POV player taps or clicks one of their cards: if
/// it's their turn and the card can be played, it's played.
fn play_card(
    click: On<PointerClick>,
    mut game: ResMut<GameState>,
    playable: Query<&Card, With<Playable>>,
    mut hand: Query<(Entity, &Spot, &mut Sprite)>,
    handle_effect_id: Res<HandleEffectId>,
    mut commands: Commands,
) {
    // The game itself says whose turn it is right away, even for a second
    // tap in the same frame.
    if game.to_act() != Some(POV) {
        return;
    }
    let clicked = click.entity;
    let Ok(card) = playable.get(clicked) else {
        return;
    };
    match game.session.act(POV, Action::Play(card.0)) {
        Ok(update) => {
            game.record(update);
            commands
                .entity(clicked)
                .insert((Spot::Trick(Side::Bottom), Travel::then(handle_effect_id.0)));
            // Until it's their turn again, the hand looks neither playable nor
            // dimmed.
            for (entity, spot, mut sprite) in &mut hand {
                if *spot == Spot::Hand {
                    commands.entity(entity).remove::<Playable>();
                    sprite.color = Color::WHITE;
                }
            }
        }
        Err(e) => {
            warn!("Invalid play: {e}");
        }
    }
}

/// Puts every card where the layout wants its spot, moving it there if it's
/// travelling. Runs every frame, so cards follow the layout as the window is
/// resized or the phone rotated.
fn place_cards(
    layout: Res<TableLayout>,
    time: Res<Time>,
    mut commands: Commands,
    mut cards: Query<(
        Entity,
        &Card,
        &Spot,
        &mut Transform,
        &mut Sprite,
        Option<&mut Travel>,
    )>,
) {
    // The hand is kept sorted.
    let mut hand: Vec<_> = cards
        .iter()
        .filter(|(_, _, spot, ..)| **spot == Spot::Hand)
        .map(|(entity, card, ..)| (card.0, entity))
        .collect();
    hand.sort();

    for (entity, card, spot, mut transform, mut sprite, travel) in &mut cards {
        let order = card_order(card.0);
        let (place, z) = match *spot {
            Spot::Hand => {
                let index = hand.iter().position(|&(_, e)| e == entity).unwrap_or(0);
                let count = hand.len();
                (Place::Hand { index, count }, 1. + index as f32)
            }
            Spot::Trick(side) => (Place::Trick(side), TRICK_Z + order),
            Spot::Taken(side) => (Place::Marker(side), TRICK_Z + order),
        };
        let target = layout.position(place).extend(z);
        let size = layout.card_size(place);

        let Some(mut travel) = travel else {
            if *spot == Spot::Hand && transform.translation.with_z(z).distance(target) >= 0.5 {
                // Move rather than jump to a new slot, as when two rows merge
                // into one.
                commands.entity(entity).insert(Travel::default());
            } else {
                put(&mut transform, &mut sprite, target, size);
            }
            continue;
        };
        // A start place follows the layout for as long as the card travels, as
        // the window can still get its real size, or the phone be rotated.
        let (from, from_size) = match travel.start {
            Some(start) => (layout.position(start).extend(0.), layout.card_size(start)),
            None => *travel
                .from
                .get_or_insert((transform.translation, sprite.custom_size.unwrap_or(size))),
        };
        travel.elapsed += time.delta_secs();
        let progress = (travel.elapsed / TRAVEL_SECS).min(1.);
        let eased = progress * progress * (3. - 2. * progress);
        put(
            &mut transform,
            &mut sprite,
            from.lerp(target, eased).with_z(TRAVEL_Z + order),
            from_size.lerp(size, eased),
        );
        if progress >= 1. {
            put(&mut transform, &mut sprite, target, size);
            commands.entity(entity).remove::<Travel>();
            if let Some(system_id) = travel.on_arrival {
                commands.run_system(system_id);
            }
        }
    }
}

/// Moves and sizes a sprite, touching it only if that changes anything: a
/// change makes Bevy redo its transforms and rendering data.
fn put(transform: &mut Mut<Transform>, sprite: &mut Mut<Sprite>, at: Vec3, size: Vec2) {
    if transform.translation != at {
        transform.translation = at;
    }
    if sprite.custom_size != Some(size) {
        sprite.custom_size = Some(size);
    }
}

/// Places the other players' markers, showing how many cards they hold.
/// Shows whose turn it is: a glow pulsing behind the marker of the player to
/// play, or a note in the top bar for the POV player.
fn show_turn(
    layout: Res<TableLayout>,
    game: Res<GameState>,
    time: Res<Time>,
    markers: Query<(&Marker, &Children)>,
    mut glows: Query<(&mut Sprite, &mut Visibility), With<TurnGlow>>,
    mut note: Single<&mut Text, With<TurnText>>,
    playable: Query<(), With<Playable>>,
    mut commands: Commands,
) {
    let to_act = game.to_act();
    for (marker, children) in &markers {
        for &child in children {
            let Ok((mut sprite, mut visibility)) = glows.get_mut(child) else {
                continue;
            };
            if to_act != Some(marker.0) {
                if visibility.set_if_neq(Visibility::Hidden) {
                    commands.entity(child).remove::<Pulse>();
                }
                continue;
            }
            if visibility.set_if_neq(Visibility::Inherited) {
                commands.entity(child).insert(Pulse);
            }
            let size = layout.card_size(Place::Marker(side(marker.0))) * GLOW_SCALE;
            if sprite.custom_size != Some(size) {
                sprite.custom_size = Some(size);
            }
            let pulse = 0.55 + 0.3 * (time.elapsed_secs() * std::f32::consts::TAU).sin();
            sprite.color = TURN_COLOR.with_alpha(pulse);
        }
    }

    // Only once the POV player's cards can be played: the game says it's their
    // turn already while the last card still travels or the trick is
    // collected.
    let wanted = if playable.is_empty() { "" } else { "Your turn" };
    if note.0 != wanted {
        note.0 = wanted.into();
    }
}

fn place_markers(
    layout: Res<TableLayout>,
    game: Res<GameState>,
    mut markers: Query<(&Marker, &Children, &mut Transform, &mut Sprite)>,
    mut counts: Query<(&mut Text2d, &mut TextFont), (With<MarkerCount>, Without<MarkerName>)>,
    mut names: Query<(&mut Transform, &mut TextFont), (With<MarkerName>, Without<Marker>)>,
) {
    let view = game.session.observe(POV);
    for (marker, children, mut transform, mut sprite) in &mut markers {
        let place = Place::Marker(side(marker.0));
        let size = layout.card_size(place);
        put(
            &mut transform,
            &mut sprite,
            layout.position(place).extend(MARKER_Z),
            size,
        );

        let count = view.hand_sizes()[marker.0];
        for &child in children {
            if let Ok((mut text, mut font)) = counts.get_mut(child) {
                let count = count.to_string();
                if text.0 != count {
                    text.0 = count;
                }
                // A squeezed window can shrink markers to nothing, but text can't
                // be sized 0.
                font.set_if_neq(TextFont::from_font_size((size.y * 0.4).max(1.)));
            }
            if let Ok((mut name_transform, mut font)) = names.get_mut(child) {
                let (font_size, below) = layout.marker_label();
                font.set_if_neq(TextFont::from_font_size(font_size.max(1.)));
                name_transform.set_if_neq(Transform::from_xyz(0., below, 1.));
            }
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
    mut shown: ResMut<ShownScore>,
) {
    if game.trick_taker().is_none() {
        players_play(&game, &mut commands, &non_pov_play_id, &enable_pov_id);
        return;
    }

    commands
        .delayed()
        .secs(COLLECTION_DELAY)
        .run_system(collect_cards_id.0);
    if let Some(score) = game.hand_score() {
        let us = POV.team();
        let them = if us == Team::NorthSouth {
            Team::EastWest
        } else {
            Team::NorthSouth
        };
        *shown = ShownScore {
            us: score[us],
            them: score[them],
            is_final: game.is_over(),
        };
    }
}

/// Writes the score in the top bar.
fn show_score(
    shown: Res<ShownScore>,
    mut label: Single<&mut Text, With<ScoreText>>,
    mut us: Single<&mut TextSpan, (With<UsScore>, Without<ThemScore>)>,
    mut them: Single<&mut TextSpan, With<ThemScore>>,
) {
    label.0 = if shown.is_final { "Final  " } else { "" }.into();
    us.0 = format!("Us {}", shown.us);
    them.0 = format!("Them {}", shown.them);
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

/// Sends the cards of the trick to whoever took it.
fn collect_cards(
    cards: Query<(Entity, &Spot)>,
    game: Res<GameState>,
    mark_for_despawn_and_continue_id: Res<MarkForDespawnAndContinueId>,
    mut cards_being_collected: ResMut<CardsBeingCollected>,
    mut commands: Commands,
) {
    if let Some(taker) = game.trick_taker() {
        let mut count = 0;
        for (card, spot) in &cards {
            if matches!(spot, Spot::Trick(_)) {
                commands.entity(card).insert((
                    Spot::Taken(side(taker)),
                    Travel::then(mark_for_despawn_and_continue_id.0),
                ));
                count += 1;
            }
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
    cards: Query<(Entity, &Spot), Without<Travel>>,
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
        for (card, spot) in &cards {
            if matches!(spot, Spot::Taken(_)) {
                commands.entity(card).insert(ToDespawn);
            }
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
    mut shown: ResMut<ShownScore>,
    card_query: Query<Entity, With<Card>>,
    mut button_query: Query<&mut Node, With<RestartButton>>,
    mut commands: Commands,
) {
    // Mark all cards for despawn
    for card_entity in card_query.iter() {
        commands.entity(card_entity).insert(ToDespawn);
    }
    // Reset game state
    *game = GameState::new();
    *shown = ShownScore::default();
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
        Some(_) => commands
            .delayed()
            .secs(BOT_PLAY_DELAY)
            .run_system(non_pov_play_id.0),
        None => {}
    }
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
    mut query: Query<(Entity, &Card, &Spot, &mut Sprite)>,
) {
    let playable = game.session.observe(POV).legal_cards();
    for (entity, card, spot, mut sprite) in query.iter_mut() {
        if *spot != Spot::Hand {
            continue;
        }
        if playable.contains(&card.0) {
            commands.entity(entity).insert(Playable);
            sprite.color = Color::WHITE;
        } else {
            commands.entity(entity).remove::<Playable>();
            sprite.color = Color::linear_rgb(0.3, 0.3, 0.3);
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

/// One shot system called for non POV players: their card comes out of their
/// marker into the trick.
fn non_pov_play(
    mut game: ResMut<GameState>,
    mut commands: Commands,
    handle_effect_id: Res<HandleEffectId>,
    italian_assets: Res<ItalianAssets>,
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
            let face = italian_assets.face(card);
            let card = Card(card);
            let spot = Spot::Trick(side(seat));
            let travel = Travel::then(handle_effect_id.0).starting_at(Place::Marker(side(seat)));
            commands.spawn_scene(bsn! {
                card
                spot
                travel
                Sprite { image: {face} }
            });
        }
        Err(e) => {
            warn!("AI invalid play: {e}");
        }
    }
}

/// A card whose face is shown.
#[derive(Component, Clone)]
pub struct Card(pub TressetteCard);

/// Where a card belongs on the table.
#[derive(Component, Clone, Copy, PartialEq)]
enum Spot {
    /// In the POV player's hand.
    Hand,
    /// Played in the trick by the player at this side.
    Trick(Side),
    /// In a trick taken by the player at this side.
    Taken(Side),
}

/// Moves a card to its spot instead of putting it there.
#[derive(Component, Clone, Default)]
struct Travel {
    /// Where the card sets off from, if not where it is.
    start: Option<Place>,
    /// Where the card started from, and its size there, once it's moving,
    /// when it has no `start`.
    from: Option<(Vec3, Vec2)>,
    elapsed: f32,
    /// Run when the card gets there.
    on_arrival: Option<SystemId>,
}

impl Travel {
    fn then(on_arrival: SystemId) -> Self {
        Self {
            on_arrival: Some(on_arrival),
            ..default()
        }
    }

    /// Sets off from `start`, wherever the layout has it while the card
    /// travels: the window may not have its real size yet when the card is
    /// spawned.
    fn starting_at(self, start: Place) -> Self {
        Self {
            start: Some(start),
            ..self
        }
    }
}

#[derive(Component, Default, Clone)]
struct Playable;

/// The glow behind a card that can be played.
#[derive(Component)]
struct PlayableGlow;

/// Puts a glow behind a card once it can be played.
fn glow_playable(add: On<Add<Playable>>, glow: Res<Glow>, mut commands: Commands) {
    commands.entity(add.entity).with_child((
        PlayableGlow,
        Sprite {
            image: glow.0.clone(),
            color: TURN_COLOR.with_alpha(PLAYABLE_ALPHA),
            // Sliced, so the glow fades out over the same width around any
            // card, starting at its edges.
            image_mode: SpriteImageMode::Sliced(TextureSlicer {
                border: BorderRect::all(GLOW_FADE),
                max_corner_scale: PLAYABLE_REACH / GLOW_FADE,
                ..default()
            }),
            ..default()
        },
    ));
}

/// Takes the glow away once a card can't be played anymore.
fn unglow_playable(
    remove: On<Remove<Playable>>,
    children: Query<&Children>,
    glows: Query<(), With<PlayableGlow>>,
    mut commands: Commands,
) {
    for child in children.iter_descendants(remove.entity) {
        if glows.contains(child) {
            // The card may be on its way out, with its glow.
            commands.entity(child).try_despawn();
        }
    }
}

/// Sizes the glows of playable cards to their cards, and keeps them below
/// every card in the hand.
fn fit_playable_glows(
    cards: Query<(&Sprite, &Transform), Without<PlayableGlow>>,
    mut glows: Query<(&ChildOf, &mut Sprite, &mut Transform), With<PlayableGlow>>,
) {
    for (child_of, mut sprite, mut transform) in &mut glows {
        let Ok((card, card_transform)) = cards.get(child_of.parent()) else {
            continue;
        };
        let Some(card_size) = card.custom_size else {
            continue;
        };
        let size = card_size + Vec2::splat(PLAYABLE_REACH * 2.);
        if sprite.custom_size != Some(size) {
            sprite.custom_size = Some(size);
        }
        let z = PLAYABLE_GLOW_Z - card_transform.translation.z;
        if transform.translation.z != z {
            transform.translation.z = z;
        }
    }
}

/// The card back standing for another player's hand.
#[derive(Component, Clone)]
struct Marker(Seat);

/// The number of cards on a marker.
#[derive(Component, Default, Clone)]
struct MarkerCount;

/// The halo around a marker, shown while that player is to play.
#[derive(Component, Default, Clone)]
struct TurnGlow;

/// On a turn glow while it's shown, and pulses.
#[derive(Component)]
struct Pulse;

/// Says in the top bar when it's the POV player's turn.
#[derive(Component, Default, Clone)]
struct TurnText;

/// The name under a marker.
#[derive(Component, Default, Clone)]
struct MarkerName;

/// The POV player's team's score, and the other team's, in the top bar.
#[derive(Component, Default, Clone)]
struct UsScore;

#[derive(Component, Default, Clone)]
struct ThemScore;

/// The score the top bar shows.
#[derive(Resource, Default)]
struct ShownScore {
    us: u8,
    them: u8,
    is_final: bool,
}

#[derive(Component, Default, Clone)]
struct ScoreText;

/// The restart button, shown only once the game is over.
#[derive(Component, Default, Clone)]
struct RestartButton;

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

/// Asks for another frame.
fn redraw(mut redraw: MessageWriter<RequestRedraw>) {
    redraw.write(RequestRedraw);
}

/// Returns whether a sprite's image is still loading.
fn images_loading(sprites: Query<&Sprite>, asset_server: Res<AssetServer>) -> bool {
    sprites.iter().any(|sprite| {
        asset_server
            .get_load_state(&sprite.image)
            .is_some_and(|state| state.is_loading())
    })
}
