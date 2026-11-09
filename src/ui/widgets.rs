//! Pieces the screens are built from.

use bevy::{
    prelude::*,
    ui_widgets::{Activate, Button},
};

use super::theme::*;
use crate::{game::layout::SAFE_TOP, state::Screen};

/// A button shaded as the pointer moves over, presses and leaves it.
#[derive(Component, Default, Clone)]
pub struct MenuButton;

/// A menu button, as wide as its column allows, and never shorter than
/// Android's minimum touch target.
pub fn menu_button(label: &str) -> impl Scene {
    bsn! {
        @button(label, FONT_BUTTON, BUTTON_PADDING)
        Node {
            width: FULL,
            max_width: px(BUTTON_MAX_WIDTH),
            min_height: px(MIN_TAP),
        }
    }
}

/// A small button, sized to its label, for the bar at the top of the table.
pub fn bar_button(label: &str) -> impl Scene {
    bsn! {
        @button(label, FONT_SMALL, BAR_BUTTON_PADDING)
    }
}

fn button(label: &str, font_size: f32, padding: Vec2) -> impl Scene {
    let ignore = Pickable::IGNORE;
    bsn! {
        MenuButton
        Button
        Node {
            padding: {UiRect::axes(px(padding.x), px(padding.y))},
            border: {UiRect::all(px(BORDER))},
            border_radius: {BorderRadius::all(px(BUTTON_RADIUS))},
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        BackgroundColor(BUTTON)
        BorderColor::all(BUTTON_BORDER)
        Children [
            Text(label)
            TextFont { font_size: {FontSize::Px(font_size)} }
            TextColor(TEXT)
            // Taps land on the button, not its label.
            ignore
        ]
    }
}

/// Padding that keeps a full screen's content off its edges, and below the
/// room phones keep for the status bar and camera.
pub fn screen_padding() -> UiRect {
    UiRect {
        top: px(SAFE_TOP + GUTTER),
        ..UiRect::all(px(GUTTER))
    }
}

/// A node covering the whole screen, over whatever else is on it.
pub fn full_screen() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            width: FULL,
            height: FULL,
        }
    }
}

/// A node taking as much of its parent's room, along the parent's direction,
/// as each sibling that is one too, whatever its content.
pub fn equal_share() -> impl Scene {
    bsn! {
        Node {
            flex_grow: 1.,
            flex_basis: px(0),
        }
    }
}

/// A full screen with `title` at the top, a back button, and `body` below,
/// shown while in `screen`.
pub fn screen(screen: Screen, title: &str, body: impl SceneList) -> impl Scene {
    let scoped = DespawnOnExit(screen);
    bsn! {
        @full_screen()
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: {screen_padding()},
            row_gap: px(SECTION_GAP),
        }
        scoped
        Children [
            Node {
                width: FULL,
                max_width: px(CONTENT_MAX_WIDTH),
                align_items: AlignItems::Center,
                column_gap: px(GUTTER),
            }
            Children [
                @menu_button("Back")
                Node { width: auto(), min_width: px(BUTTON_MIN_WIDTH) }
                on(back)
                --
                Text(title)
                TextFont { font_size: FontSize::Px(FONT_HEADING) }
                TextColor(TEXT)
            ]
            --
            Node {
                width: FULL,
                max_width: px(CONTENT_MAX_WIDTH),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(ITEM_GAP),
            }
            Children [ {body} ]
        ]
    }
}

/// Goes to the screen above this one.
fn back(_: On<Activate>, screen: Res<State<Screen>>, mut next: ResMut<NextState<Screen>>) {
    if let Some(parent) = screen.get().parent() {
        next.set(parent);
    }
}

pub fn button_over(over: On<PointerOver>, buttons: Query<&mut BackgroundColor, With<MenuButton>>) {
    shade(over.entity, BUTTON_HOVERED, buttons);
}

pub fn button_out(out: On<PointerOut>, buttons: Query<&mut BackgroundColor, With<MenuButton>>) {
    shade(out.entity, BUTTON, buttons);
}

pub fn button_press(
    press: On<PointerPress>,
    buttons: Query<&mut BackgroundColor, With<MenuButton>>,
) {
    shade(press.entity, BUTTON_PRESSED, buttons);
}

pub fn button_release(
    release: On<PointerRelease>,
    buttons: Query<&mut BackgroundColor, With<MenuButton>>,
) {
    shade(release.entity, BUTTON_HOVERED, buttons);
}

fn shade(entity: Entity, color: Color, mut buttons: Query<&mut BackgroundColor, With<MenuButton>>) {
    if let Ok(mut background) = buttons.get_mut(entity) {
        background.set_if_neq(BackgroundColor(color));
    }
}
