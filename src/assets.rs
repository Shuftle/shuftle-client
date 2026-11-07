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
    settings::CardLook,
    state::{AppState, GameSession},
};

// The decks and backs the game ships, checked when the game was built.
include!(concat!(env!("OUT_DIR"), "/cards.rs"));

/// A deck of cards in the assets.
pub struct Deck {
    /// Its folder in `ITALIAN_DECKS_DIR`, `<pattern>/<set>`.
    pub id: &'static str,
    /// What the game calls it.
    pub name: &'static str,
}

impl Deck {
    /// Returns the deck `id`, or the default one if it's gone.
    pub fn find(id: &str) -> &'static Deck {
        ITALIAN_DECKS
            .iter()
            .find(|deck| deck.id == id)
            .unwrap_or(DEFAULT_ITALIAN_DECK)
    }
}

/// A card back in the assets.
pub struct Back {
    /// Its folder in `ITALIAN_BACKS_DIR`.
    pub id: &'static str,
    /// What the game calls it.
    pub name: &'static str,
}

impl Back {
    /// Returns the back `id`, or the default one if it's gone.
    pub fn find(id: &str) -> &'static Back {
        ITALIAN_BACKS
            .iter()
            .find(|back| back.id == id)
            .unwrap_or(DEFAULT_ITALIAN_BACK)
    }
}

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Loading), load_cards)
            .add_systems(OnExit(GameSession), free_cards)
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
            .init_resource::<PendingImages>()
            .add_observer(wait_for_image_node)
            .add_systems(Last, keep_drawing.run_if(images_pending));
    }
}

/// Images being waited for, which keep frames coming until they're in.
#[derive(Resource, Default)]
pub struct PendingImages(Vec<Handle<Image>>);

impl PendingImages {
    /// Keeps frames coming until `image` is loaded, or fails to, and returns it.
    pub fn wait_for(&mut self, image: Handle<Image>) -> Handle<Image> {
        self.0.push(image.clone());
        image
    }
}

/// Waits for the image of a UI node added while it's still loading.
fn wait_for_image_node(
    add: On<Add<ImageNode>>,
    nodes: Query<&ImageNode>,
    asset_server: Res<AssetServer>,
    mut pending: ResMut<PendingImages>,
) {
    if let Ok(node) = nodes.get(add.entity)
        && !asset_server.is_loaded_with_dependencies(&node.image)
    {
        pending.wait_for(node.image.clone());
    }
}

/// Whether any image is still loading.
fn images_pending(pending: Res<PendingImages>) -> bool {
    !pending.0.is_empty()
}

/// Forgets the images that came in or failed, and asks for another frame
/// while any is still loading.
fn keep_drawing(
    mut pending: ResMut<PendingImages>,
    asset_server: Res<AssetServer>,
    mut redraw: MessageWriter<RequestRedraw>,
) {
    pending.0.retain(|image| {
        !asset_server.is_loaded_with_dependencies(image)
            && !matches!(
                asset_server.get_load_state(image),
                Some(LoadState::Failed(_))
            )
    });
    if !pending.0.is_empty() {
        redraw.write(RequestRedraw);
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

/// Asks for the images of the chosen deck and back. If they're all loaded
/// already, the game starts right away.
fn load_cards(
    asset_server: Res<AssetServer>,
    card_look: Res<CardLook>,
    mut pending: ResMut<PendingImages>,
    mut commands: Commands,
    mut next: ResMut<NextState<AppState>>,
) {
    let (deck, back) = (card_look.deck.deck(), card_look.back.back());
    let mut load = |path: String| pending.wait_for(asset_server.load(path));
    let faces = Suit::ALL
        .into_iter()
        .map(|suit| {
            ItalianRank::ALL
                .into_iter()
                .map(|rank| {
                    load(format!(
                        "{ITALIAN_DECKS_DIR}/{}/{}",
                        deck.id,
                        italian_face(suit, rank)
                    ))
                })
                .collect()
        })
        .collect();
    let images = CardImages {
        faces,
        back: load(format!("{ITALIAN_BACKS_DIR}/{}/{BACK_FILE}", back.id)),
    };
    if let Some(state) = progress(&images, &asset_server) {
        next.set(state);
    }
    commands.insert_resource(images);
}

/// Lets go of the cards' images once their game is over, or failed to load.
fn free_cards(mut commands: Commands) {
    commands.remove_resource::<CardImages>();
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
