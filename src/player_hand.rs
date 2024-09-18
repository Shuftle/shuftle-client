use bevy::prelude::*;
use bevy_mod_picking::{
    events::{Click, Pointer},
    prelude::{Listener, On},
};
use shuftlib::{
    common::{cards::Deck, hands::TrickTakingGame},
    tressette::{TressetteCard, TressetteRules},
};

use crate::ItalianAssets;

pub struct PlayerHandPlugin;

impl Plugin for PlayerHandPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(PostStartup, setup_hand);
    }
}

fn setup_hand(commands: Commands, italian_assets: Res<ItalianAssets>) {
    let mut deck = Deck::italian();
    deck.shuffle();
    let cards: Vec<TressetteCard> = (0..TressetteRules::TRICKS)
        .map(|_| deck.draw().unwrap().into())
        .collect();
    spawn_hand(&cards, commands, italian_assets);
}

fn spawn_hand(cards: &[TressetteCard], mut commands: Commands, italian_assets: Res<ItalianAssets>) {
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
    let cards_ids: Vec<_> = cards
        .iter()
        .enumerate()
        .map(|(i, card)| {
            commands
                .spawn((
                    Cardbundle::default()
                        .with_card(Card(*card))
                        .with_sprite(SpriteBundle {
                            texture: italian_assets.0[card.suit() as usize]
                                [card.rank() as usize - 1]
                                .clone_weak(),
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
    mut unselected_card_query: Query<(&mut Transform, &Card), (With<Playable>, Without<Selected>)>,
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
    if let Ok((mut transform, card)) = unselected_card_query.get_mut(clicked_card) {
        info!("Card {} was selected", card.0);
        transform.translation.y += 50.;
        commands.entity(clicked_card).insert(Selected);
    }
}

#[derive(Component, Default)]
struct Card(TressetteCard);

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

    fn with_card(mut self, card: Card) -> Cardbundle {
        self.card = card;
        self
    }
}
