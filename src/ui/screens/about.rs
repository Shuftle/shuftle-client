//! What the game is, under which licences, and where to find more.

use bevy::{prelude::*, text::LineHeight, ui_widgets::Activate};

use crate::{
    state::Screen,
    ui::{
        theme::{
            FONT_BODY, FONT_SMALL, FULL, ITEM_GAP, PARAGRAPH_GAP, PARAGRAPH_LINE_HEIGHT,
            SECTION_GAP, TEXT, TEXT_DIM,
        },
        widgets::{menu_button, screen},
    },
};

const ABOUT: [&str; 3] = [
    "Shuftle is a collection of turn-based games. It's free software: free as in \
     free speech, and free as in free beer.",
    "Made with Bevy, written in Rust.",
    "The code is licensed under the GNU AGPLv3. The art keeps its own \
     licences, listed in the README of shuftle-client on GitHub.",
];

/// Where the code is: the organisation, with the game and its libraries.
const SOURCE_URL: &str = "https://github.com/Shuftle";
/// The game's page on the web.
const WEBSITE_URL: &str = "https://vitkorp.eu/shuftle";

/// The text at the top, and the links with the version under them at the
/// bottom.
pub fn scene() -> impl Scene {
    let paragraphs: Vec<_> = ABOUT
        .into_iter()
        .map(|paragraph| {
            let line_height = LineHeight::RelativeToFont(PARAGRAPH_LINE_HEIGHT);
            bsn! {
                Text({paragraph})
                TextFont { font_size: FontSize::Px(FONT_BODY) }
                TextColor(TEXT)
                line_height
                Node { width: FULL }
            }
        })
        .collect();
    let version = format!("Version {}", env!("CARGO_PKG_VERSION"));
    screen(
        Screen::About,
        "About",
        bsn_list! {
            Node {
                width: FULL,
                flex_grow: 1.,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                row_gap: px(SECTION_GAP),
            }
            Children [
                Node {
                    width: FULL,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(PARAGRAPH_GAP),
                }
                Children [ {paragraphs} ]
                --
                Node {
                    width: FULL,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(SECTION_GAP),
                }
                Children [
                    Node {
                        width: FULL,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: px(ITEM_GAP),
                    }
                    Children [
                        @menu_button("Source code") on(open(SOURCE_URL))
                        --
                        @menu_button("Website") on(open(WEBSITE_URL))
                    ]
                    --
                    Text({version})
                    TextFont { font_size: FontSize::Px(FONT_SMALL) }
                    TextColor(TEXT_DIM)
                ]
            ]
        },
    )
}

/// Returns what opens `url` in the system's browser when a button is
/// activated.
fn open(url: &'static str) -> impl Fn(On<Activate>) + Clone {
    move |_| {
        if let Err(e) = webbrowser::open(url) {
            warn!("Couldn't open {url}: {e}");
        }
    }
}
