//! How the UI looks: colours, text sizes and spacing shared by every screen.
//!
//! Sizes are in logical pixels.

use bevy::prelude::*;

use crate::assets::{CARD_HEIGHT, CARD_WIDTH};

/// Text on the felt: a warm off-white, softer than pure white.
pub const TEXT: Color = Color::srgb(0.96, 0.93, 0.85);

/// Buttons: dark glass over the felt, darker when hovered and pressed, with a
/// faint light edge.
pub const BUTTON: Color = Color::srgba(0., 0., 0., 0.35);
pub const BUTTON_HOVERED: Color = Color::srgba(0., 0., 0., 0.5);
pub const BUTTON_PRESSED: Color = Color::srgba(0., 0., 0., 0.65);
pub const BUTTON_BORDER: Color = Color::srgba(1., 1., 1., 0.25);
/// The chosen option in a row of choices: lighter than a button, with the
/// text's colour as a solid edge.
pub const CHOSEN: Color = Color::srgba(1., 1., 1., 0.15);
pub const CHOSEN_BORDER: Color = TEXT;

/// Behind the bar at the top of the table.
pub const BAR: Color = Color::srgba(0., 0., 0., 0.3);

/// Over the table while a dialog is open.
pub const SCRIM: Color = Color::srgba(0., 0., 0., 0.5);

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
/// Space between a dialog's edges and what's in it.
pub const DIALOG_PADDING: f32 = SECTION_GAP;
/// Widest a dialog gets: a button and the space around it.
pub const DIALOG_MAX_WIDTH: f32 = BUTTON_MAX_WIDTH + 2. * DIALOG_PADDING;
/// Widest a screen's content gets.
pub const CONTENT_MAX_WIDTH: f32 = 720.;
/// Space around a button's label, sideways and up and down.
pub const BUTTON_PADDING: Vec2 = Vec2::new(24., 10.);
pub const BUTTON_RADIUS: f32 = 10.;
pub const BORDER: f32 = 1.;
/// Width of the edge of an option in a row of choices.
pub const CHOICE_BORDER: f32 = 2.;
/// Width of every option in a row of choices.
pub const CHOICE_WIDTH: f32 = 140.;
/// Space inside an option, around its preview and name, and between them.
pub const CHOICE_PADDING: f32 = 10.;
pub const CHOICE_GAP: f32 = 8.;
/// An option's preview: a small card, in the cards' shape.
pub const PREVIEW_WIDTH: f32 = 72.;
pub const PREVIEW_HEIGHT: f32 = PREVIEW_WIDTH * CARD_HEIGHT as f32 / CARD_WIDTH as f32;
/// Corner roundness of a preview, like a card's.
pub const PREVIEW_RADIUS: f32 = 4.;

/// The scrollbar beside a screen's content: its width, its space to the
/// screen's edge, and the shortest its thumb gets.
pub const SCROLLBAR_WIDTH: f32 = 6.;
pub const SCROLLBAR_MARGIN: f32 = 4.;
pub const SCROLLBAR_MIN_THUMB: f32 = 32.;
/// Its thumb: faint light glass, brighter while hovered or dragged.
pub const THUMB: Color = Color::srgba(1., 1., 1., 0.25);
pub const THUMB_ACTIVE: Color = Color::srgba(1., 1., 1., 0.5);

/// All of the parent's width or height.
pub const FULL: Val = Val::Percent(100.);
