mod camera;
mod player_hand;

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use camera::CameraPlugin;
use player_hand::PlayerHandPlugin;
use shuftlib::{
    common::cards::{FrenchRank, ItalianRank, Suit},
    IntoEnumIterator,
};

fn main() {
    App::new()
        .add_systems(Startup, load_italian_assets)
        .add_plugins(PlayerHandPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .run();
}

fn load_french_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut french_assets = FrenchAssets(Vec::with_capacity(4));
    for suit in Suit::iter() {
        let mut cards_in_suit = Vec::with_capacity(13);
        for rank in FrenchRank::iter() {
            let sprite_handle =
                asset_server.load(format!("cards/french/card-{}-{}.png", suit, rank as u8));
            cards_in_suit.push(sprite_handle);
        }
        french_assets.0.push(cards_in_suit);
    }

    commands.insert_resource(french_assets);
}

fn load_italian_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut italian_assets = ItalianAssets(Vec::with_capacity(4));
    for suit in Suit::iter() {
        let mut cards_in_suit = Vec::with_capacity(10);
        for rank in ItalianRank::iter() {
            let sprite_handle =
                asset_server.load(format!("cards/italian/card-{}-{}.png", suit, rank as u8));
            cards_in_suit.push(sprite_handle);
        }
        italian_assets.0.push(cards_in_suit);
    }

    commands.insert_resource(italian_assets);
}

#[derive(Resource)]
struct FrenchAssets(Vec<Vec<Handle<Image>>>);

#[derive(Resource)]
struct ItalianAssets(Vec<Vec<Handle<Image>>>);
