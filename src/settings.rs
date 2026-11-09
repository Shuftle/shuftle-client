//! What the player chose, kept between runs by Bevy's settings: a TOML file on
//! desktop, the browser's local storage on the web.

use bevy::{
    prelude::*,
    settings::{ReflectSettingsGroup, SaveSettingsSync, SettingsGroup},
};

use crate::assets::{Back, DEFAULT_ITALIAN_BACK, DEFAULT_ITALIAN_DECK, Deck};

/// The name the settings are kept under: the folder on desktop, the key prefix
/// on the web. The app's id, from `.cargo/config.toml`.
pub const APP_NAME: &str = env!("APP_ID");

/// Registers the settings groups, for `SettingsPlugin` to load.
pub fn register(app: &mut App) {
    app.register_type::<CardLook>().register_type::<TableLook>();
}

pub struct SavingPlugin;

impl Plugin for SavingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Not when they're loaded, at start.
                save.run_if(resource_changed::<CardLook>.and_then(not(resource_added::<CardLook>))),
                save.run_if(
                    resource_changed::<TableLook>.and_then(not(resource_added::<TableLook>)),
                ),
            ),
        );
    }
}

/// The cards the game is played with.
#[derive(Resource, SettingsGroup, Reflect, Default, Clone, PartialEq)]
#[reflect(Resource, SettingsGroup, Default)]
pub struct CardLook {
    pub deck: DeckId,
    pub back: BackId,
}

/// The table the game is played on.
#[derive(Resource, SettingsGroup, Reflect, Default, Clone, PartialEq)]
#[reflect(Resource, SettingsGroup, Default)]
pub struct TableLook {
    pub felt: FeltColour,
}

/// The deck a player chose, by id, which may name one that's gone.
#[derive(Reflect, Clone, PartialEq, Debug)]
pub struct DeckId(pub String);

impl DeckId {
    /// Returns the deck, or the default one if it's gone.
    pub fn deck(&self) -> &'static Deck {
        Deck::find(&self.0)
    }
}

impl Default for DeckId {
    fn default() -> Self {
        Self(DEFAULT_ITALIAN_DECK.id.into())
    }
}

/// The card back a player chose, by id, which may name one that's gone.
#[derive(Reflect, Clone, PartialEq, Debug)]
pub struct BackId(pub String);

impl BackId {
    /// Returns the back, or the default one if it's gone.
    pub fn back(&self) -> &'static Back {
        Back::find(&self.0)
    }
}

impl Default for BackId {
    fn default() -> Self {
        Self(DEFAULT_ITALIAN_BACK.id.into())
    }
}

/// The colour of the felt.
#[derive(Reflect, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FeltColour {
    #[default]
    Green,
    Blue,
    Red,
}

impl FeltColour {
    pub const ALL: [Self; 3] = [Self::Green, Self::Blue, Self::Red];

    /// Returns what the game calls it.
    pub fn name(self) -> &'static str {
        match self {
            Self::Green => "Green",
            Self::Blue => "Blue",
            Self::Red => "Red",
        }
    }
}

/// Saves what changed.
fn save(mut commands: Commands) {
    commands.queue(SaveSettingsSync::IfChanged);
}
