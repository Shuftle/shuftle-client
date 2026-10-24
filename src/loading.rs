//! Loads what a game needs before it starts, so the table never shows a card
//! still on its way.

use bevy::{
    asset::{AssetLoadFailedEvent, LoadState},
    prelude::*,
    window::RequestRedraw,
};
use shuftlib::{
    cards::italian::{ItalianRank, Suit},
    tressette::TressetteCard,
};

use crate::{
    menu::widgets::full_screen,
    state::AppState,
    theme::{FONT_HEADING, TEXT},
};

/// The deck the cards are drawn from.
const DECK: &str = "cards/italian/paris/svg-cards";

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Loading), (load_cards, spawn_screen))
            // Before the state transition, so the game starts in the frame the
            // last image came in.
            .add_systems(
                PreUpdate,
                check_loaded.run_if(
                    in_state(AppState::Loading).and_then(
                        on_message::<AssetEvent<Image>>
                            .or_else(on_message::<AssetLoadFailedEvent<Image>>),
                    ),
                ),
            )
            // The asset server only takes in what finished loading while frames
            // are drawn.
            .add_systems(Last, redraw.run_if(in_state(AppState::Loading)));
    }
}

/// The images of the cards.
#[derive(Resource)]
pub struct CardImages {
    /// The faces, by suit in [`Suit::ALL`] order, then by rank in
    /// [`ItalianRank::ALL`] order.
    faces: Vec<Vec<Handle<Image>>>,
    pub back: Handle<Image>,
}

impl CardImages {
    /// Returns the face of `card`.
    pub fn face(&self, card: TressetteCard) -> Handle<Image> {
        self.faces[card.suit() as usize][card.rank() as usize - 1].clone()
    }

    fn all(&self) -> impl Iterator<Item = &Handle<Image>> {
        self.faces.iter().flatten().chain([&self.back])
    }
}

/// Returns the name the deck's files use for `suit`.
fn suit_name(suit: Suit) -> &'static str {
    match suit {
        Suit::Cups => "cups",
        Suit::Coins => "coins",
        Suit::Clubs => "clubs",
        Suit::Swords => "swords",
    }
}

/// Asks for the deck's images. From the second game on they're loaded
/// already, and the game starts right away.
fn load_cards(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut next: ResMut<NextState<AppState>>,
) {
    let faces = Suit::ALL
        .into_iter()
        .map(|suit| {
            ItalianRank::ALL
                .into_iter()
                .map(|rank| {
                    asset_server.load(format!("{DECK}/{}-{}.png", suit_name(suit), rank as u8))
                })
                .collect()
        })
        .collect();
    let images = CardImages {
        faces,
        back: asset_server.load(format!("{DECK}/back.png")),
    };
    if let Some(state) = progress(&images, &asset_server) {
        next.set(state);
    }
    commands.insert_resource(images);
}

/// Starts the game once every image came in, or goes back to the menu if one
/// can't be loaded.
fn check_loaded(
    images: Res<CardImages>,
    asset_server: Res<AssetServer>,
    mut next: ResMut<NextState<AppState>>,
) {
    if let Some(state) = progress(&images, &asset_server) {
        next.set(state);
    }
}

/// Returns where to go once loading is over, or `None` while it isn't.
fn progress(images: &CardImages, asset_server: &AssetServer) -> Option<AppState> {
    let mut loaded = true;
    for image in images.all() {
        if let Some(LoadState::Failed(e)) = asset_server.get_load_state(image) {
            error!("Couldn't load the cards: {e}");
            return Some(AppState::Menu);
        }
        loaded &= asset_server.is_loaded_with_dependencies(image);
    }
    loaded.then_some(AppState::InGame)
}

fn spawn_screen(mut commands: Commands) {
    let scoped = DespawnOnExit(AppState::Loading);
    commands.spawn_scene(bsn! {
        @full_screen()
        Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        scoped
        Children [
            Text("Loading…")
            TextFont { font_size: FontSize::Px(FONT_HEADING) }
            TextColor(TEXT)
        ]
    });
}

/// Asks for another frame.
fn redraw(mut redraw: MessageWriter<RequestRedraw>) {
    redraw.write(RequestRedraw);
}
