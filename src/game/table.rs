//! The felt the game is played on.

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use super::layout::TableLayout;

/// Felt colour in the middle of the table, and at its edges.
const CENTRE: [f32; 3] = [0.16, 0.42, 0.25];
const EDGE: [f32; 3] = [0.05, 0.18, 0.11];

/// Side of the vignette texture, which is stretched over the screen.
const VIGNETTE_SIZE: u32 = 256;
/// How the felt darkens from the middle out: above 1, it stays light longer
/// and darkens towards the corners.
const VIGNETTE_FALLOFF: f32 = 1.6;
/// Side of the grain texture, which is tiled over the screen.
const GRAIN_SIZE: u32 = 128;
/// How much the grain darkens the felt, at most.
const GRAIN_STRENGTH: f32 = 0.04;

/// Depth of the felt's layers, below everything on the table, the grain over
/// the vignette.
const VIGNETTE_Z: f32 = -100.;
const GRAIN_Z: f32 = -99.;

/// Side of the glow texture, which is stretched around what glows.
const GLOW_SIZE: u32 = 64;
/// Share of the glow, across, that's solid, the rest fading out.
const GLOW_SOLID: f32 = 0.5;
/// Width of the glow's fading edge, in pixels of its texture.
pub const GLOW_FADE: f32 = GLOW_SIZE as f32 * (1. - GLOW_SOLID) * 0.5;

/// Channels in a pixel of the generated textures: red, green, blue, alpha.
const CHANNELS: usize = 4;

/// Odd constants the noise hash multiplies by and the shifts it mixes with,
/// spreading bits so neighbouring pixels look unrelated.
const NOISE_X: u32 = 0x27d4_eb2d;
const NOISE_Y: u32 = 0x1656_67b1;
const NOISE_MIX: u32 = 0x85eb_ca6b;
const NOISE_SHIFTS: [u32; 2] = [15, 13];

pub struct TablePlugin;

impl Plugin for TablePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(EDGE[0], EDGE[1], EDGE[2])))
            .init_resource::<Glow>()
            .add_systems(Startup, spawn_felt)
            .add_systems(Update, size_felt.run_if(resource_changed::<TableLayout>));
    }
}

/// A soft white halo, to tint and put behind something to make it stand out.
#[derive(Resource)]
pub struct Glow(pub Handle<Image>);

impl FromWorld for Glow {
    fn from_world(world: &mut World) -> Self {
        Self(world.resource_mut::<Assets<Image>>().add(glow_image()))
    }
}

/// A layer of the felt, covering the whole screen.
#[derive(Component)]
struct Felt;

fn spawn_felt(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn((
        Felt,
        Sprite::from_image(images.add(vignette_image())),
        Transform::from_xyz(0., 0., VIGNETTE_Z),
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
        Transform::from_xyz(0., 0., GRAIN_Z),
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
        let vignette = (uv.length() / std::f32::consts::SQRT_2).powf(VIGNETTE_FALLOFF);
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

/// Draws a glow: a white rectangle whose edges fade out.
fn glow_image() -> Image {
    image(GLOW_SIZE, |x, y| {
        let uv = (Vec2::new(x as f32, y as f32) + 0.5) / GLOW_SIZE as f32 * 2. - 1.;
        // How far out of the solid middle, from 0 there to 1 at the edge.
        let out = ((uv.abs() - GLOW_SOLID).max(Vec2::ZERO) / (1. - GLOW_SOLID))
            .length()
            .min(1.);
        [1., 1., 1., 1. - out * out * (3. - 2. * out)]
    })
}

/// Draws a square image of `size`, with the colour `pixel` gives each pixel.
fn image(size: u32, pixel: impl Fn(u32, u32) -> [f32; CHANNELS]) -> Image {
    let mut data = Vec::with_capacity((size * size) as usize * CHANNELS);
    for y in 0..size {
        for x in 0..size {
            data.extend(pixel(x, y).map(|value| (value.clamp(0., 1.) * u8::MAX as f32) as u8));
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
    let mut hash = x.wrapping_mul(NOISE_X) ^ y.wrapping_mul(NOISE_Y);
    hash ^= hash >> NOISE_SHIFTS[0];
    hash = hash.wrapping_mul(NOISE_MIX);
    hash ^= hash >> NOISE_SHIFTS[1];
    (hash & u16::MAX as u32) as f32 / u16::MAX as f32
}
