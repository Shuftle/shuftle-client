use bevy::prelude::*;

use crate::load_assets::CardBack;

#[derive(Event)]
pub struct SpawnOtherPlayer {
    transform: Transform,
}

pub struct OtherPlayer;

impl Plugin for OtherPlayer {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<SpawnOtherPlayer>()
            .add_systems(PostStartup, spawn_hand)
            .add_systems(PostStartup, test_spawn);
    }
}

fn test_spawn(mut event_writer: EventWriter<SpawnOtherPlayer>) {
    // Spawn player 2 (in front of the main player)
    event_writer.send(SpawnOtherPlayer {
        transform: Transform {
            translation: Vec3 {
                x: -100.,
                y: 250.,
                ..default()
            },
            rotation: Quat::from_rotation_z((180f32).to_radians()),
            ..default()
        },
    });
    // Spawn player 1 (to the left of the main player)
    event_writer.send(SpawnOtherPlayer {
        transform: Transform {
            translation: Vec3 {
                x: -575.,
                y: 0.,
                ..default()
            },
            rotation: Quat::from_rotation_z((-90f32).to_radians()),
            ..default()
        },
    });
    // Spawn player 3 (to the right of the main player)
    event_writer.send(SpawnOtherPlayer {
        transform: Transform {
            translation: Vec3 {
                x: 350.,
                y: 0.,
                ..default()
            },
            rotation: Quat::from_rotation_z((90f32).to_radians()),
            ..default()
        },
    });
}

fn spawn_hand(
    mut commands: Commands,
    card_back: Res<CardBack>,
    mut spawn_event: EventReader<SpawnOtherPlayer>,
) {
    for event in spawn_event.read() {
        let hand_id = commands
            .spawn((
                event.transform,
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
                                x: 25. * i as f32,
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
}
