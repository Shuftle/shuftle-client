//! The games the player can pick, and what's shown about each of them.

/// A game in the list the player picks from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Game {
    Tressette,
}

impl Game {
    /// Every game, in the order they're listed.
    pub const ALL: [Game; 1] = [Game::Tressette];

    pub fn name(self) -> &'static str {
        match self {
            Game::Tressette => "Tressette",
        }
    }

    /// How the game is played, in paragraphs.
    pub fn rules(self) -> &'static str {
        match self {
            Game::Tressette => TRESSETTE_RULES,
        }
    }
}

const TRESSETTE_RULES: &str = "\
Four players in two teams, partners sitting opposite each other. Play goes \
counter-clockwise, and everyone is dealt ten cards from the 40-card deck.

The player after the dealer leads the first trick. Everyone must follow the \
suit led if they can, and may play any card otherwise. The highest card of \
the suit led takes the trick, and its player leads the next one. From \
highest to lowest: Three, Two, Ace, King, Knight, Jack, Seven, Six, Five, \
Four.

Aces are worth one point each, and Threes, Twos and face cards a third of a \
point each; the other cards are worth nothing. Thirds left over are dropped, \
and the last trick is worth one more point, for 11 points a hand.

A team wins with 31 points or more, as long as it's ahead of the other team. \
Otherwise a new hand is dealt.";
