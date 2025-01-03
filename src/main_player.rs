use bevy::prelude::*;
use shuftlib::{
    common::{cards::Deck, hands::TrickTakingGame},
    tressette::{TressetteCard, TressetteRules},
};

use crate::load_assets::ItalianAssets;

pub struct MainPlayer;

impl Plugin for MainPlayer {
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
            Transform {
                translation: Vec3 {
                    x: -200.,
                    y: -250.,
                    ..default()
                },
                ..default()
            },
            InheritedVisibility::default(),
            GlobalTransform::default(),
        ))
        .id();
    let cards_ids: Vec<_> = cards
        .iter()
        .enumerate()
        .map(|(i, card)| {
            commands
                .spawn(Cardbundle {
                    card: Card(*card),
                    sprite: Sprite {
                        image: italian_assets.0[card.suit() as usize][card.rank() as usize - 1]
                            .clone_weak(),
                        ..default()
                    },
                    transform: Transform {
                        translation: Vec3 {
                            x: 50. * i as f32,
                            z: i as f32,
                            ..default()
                        },
                        ..default()
                    },
                    playable: Playable,
                })
                .observe(select_play_card)
                .id()
        })
        .collect();
    commands.entity(hand_id).add_children(&cards_ids);
}

fn select_play_card(
    mut trigger: Trigger<Pointer<Click>>,
    mut selected_card_query: Query<
        (Entity, &mut Transform),
        (With<Card>, With<Playable>, With<Selected>),
    >,
    mut unselected_card_query: Query<(&mut Transform, &Card), (With<Playable>, Without<Selected>)>,
    mut commands: Commands,
) {
    trigger.propagate(false);
    let click_event = trigger.event();
    let clicked_card = click_event.target;
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
    pub transform: Transform,
    pub card: Card,
    pub sprite: Sprite,
    pub playable: Playable,
}
