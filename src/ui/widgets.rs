//! Pieces the screens are built from.

use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui::{Checked, Pressed},
    ui_widgets::{
        Activate, Button, ControlOrientation, RadioButton, RadioGroup, ScrollArea, Scrollbar,
        ScrollbarDragState, ScrollbarThumb,
    },
};

use super::theme::*;
use crate::{
    game::layout::{BAR_HEIGHT, SAFE_TOP},
    state::Screen,
};

/// A button or option shaded by what Bevy's widgets track: pressed, hovered
/// and chosen.
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

fn button(label: &str, font_size: f32, padding: Vec2) -> impl Scene {
    let ignore = Pickable::IGNORE;
    bsn! {
        MenuButton
        Button
        Hovered
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

/// The value an option in a row of choices stands for.
#[derive(Component, Clone, Default)]
pub struct Choice<T: Clone + Default + Send + Sync + Unpin + 'static>(pub T);

/// What an option in a row of choices looks like, above its name.
pub enum Preview {
    /// An image, by its path in the assets, shown at the size of a card.
    Image(String),
    /// A swatch of a colour, the size of a card.
    Swatch(Color),
}

/// An option in a row of choices: the value it stands for, its name, and what
/// it looks like.
pub struct ChoiceOption<T> {
    pub value: T,
    pub name: &'static str,
    pub preview: Preview,
}

/// A row of choices: `label` above a tile per option, all the same size and
/// centred, with the one whose value is `chosen` checked. Picking one triggers
/// `ValueChange<Entity>` on the row, with the tile's entity, whose [`Choice`]
/// holds the value.
pub fn choice_row<T: Clone + Default + PartialEq + Send + Sync + Unpin + 'static>(
    label: &str,
    options: Vec<ChoiceOption<T>>,
    chosen: &T,
) -> impl Scene {
    let options: Vec<_> = options
        .into_iter()
        .map(
            |ChoiceOption {
                 value,
                 name,
                 preview,
             }| {
                let checked = (value == *chosen).then_some(bsn! { Checked });
                bsn! { @choice(name, preview) Choice::<T>(value) @{checked} }
            },
        )
        .collect();
    bsn! {
        Node {
            width: FULL,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(ITEM_GAP),
        }
        Children [
            Text(label)
            TextFont { font_size: FontSize::Px(FONT_BODY) }
            TextColor(TEXT)
            --
            Node {
                width: FULL,
                justify_content: JustifyContent::Center,
                flex_wrap: FlexWrap::Wrap,
                column_gap: px(ITEM_GAP),
                row_gap: px(ITEM_GAP),
            }
            RadioGroup
            Children [ {options} ]
        ]
    }
}

/// An option in a row of choices: a tile with `preview` above `name`, shaded
/// like a button, lighter and with a solid edge while it's the chosen one.
fn choice(name: &str, preview: Preview) -> impl Scene {
    let ignore = Pickable::IGNORE;
    let (image, swatch) = match preview {
        Preview::Image(path) => (Some(bsn! { ImageNode { image: {path} } }), None),
        Preview::Swatch(color) => (None, Some(bsn! { BackgroundColor(color) })),
    };
    bsn! {
        MenuButton
        RadioButton
        Hovered
        Node {
            width: px(CHOICE_WIDTH),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(CHOICE_GAP),
            padding: {UiRect::all(px(CHOICE_PADDING))},
            border: {UiRect::all(px(CHOICE_BORDER))},
            border_radius: {BorderRadius::all(px(BUTTON_RADIUS))},
        }
        BackgroundColor(BUTTON)
        BorderColor::all(BUTTON_BORDER)
        Children [
            @preview_size()
            @{image}
            @{swatch}
            ignore
            --
            Text(name)
            TextFont { font_size: FontSize::Px(FONT_SMALL) }
            TextColor(TEXT)
            // Taps land on the option, not its name.
            ignore
        ]
    }
}

/// A preview in an option, the size of a small card.
fn preview_size() -> impl Scene {
    bsn! {
        Node {
            width: px(PREVIEW_WIDTH),
            height: px(PREVIEW_HEIGHT),
            border_radius: {BorderRadius::all(px(PREVIEW_RADIUS))},
        }
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

/// A full screen shown while in `screen`: a bar at the top with a back arrow
/// and `title`, which stays put, and `body` below, in a centred column, which
/// scrolls when it doesn't fit.
pub fn screen(screen: Screen, title: &str, body: impl SceneList) -> impl Scene {
    let scoped = DespawnOnExit(screen);
    bsn! {
        @full_screen()
        Node {
            flex_direction: FlexDirection::Column,
        }
        scoped
        Children [
            @app_bar(bsn_list! {
                @back_arrow()
                on(back)
                --
                Text(title)
                TextFont { font_size: FontSize::Px(FONT_HEADING) }
                TextColor(TEXT)
            })
            --
            // The rest of the screen, scrolling, with a scrollbar on the right.
            Node {
                width: FULL,
                flex_grow: 1.,
                min_height: px(0),
            }
            Children [
                #Body
                Node {
                    flex_grow: 1.,
                    min_width: px(0),
                    overflow: Overflow::scroll_y(),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: {UiRect::all(px(GUTTER))},
                }
                ScrollArea
                DragScroll
                Children [
                    Node {
                        width: FULL,
                        max_width: px(CONTENT_MAX_WIDTH),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: px(ITEM_GAP),
                    }
                    Children [ {body} ]
                ]
                --
                Node {
                    width: px(SCROLLBAR_WIDTH),
                    margin: {UiRect::axes(px(SCROLLBAR_MARGIN), px(GUTTER))},
                }
                Scrollbar {
                    target: #Body,
                    orientation: ControlOrientation::Vertical,
                    min_thumb_length: SCROLLBAR_MIN_THUMB,
                }
                Children [
                    Hovered
                    ScrollbarThumb {
                        border_radius: {BorderRadius::all(px(SCROLLBAR_WIDTH * 0.5))},
                    }
                    BackgroundColor(THUMB)
                ]
            ]
        ]
    }
}

/// The thumbs whose look may have changed: hovered, dragged, or resized as
/// their content changed.
pub type ThumbChanged = (
    With<ScrollbarThumb>,
    Or<(
        Changed<Hovered>,
        Changed<ScrollbarDragState>,
        Changed<ComputedNode>,
    )>,
);

/// Shades the scrollbar thumbs whose state changed: lighter while hovered or
/// dragged, and hidden while the content fits, when the thumb fills its whole
/// track.
pub fn shade_thumbs(
    mut thumbs: Query<
        (
            &Hovered,
            &ScrollbarDragState,
            &ComputedNode,
            &ChildOf,
            &mut BackgroundColor,
        ),
        ThumbChanged,
    >,
    tracks: Query<&ComputedNode, With<Scrollbar>>,
) {
    for (hovered, drag, thumb, track, mut background) in &mut thumbs {
        let fills_track = tracks
            .get(track.parent())
            .is_ok_and(|track| thumb.size().y >= track.size().y);
        let color = if fills_track {
            Color::NONE
        } else if hovered.get() || drag.dragging {
            THUMB_ACTIVE
        } else {
            THUMB
        };
        background.set_if_neq(BackgroundColor(color));
    }
}

/// The bar at the top of every screen and of the table, below the room phones
/// keep for the status bar and camera, with `content` in a row.
pub fn app_bar(content: impl SceneList) -> impl Scene {
    bsn! {
        Node {
            width: FULL,
            height: {px(SAFE_TOP + BAR_HEIGHT)},
            align_items: AlignItems::Center,
            column_gap: px(ITEM_GAP),
            padding: {UiRect {
                top: px(SAFE_TOP),
                left: px(GUTTER),
                right: px(GUTTER),
                ..default()
            }},
        }
        BackgroundColor(BAR)
        Children [ {content} ]
    }
}

/// The arrow back to the screen above.
pub fn back_arrow() -> impl Scene {
    icon_button("<")
}

/// A square button showing a single sign, as big as Android's minimum touch
/// target.
pub fn icon_button(sign: &str) -> impl Scene {
    bsn! {
        @button(sign, FONT_HEADING, Vec2::ZERO)
        Node {
            width: px(MIN_TAP),
            height: px(MIN_TAP),
        }
    }
}

/// A node that scrolls when it's dragged.
#[derive(Component, Default, Clone)]
pub struct DragScroll {
    /// Where it was scrolled to when the drag started.
    start: Vec2,
}

/// Remembers where a dragged node was scrolled to.
pub fn drag_scroll_start(
    drag: On<PointerDragStart>,
    mut nodes: Query<(&ComputedNode, &mut DragScroll)>,
) {
    if let Ok((node, mut scroll)) = nodes.get_mut(drag.entity) {
        scroll.start = node.scroll_position * node.inverse_scale_factor;
    }
}

/// Scrolls a dragged node by as much as the pointer moved, the other way.
pub fn drag_scroll(
    drag: On<PointerDrag>,
    ui_scale: Res<UiScale>,
    mut nodes: Query<(&DragScroll, &mut ScrollPosition)>,
) {
    if let Ok((scroll, mut position)) = nodes.get_mut(drag.entity) {
        position.0 = (scroll.start - drag.distance / ui_scale.0).max(Vec2::ZERO);
    }
}

/// Goes to the screen above this one.
fn back(_: On<Activate>, screen: Res<State<Screen>>, mut next: ResMut<NextState<Screen>>) {
    if let Some(parent) = screen.get().parent() {
        next.set(parent);
    }
}

/// The buttons and options whose look may have changed: pressed, hovered or
/// chosen, as Bevy's widgets track it.
pub type ShadeChanged = (
    With<MenuButton>,
    Or<(Changed<Pressed>, Changed<Hovered>, Added<Checked>)>,
);

/// Shades the buttons and options whose state changed, or that lost `Pressed`
/// or `Checked`.
pub fn shade_buttons(
    mut buttons: Query<
        (
            Has<Pressed>,
            &Hovered,
            Has<Checked>,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<MenuButton>,
    >,
    changed: Query<Entity, ShadeChanged>,
    mut unpressed: RemovedComponents<Pressed>,
    mut unchecked: RemovedComponents<Checked>,
) {
    let entities: Vec<Entity> = changed
        .iter()
        .chain(unpressed.read())
        .chain(unchecked.read())
        .collect();
    for entity in entities {
        if let Ok((pressed, hovered, checked, mut background, mut border)) = buttons.get_mut(entity)
        {
            let (color, edge) = shade(pressed, hovered.get(), checked);
            background.set_if_neq(BackgroundColor(color));
            border.set_if_neq(BorderColor::all(edge));
        }
    }
}

/// Returns the background and edge of a button or option: darker while the
/// pointer is over it or presses it, and lighter with a solid edge while it's
/// the chosen option.
fn shade(pressed: bool, hovered: bool, checked: bool) -> (Color, Color) {
    let background = match (pressed, hovered, checked) {
        (true, _, _) => BUTTON_PRESSED,
        (false, true, _) => BUTTON_HOVERED,
        (false, false, true) => CHOSEN,
        (false, false, false) => BUTTON,
    };
    let edge = if checked {
        CHOSEN_BORDER
    } else {
        BUTTON_BORDER
    };
    (background, edge)
}
