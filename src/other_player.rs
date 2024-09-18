use bevy::prelude::*;

use crate::load_assets::CardBack;

pub struct OtherPlayer;

impl Plugin for OtherPlayer {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(PostStartup, spawn_hand);
    }
}

fn spawn_hand(mut commands: Commands, card_back: Res<CardBack>) {
    let hand_id = commands
        .spawn((
            Transform {
                translation: Vec3 {
                    x: -450.,
                    y: 250.,
                    ..default()
                },
                ..default()
            },
            InheritedVisibility::default(),
            GlobalTransform::default(),
        ))
        .id();
    let cards_ids: Vec<_> = (0..10)
        .into_iter()
        .map(|i| {
            commands
                .spawn(SpriteBundle {
                    transform: Transform {
                        translation: Vec3 {
                            x: 100. * i as f32,
                            ..default()
                        },
                        ..default()
                    },
                    texture: card_back.as_ref().0.clone_weak(),
                    ..default()
                })
                .id()
        })
        .collect();
    commands.entity(hand_id).push_children(&cards_ids);
}
