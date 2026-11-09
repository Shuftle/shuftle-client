//! What the player can change about the game.

use bevy::{ecs::component::Mutable, prelude::*, ui_widgets::ValueChange};
use shuftlib::cards::italian::{ItalianRank, Suit};

use crate::{
    assets::{
        BACK_FILE, ITALIAN_BACKS, ITALIAN_BACKS_DIR, ITALIAN_DECKS, ITALIAN_DECKS_DIR, italian_face,
    },
    game::table::felt_swatch,
    settings::{BackId, CardLook, DeckId, FeltColour, TableLook},
    state::Screen,
    ui::{
        theme::{FULL, SECTION_GAP},
        widgets::{Choice, ChoiceOption, Preview, choice_row, screen},
    },
};

/// The card each deck's preview shows.
const DECK_PREVIEW: (Suit, ItalianRank) = (Suit::Cups, ItalianRank::King);

/// Shows the settings, with what the player chose checked. A choice naming
/// something that's gone shows what replaces it.
pub fn spawn(mut commands: Commands, card_look: Res<CardLook>, table_look: Res<TableLook>) {
    commands.spawn_scene(scene(
        DeckId(card_look.deck.deck().id.into()),
        BackId(card_look.back.back().id.into()),
        table_look.felt,
    ));
}

/// A row of choices per setting, each option with a preview, with `deck`,
/// `back` and `felt` checked.
fn scene(deck: DeckId, back: BackId, felt: FeltColour) -> impl Scene {
    let (suit, rank) = DECK_PREVIEW;
    let decks = ITALIAN_DECKS
        .iter()
        .map(|deck| ChoiceOption {
            value: DeckId(deck.id.into()),
            name: deck.name,
            preview: Preview::Image(format!(
                "{ITALIAN_DECKS_DIR}/{}/{}",
                deck.id,
                italian_face(suit, rank)
            )),
        })
        .collect();
    let backs = ITALIAN_BACKS
        .iter()
        .map(|back| ChoiceOption {
            value: BackId(back.id.into()),
            name: back.name,
            preview: Preview::Image(format!("{ITALIAN_BACKS_DIR}/{}/{BACK_FILE}", back.id)),
        })
        .collect();
    let felts = FeltColour::ALL
        .into_iter()
        .map(|colour| ChoiceOption {
            value: colour,
            name: colour.name(),
            preview: Preview::Swatch(felt_swatch(colour)),
        })
        .collect();
    screen(
        Screen::Settings,
        "Settings",
        bsn_list! {
            Node {
                width: FULL,
                flex_direction: FlexDirection::Column,
                row_gap: px(SECTION_GAP),
            }
            Children [
                @choice_row("Deck", decks, &deck)
                --
                @choice_row("Card back", backs, &back)
                --
                @choice_row("Felt", felts, &felt)
            ]
        },
    )
}

/// A setting picked from a row of choices, and the group it's kept in.
pub trait Pick: Clone + Default + PartialEq + Send + Sync + Unpin + 'static {
    type Group: Resource<Mutability = Mutable>;
    /// Returns where the setting is in its group.
    fn field(group: &mut Self::Group) -> &mut Self;
}

impl Pick for DeckId {
    type Group = CardLook;
    fn field(group: &mut CardLook) -> &mut Self {
        &mut group.deck
    }
}

impl Pick for BackId {
    type Group = CardLook;
    fn field(group: &mut CardLook) -> &mut Self {
        &mut group.back
    }
}

impl Pick for FeltColour {
    type Group = TableLook;
    fn field(group: &mut TableLook) -> &mut Self {
        &mut group.felt
    }
}

/// Sets the setting `T` to the option picked in its row, if it's a new value.
pub fn pick<T: Pick>(
    change: On<ValueChange<Entity>>,
    choices: Query<&Choice<T>>,
    mut group: ResMut<T::Group>,
) {
    let Ok(choice) = choices.get(change.value) else {
        return;
    };
    if *T::field(group.bypass_change_detection()) != choice.0 {
        *T::field(&mut group) = choice.0.clone();
    }
}
