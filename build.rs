//! Checks that the card sets the game ships are in `assets/`, and describes
//! them to the game. A release build fails when one of their files is missing;
//! other builds only warn.

use std::{
    env,
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
};

/// Where the assets are, from the package's folder.
const ASSETS: &str = "assets";

/// Where the Italian decks and backs are, in the assets.
const ITALIAN_DECKS_DIR: &str = "cards/italian/decks";
const ITALIAN_BACKS_DIR: &str = "cards/italian/backs";

/// The Italian decks the game ships, the default first: their folder in
/// `ITALIAN_DECKS_DIR`, and what the game calls them.
const ITALIAN_DECKS: [(&str, &str); 3] = [
    ("siciliane/maurizio-grillo", "Siciliane"),
    ("napoletane/trocche100", "Napoletane"),
    ("paris/svg-cards", "Paris"),
];

/// The Italian backs the game ships, the default first: their folder in
/// `ITALIAN_BACKS_DIR`, and what the game calls them.
const ITALIAN_BACKS: [(&str, &str); 4] = [
    ("diamonds", "Diamonds"),
    ("dots", "Dots"),
    ("lattice", "Lattice"),
    ("stars", "Stars"),
];

/// The Italian suits and ranks: as shuftlib names them, and as the files do.
const ITALIAN_SUITS: [(&str, &str); 4] = [
    ("Coins", "coins"),
    ("Cups", "cups"),
    ("Swords", "swords"),
    ("Clubs", "clubs"),
];
const ITALIAN_RANKS: [(&str, u8); 10] = [
    ("Ace", 1),
    ("Two", 2),
    ("Three", 3),
    ("Four", 4),
    ("Five", 5),
    ("Six", 6),
    ("Seven", 7),
    ("Jack", 8),
    ("Knight", 9),
    ("King", 10),
];

/// The file of a back's image, in its folder.
const BACK_FILE: &str = "back.png";
/// Each set's credits, which ship beside it.
const CREDITS: &str = "CREDITS.md";

/// The size every card image shares, in pixels: the Italian regional cards'
/// shape, 512 pixels tall.
const CARD_WIDTH: u32 = 312;
const CARD_HEIGHT: u32 = 512;

/// Where a PNG's header stores its width and height, as big-endian `u32`s.
const PNG_WIDTH_AT: usize = 16;
const PNG_HEIGHT_AT: usize = 20;

/// The `PROFILE` Cargo gives the build scripts of release builds.
const RELEASE: &str = "release";

fn main() {
    println!("cargo:rerun-if-changed={ASSETS}");
    let assets = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join(ASSETS);

    let faces: Vec<String> = ITALIAN_SUITS
        .iter()
        .flat_map(|(_, suit)| ITALIAN_RANKS.iter().map(|(_, rank)| face_file(suit, *rank)))
        .collect();
    for (id, _) in ITALIAN_DECKS {
        check_set(&assets.join(ITALIAN_DECKS_DIR).join(id), &faces);
    }
    for (id, _) in ITALIAN_BACKS {
        check_set(
            &assets.join(ITALIAN_BACKS_DIR).join(id),
            &[BACK_FILE.into()],
        );
    }

    let mut code = format!(
        "/// Where the Italian decks and backs are, in the assets.\n\
         pub const ITALIAN_DECKS_DIR: &str = {ITALIAN_DECKS_DIR:?};\n\
         pub const ITALIAN_BACKS_DIR: &str = {ITALIAN_BACKS_DIR:?};\n\
         /// The file of a back's image, in its folder.\n\
         pub const BACK_FILE: &str = {BACK_FILE:?};\n\
         /// The size every card image shares, in pixels.\n\
         pub const CARD_WIDTH: u32 = {CARD_WIDTH};\n\
         pub const CARD_HEIGHT: u32 = {CARD_HEIGHT};\n"
    );
    code += "/// The Italian decks the game ships.\npub const ITALIAN_DECKS: &[Deck] = &[\n";
    for (id, name) in ITALIAN_DECKS {
        writeln!(code, "    Deck {{ id: {id:?}, name: {name:?} }},").unwrap();
    }
    code += "];\n/// The Italian backs the game ships.\npub const ITALIAN_BACKS: &[Back] = &[\n";
    for (id, name) in ITALIAN_BACKS {
        writeln!(code, "    Back {{ id: {id:?}, name: {name:?} }},").unwrap();
    }
    code += "];\n\
             /// The deck and back a game uses until the player picks others.\n\
             pub const DEFAULT_ITALIAN_DECK: &Deck = &ITALIAN_DECKS[0];\n\
             pub const DEFAULT_ITALIAN_BACK: &Back = &ITALIAN_BACKS[0];\n\
             /// Returns the file of the face with `suit` and `rank` in an Italian deck.\n\
             pub fn italian_face(suit: Suit, rank: ItalianRank) -> &'static str {\n    \
             match (suit, rank) {\n";
    for (suit_variant, suit) in ITALIAN_SUITS {
        for (rank_variant, rank) in ITALIAN_RANKS {
            writeln!(
                code,
                "        (Suit::{suit_variant}, ItalianRank::{rank_variant}) => {:?},",
                face_file(suit, rank)
            )
            .unwrap();
        }
    }
    code += "    }\n}\n";

    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("cards.rs"),
        code,
    )
    .unwrap();
}

/// Returns the file of the face of `suit` and `rank` in a deck.
fn face_file(suit: &str, rank: u8) -> String {
    format!("{suit}-{rank}.png")
}

/// Checks the set in `dir` has every one of `files`, at the size every card
/// shares, and its credits.
fn check_set(dir: &Path, files: &[String]) {
    if !dir.is_dir() {
        missing(&format!("no {}", dir.display()));
        return;
    }
    let absent: Vec<&str> = files
        .iter()
        .filter(|file| !dir.join(file).is_file())
        .map(String::as_str)
        .collect();
    if !absent.is_empty() {
        missing(&format!("{} misses {}", dir.display(), absent.join(", ")));
    }
    for file in files.iter().filter(|file| dir.join(file).is_file()) {
        check_size(&dir.join(file));
    }
    if !dir.join(CREDITS).is_file() {
        warn(&format!("{} has no {CREDITS}", dir.display()));
    }
}

/// Warns if the PNG at `path` isn't the size every card shares.
fn check_size(path: &Path) {
    let Ok(bytes) = fs::read(path) else {
        warn(&format!("can't read {}", path.display()));
        return;
    };
    let read = |at: usize| {
        bytes
            .get(at..at + 4)
            .map(|be| u32::from_be_bytes(be.try_into().unwrap()))
    };
    let (Some(width), Some(height)) = (read(PNG_WIDTH_AT), read(PNG_HEIGHT_AT)) else {
        warn(&format!("{} isn't a PNG", path.display()));
        return;
    };
    if (width, height) != (CARD_WIDTH, CARD_HEIGHT) {
        warn(&format!(
            "{} is {width}x{height}, not {CARD_WIDTH}x{CARD_HEIGHT}",
            path.display()
        ));
    }
}

fn warn(message: &str) {
    println!("cargo:warning={message}");
}

/// Fails a release build with `message`, and only warns in other builds.
fn missing(message: &str) {
    if env::var("PROFILE").is_ok_and(|profile| profile == RELEASE) {
        println!("cargo::error={message}");
    } else {
        warn(message);
    }
}
