//! How the UI looks: colours, text sizes and spacing shared by every screen.
//!
//! Sizes are in logical pixels.

use bevy::prelude::*;

/// Text on the felt: a warm off-white, softer than pure white.
pub const TEXT: Color = Color::srgb(0.96, 0.93, 0.85);

/// Buttons: dark glass over the felt, darker when hovered and pressed, with a
/// faint light edge.
pub const BUTTON: Color = Color::srgba(0., 0., 0., 0.35);
pub const BUTTON_HOVERED: Color = Color::srgba(0., 0., 0., 0.5);
pub const BUTTON_PRESSED: Color = Color::srgba(0., 0., 0., 0.65);
pub const BUTTON_BORDER: Color = Color::srgba(1., 1., 1., 0.25);

/// Behind the bar at the top of the table.
pub const BAR: Color = Color::srgba(0., 0., 0., 0.3);

/// Text sizes, from the game's name down to the smallest labels.
pub const FONT_TITLE: f32 = 64.;
pub const FONT_HEADING: f32 = 36.;
pub const FONT_BUTTON: f32 = 26.;
pub const FONT_BODY: f32 = 24.;
pub const FONT_SMALL: f32 = 20.;

/// Space between a screen's edges and what's on it.
pub const GUTTER: f32 = 16.;
/// Space between the sections of a screen.
pub const SECTION_GAP: f32 = 24.;
/// Space between items in a list, like buttons.
pub const ITEM_GAP: f32 = 12.;

/// Android's minimum touch target.
pub const MIN_TAP: f32 = 48.;
/// Widest a button gets.
pub const BUTTON_MAX_WIDTH: f32 = 360.;
/// Widest a screen's content gets, so lines stay easy to read.
pub const CONTENT_MAX_WIDTH: f32 = 720.;
/// Narrowest a button sized to its label gets.
pub const BUTTON_MIN_WIDTH: f32 = 96.;
/// Space around a button's label, sideways and up and down.
pub const BUTTON_PADDING: Vec2 = Vec2::new(24., 10.);
/// The same, for the small buttons in the top bar.
pub const BAR_BUTTON_PADDING: Vec2 = Vec2::new(10., 4.);
pub const BUTTON_RADIUS: f32 = 10.;
pub const BORDER: f32 = 1.;

/// All of the parent's width or height.
pub const FULL: Val = Val::Percent(100.);
