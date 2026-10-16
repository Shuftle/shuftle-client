//! Where things go on the table, worked out from the window size so that
//! portrait phones, landscape screens and resized windows all get a layout
//! that fits.
//!
//! Positions are in world units, which match logical pixels, with the origin
//! at the centre of the screen.

use bevy::{prelude::*, window::PrimaryWindow};

/// Width over height of the card images.
const CARD_ASPECT: f32 = 2. / 3.;

/// Room at the top that phones keep for the status bar and the selfie camera,
/// which the game draws under but can't put anything to see or tap in. It's
/// the status bar's height on current phones: their cameras sit inside it.
pub const SAFE_TOP: f32 = if cfg!(target_os = "android") { 48. } else { 0. };

/// Height of the bar at the top for the score, below `SAFE_TOP`.
pub const BAR_HEIGHT: f32 = 40.;

pub struct LayoutPlugin;

impl Plugin for LayoutPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TableLayout::new(Vec2::new(1280., 720.), SAFE_TOP))
            .add_systems(PreUpdate, update_layout);
    }
}

/// Recomputes the layout when the window is resized or rotated.
fn update_layout(window: Single<&Window, With<PrimaryWindow>>, mut layout: ResMut<TableLayout>) {
    let size = window.size();
    // The window can report no size while it's being created.
    if size.min_element() > 0. && size != layout.size {
        *layout = TableLayout::new(size, SAFE_TOP);
    }
}

/// A side of the screen, where a player sits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Bottom,
    Right,
    Top,
    Left,
}

impl Side {
    pub const ALL: [Side; 4] = [Side::Bottom, Side::Right, Side::Top, Side::Left];
}

/// Somewhere a card can be on the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    /// The `index`th of the `count` cards in the POV player's hand.
    Hand { index: usize, count: usize },
    /// Played in the trick by the player at this side.
    Trick(Side),
    /// The card back standing for the hand of the player at this side, also
    /// where the tricks they take go. The POV player has none on screen: theirs
    /// is just below the bottom edge.
    Marker(Side),
}

/// The table's layout for the current window size.
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct TableLayout {
    pub size: Vec2,
    hand_card: Vec2,
    trick_card: Vec2,
    marker_card: Vec2,
    gap: f32,
    /// Most hand cards on a row.
    columns: usize,
    /// Centre of the bottom row of the hand.
    hand_bottom_row: f32,
    /// Bounds of the area above the hand, as min and max corners.
    table_min: Vec2,
    table_max: Vec2,
    trick_center: Vec2,
    /// Height kept free at the top of the screen, for the top bar.
    top: f32,
}

impl TableLayout {
    /// Lays the table out for a screen of `size`: the hand at the bottom, as
    /// large as fits, the other players' markers along the remaining edges, and
    /// the trick between them. The top bar goes below `safe_top`.
    pub fn new(size: Vec2, safe_top: f32) -> Self {
        let margin = (size.min_element() * 0.03).max(4.);
        let gap = margin * 0.5;
        let half = size * 0.5;

        // Phones held upright have room for two rows of five big cards; wide
        // screens fit all ten in a row.
        let portrait = size.y > size.x;
        let (columns, rows, hand_share) = if portrait {
            (5, 2, 0.45)
        } else {
            (10, 1, 0.35)
        };
        let hand_height = size.y * hand_share - margin;
        let width_bound = (size.x - 2. * margin - (columns - 1) as f32 * gap) / columns as f32;
        let height_bound = (hand_height - (rows - 1) as f32 * gap) / rows as f32;
        let hand_card = card_of_width(width_bound.min(height_bound * CARD_ASPECT));
        let hand_bottom_row = -half.y + margin + hand_card.y * 0.5;

        let hand_top = -half.y + margin + rows as f32 * hand_card.y + (rows - 1) as f32 * gap;
        let table_min = Vec2::new(-half.x + margin, hand_top + gap);
        let top = safe_top + BAR_HEIGHT;
        let table_max = half - margin - Vec2::Y * top;
        let table_size = table_max - table_min;

        let marker_card = card_of_height((table_size.min_element() * 0.18).min(hand_card.y * 0.6));

        // The trick fills what the markers leave, in a diamond three cards wide
        // and two tall, but never with cards bigger than the hand's.
        let trick_width = table_size.x - 2. * (marker_card.x + gap);
        let trick_height = table_size.y - (marker_card.y + gap);
        let trick_card = card_of_width(
            ((trick_width - 2. * gap) / 3.)
                .min((trick_height - gap) * 0.5 * CARD_ASPECT)
                .min(hand_card.x),
        );
        let trick_center = Vec2::new(
            (table_min.x + table_max.x) * 0.5,
            (table_min.y + table_max.y - marker_card.y - gap) * 0.5,
        );

        Self {
            size,
            hand_card,
            trick_card,
            marker_card,
            gap,
            columns,
            hand_bottom_row,
            table_min,
            table_max,
            trick_center,
            top,
        }
    }

    /// Returns where the centre of a card at `place` goes.
    pub fn position(&self, place: Place) -> Vec2 {
        match place {
            Place::Hand { index, count } => self.hand_position(index, count),
            Place::Trick(side) => {
                let card = self.trick_card;
                self.trick_center
                    + match side {
                        Side::Bottom => Vec2::new(0., -(card.y + self.gap) * 0.5),
                        Side::Right => Vec2::new(card.x + self.gap, 0.),
                        Side::Top => Vec2::new(0., (card.y + self.gap) * 0.5),
                        Side::Left => Vec2::new(-card.x - self.gap, 0.),
                    }
            }
            Place::Marker(side) => {
                let card = self.marker_card;
                let middle = (self.table_min.y + self.table_max.y) * 0.5;
                match side {
                    Side::Bottom => Vec2::new(0., -self.size.y * 0.5 - card.y),
                    Side::Right => Vec2::new(self.table_max.x - card.x * 0.5, middle),
                    Side::Top => Vec2::new(0., self.table_max.y - card.y * 0.5),
                    Side::Left => Vec2::new(self.table_min.x + card.x * 0.5, middle),
                }
            }
        }
    }

    /// Returns the size of a card at `place`.
    pub fn card_size(&self, place: Place) -> Vec2 {
        match place {
            Place::Hand { .. } => self.hand_card,
            Place::Trick(_) => self.trick_card,
            Place::Marker(_) => self.marker_card,
        }
    }

    /// Spreads `count` cards evenly over as few rows as fit, in reading order,
    /// each row centred.
    fn hand_position(&self, index: usize, count: usize) -> Vec2 {
        let rows = count.div_ceil(self.columns).max(1);
        let per_row = count.div_ceil(rows).max(1);
        let row = index / per_row;
        let in_row = per_row.min(count - row * per_row);
        let column = (index % per_row) as f32 - (in_row - 1) as f32 * 0.5;
        let rows_below = (rows - 1 - row) as f32;
        Vec2::new(
            column * (self.hand_card.x + self.gap),
            self.hand_bottom_row + rows_below * (self.hand_card.y + self.gap),
        )
    }
}

/// Returns the size of a card `width` wide. Screens too small to fit anything
/// get empty cards rather than negative ones.
fn card_of_width(width: f32) -> Vec2 {
    let width = width.max(0.);
    Vec2::new(width, width / CARD_ASPECT)
}

fn card_of_height(height: f32) -> Vec2 {
    card_of_width(height * CARD_ASPECT)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Phone screens, upright and sideways, in logical pixels.
    const PHONES: [Vec2; 6] = [
        // Small.
        Vec2::new(360., 640.),
        Vec2::new(640., 360.),
        // Pixel 9a.
        Vec2::new(411., 923.),
        Vec2::new(923., 411.),
        // Large.
        Vec2::new(430., 932.),
        Vec2::new(932., 430.),
    ];

    /// Other screens the layout must handle.
    const OTHERS: [Vec2; 7] = [
        // A foldable's inner screen, almost square.
        Vec2::new(673., 841.),
        // Tablet, upright and sideways.
        Vec2::new(800., 1280.),
        Vec2::new(1280., 800.),
        // Desktop.
        Vec2::new(1280., 720.),
        Vec2::new(1920., 1080.),
        // Smallest desktop window.
        Vec2::new(640., 400.),
        // Very wide and short.
        Vec2::new(1600., 400.),
    ];

    /// Every screen, with and without room for a phone's status bar.
    fn layouts() -> impl Iterator<Item = TableLayout> {
        PHONES
            .into_iter()
            .chain(OTHERS)
            .flat_map(|size| [0., 48.].map(|safe_top| TableLayout::new(size, safe_top)))
    }

    fn rect(layout: &TableLayout, place: Place) -> Rect {
        Rect::from_center_size(layout.position(place), layout.card_size(place))
    }

    /// The cards of a hand of `count`.
    fn hand(count: usize) -> impl Iterator<Item = Place> {
        (0..count).map(move |index| Place::Hand { index, count })
    }

    /// Everything on the table but the hand: the trick and the markers on
    /// screen.
    fn table() -> Vec<Place> {
        Side::ALL
            .map(Place::Trick)
            .into_iter()
            .chain([Side::Right, Side::Top, Side::Left].map(Place::Marker))
            .collect()
    }

    /// The screen below the top bar.
    fn screen(layout: &TableLayout) -> Rect {
        let half = layout.size * 0.5;
        Rect::from_corners(-half, half - Vec2::Y * layout.top)
    }

    fn inside(outer: Rect, inner: Rect) -> bool {
        outer.contains(inner.min) && outer.contains(inner.max)
    }

    fn overlap(a: Rect, b: Rect) -> bool {
        // Touching edges don't count.
        let inner = a.intersect(b);
        inner.width() > 0.01 && inner.height() > 0.01
    }

    #[test]
    fn everything_but_the_bottom_marker_is_on_screen() {
        for layout in layouts() {
            let size = layout.size;
            let screen = screen(&layout);
            for place in table() {
                assert!(
                    inside(screen, rect(&layout, place)),
                    "{place:?} off a {size} screen"
                );
            }
            for count in 1..=10 {
                for place in hand(count) {
                    assert!(
                        inside(screen, rect(&layout, place)),
                        "{place:?} off a {size} screen"
                    );
                }
            }
        }
    }

    #[test]
    fn nothing_overlaps() {
        for layout in layouts() {
            let size = layout.size;
            let table = table();
            for (i, &a) in table.iter().enumerate() {
                for &b in &table[i + 1..] {
                    assert!(
                        !overlap(rect(&layout, a), rect(&layout, b)),
                        "{a:?} and {b:?} overlap on a {size} screen"
                    );
                }
            }
            for count in 1..=10 {
                let hand: Vec<_> = hand(count).collect();
                for (i, &a) in hand.iter().enumerate() {
                    for &b in &hand[i + 1..] {
                        assert!(
                            !overlap(rect(&layout, a), rect(&layout, b)),
                            "{a:?} and {b:?} overlap on a {size} screen"
                        );
                    }
                    for &other in &table {
                        assert!(
                            !overlap(rect(&layout, a), rect(&layout, other)),
                            "{a:?} and {other:?} overlap on a {size} screen"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn hand_cards_are_big_enough_to_tap() {
        // Android's minimum touch target is 48dp, which are logical pixels.
        for size in PHONES {
            let card = TableLayout::new(size, 48.).card_size(Place::Hand {
                index: 0,
                count: 10,
            });
            assert!(card.x >= 48., "{card} cards on a {size} phone");
        }
    }

    /// Returns the rows of a hand of `count`, from the top, as the indices of
    /// their cards.
    fn rows(layout: &TableLayout, count: usize) -> Vec<Vec<usize>> {
        let mut rows: Vec<(f32, Vec<usize>)> = Vec::new();
        for (index, place) in hand(count).enumerate() {
            let y = layout.position(place).y;
            match rows.iter_mut().find(|(row, _)| *row == y) {
                Some((_, cards)) => cards.push(index),
                None => rows.push((y, vec![index])),
            }
        }
        rows.sort_by(|a, b| b.0.total_cmp(&a.0));
        rows.into_iter().map(|(_, cards)| cards).collect()
    }

    #[test]
    fn portrait_hands_take_two_rows_of_five() {
        let layout = TableLayout::new(Vec2::new(411., 923.), 48.);
        assert_eq!(
            rows(&layout, 10),
            [vec![0, 1, 2, 3, 4], vec![5, 6, 7, 8, 9]]
        );
        // As cards are played the rows even out, then merge.
        assert_eq!(rows(&layout, 7), [vec![0, 1, 2, 3], vec![4, 5, 6]]);
        assert_eq!(rows(&layout, 5), [vec![0, 1, 2, 3, 4]]);
    }

    #[test]
    fn landscape_hands_take_one_row() {
        let layout = TableLayout::new(Vec2::new(1280., 720.), 0.);
        assert_eq!(rows(&layout, 10), [Vec::from_iter(0..10)]);
    }

    #[test]
    fn trick_cards_keep_apart() {
        for layout in layouts() {
            let size = layout.size;
            for (i, &a) in Side::ALL.iter().enumerate() {
                for &b in &Side::ALL[i + 1..] {
                    let a = rect(&layout, Place::Trick(a)).inflate(0.5);
                    let b = rect(&layout, Place::Trick(b));
                    assert!(!overlap(a, b), "{a:?} touches {b:?} on a {size} screen");
                }
            }
        }
    }

    #[test]
    fn tiny_screens_get_no_negative_sizes() {
        // A browser can squeeze the canvas to nothing.
        for size in [Vec2::new(60., 60.), Vec2::new(1., 1.), Vec2::new(300., 20.)] {
            let layout = TableLayout::new(size, 0.);
            let mut places = table();
            places.extend(hand(10));
            for place in places {
                let card = layout.card_size(place);
                assert!(
                    card.min_element() >= 0.,
                    "{place:?} is {card} on a {size} screen"
                );
            }
        }
    }
}
