//! The felt the game is played on.

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::layout::TableLayout;

/// Felt colour in the middle of the table, and at its edges.
const CENTRE: [f32; 3] = [0.16, 0.42, 0.25];
const EDGE: [f32; 3] = [0.05, 0.18, 0.11];

/// Side of the vignette texture, which is stretched over the screen.
const VIGNETTE_SIZE: u32 = 256;
/// Side of the grain texture, which is tiled over the screen.
const GRAIN_SIZE: u32 = 128;
/// How much the grain darkens the felt, at most.
const GRAIN_STRENGTH: f32 = 0.04;

pub struct TablePlugin;

impl Plugin for TablePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(EDGE[0], EDGE[1], EDGE[2])))
            .add_systems(Startup, spawn_felt)
            .add_systems(Update, size_felt.run_if(resource_changed::<TableLayout>));
    }
}

/// A layer of the felt, covering the whole screen.
#[derive(Component)]
struct Felt;

fn spawn_felt(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn((
        Felt,
        Sprite::from_image(images.add(vignette_image())),
        Transform::from_xyz(0., 0., -100.),
    ));
    // Tiled rather than stretched, so the grain looks the same on any screen.
    commands.spawn((
        Felt,
        Sprite {
            image: images.add(grain_image()),
            image_mode: SpriteImageMode::Tiled {
                tile_x: true,
                tile_y: true,
                stretch_value: 1.,
            },
            ..default()
        },
        Transform::from_xyz(0., 0., -99.),
    ));
}

/// Stretches the felt over the whole screen.
fn size_felt(layout: Res<TableLayout>, mut layers: Query<&mut Sprite, With<Felt>>) {
    for mut sprite in &mut layers {
        sprite.custom_size = Some(layout.size);
    }
}

/// Draws the felt's colour: lighter in the middle, darker towards the edges.
fn vignette_image() -> Image {
    image(VIGNETTE_SIZE, |x, y| {
        let uv = Vec2::new(x as f32, y as f32) / VIGNETTE_SIZE as f32 * 2. - 1.;
        let vignette = (uv.length() / std::f32::consts::SQRT_2).powf(1.6);
        let [r, g, b] = std::array::from_fn(|channel| {
            CENTRE[channel] + (EDGE[channel] - CENTRE[channel]) * vignette
        });
        [r, g, b, 1.]
    })
}

/// Draws the felt's grain: dark specks, mostly transparent.
fn grain_image() -> Image {
    image(GRAIN_SIZE, |x, y| {
        [0., 0., 0., noise(x, y) * GRAIN_STRENGTH]
    })
}

/// Draws a square image of `size`, with the colour `pixel` gives each pixel.
fn image(size: u32, pixel: impl Fn(u32, u32) -> [f32; 4]) -> Image {
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            data.extend(pixel(x, y).map(|value| (value.clamp(0., 1.) * 255.) as u8));
        }
    }
    Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

/// Returns a value between 0 and 1 that looks random from pixel to pixel.
fn noise(x: u32, y: u32) -> f32 {
    let mut hash = x.wrapping_mul(0x27d4_eb2d) ^ y.wrapping_mul(0x1656_67b1);
    hash ^= hash >> 15;
    hash = hash.wrapping_mul(0x85eb_ca6b);
    hash ^= hash >> 13;
    (hash & 0xffff) as f32 / 0xffff as f32
}
