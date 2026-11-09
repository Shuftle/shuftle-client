//! What's shown over the table during a game: the bar at the top, with the
//! score and the way back to the menu, and the restart button once the game
//! is over.

use bevy::{
    input::keyboard::KeyboardInput, input_focus::FocusedInput, prelude::*, ui_widgets::Activate,
};

use super::{
    theme::FONT_BODY,
    widgets::{app_bar, back_arrow, full_screen, menu_button},
};
use crate::{
    game::logic::{GameOver, RestartGame, ShownScore, THEM_COLOR, US_COLOR},
    state::{AppState, is_back},
};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::InGame),
            (top_bar.spawn(), restart_overlay.spawn()),
        )
        .add_systems(
            Update,
            show_score.run_if(in_state(AppState::InGame).and_then(resource_changed::<ShownScore>)),
        )
        .add_observer(show_restart)
        .add_observer(back_to_menu.run_if(in_state(AppState::InGame)));
    }
}

/// The bar at the top: the arrow back to the menu, and the score on the right.
fn top_bar() -> impl Scene {
    let scoped = DespawnOnExit(AppState::InGame);
    bsn! {
        @app_bar(bsn_list! {
            @back_arrow()
            on(open_menu)
            --
            // Pushes the score to the right.
            Node { flex_grow: 1. }
            --
            Text
            TextFont { font_size: FontSize::Px(FONT_BODY) }
            ScoreText
            Children [
                TextSpan
                TextFont { font_size: FontSize::Px(FONT_BODY) }
                TextColor({US_COLOR})
                UsScore
                --
                TextSpan("   ")
                TextFont { font_size: FontSize::Px(FONT_BODY) }
                --
                TextSpan
                TextFont { font_size: FontSize::Px(FONT_BODY) }
                TextColor({THEM_COLOR})
                ThemScore
            ]
        })
        Node { position_type: PositionType::Absolute }
        scoped
    }
}

/// The restart button, centred on the table and hidden until the game is
/// over. What centres it covers the screen, so it lets taps through.
fn restart_overlay() -> impl Scene {
    let scoped = DespawnOnExit(AppState::InGame);
    let ignore = Pickable::IGNORE;
    bsn! {
        @full_screen()
        Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        ignore
        scoped
        Children [
            @menu_button("Restart Game")
            Node { display: Display::None }
            RestartButton
            on(restart)
        ]
    }
}

/// Writes the score in the top bar.
fn show_score(
    shown: Res<ShownScore>,
    mut label: Single<&mut Text, With<ScoreText>>,
    mut us: Single<&mut TextSpan, (With<UsScore>, Without<ThemScore>)>,
    mut them: Single<&mut TextSpan, With<ThemScore>>,
) {
    label.0 = if shown.is_final { "Final  " } else { "" }.into();
    us.0 = format!("Us {}", shown.us);
    them.0 = format!("Them {}", shown.them);
}

/// Shows the restart button once the game is over.
fn show_restart(_: On<GameOver>, mut button: Single<&mut Node, With<RestartButton>>) {
    button.display = Display::Flex;
}

/// Called when the restart button is activated: it hides, and a new game
/// starts.
fn restart(
    activate: On<Activate>,
    mut buttons: Query<&mut Node, With<RestartButton>>,
    mut commands: Commands,
) {
    if let Ok(mut node) = buttons.get_mut(activate.entity) {
        node.display = Display::None;
    }
    commands.trigger(RestartGame);
}

/// Called when the menu button in the top bar is activated: the game is left
/// for the main menu.
fn open_menu(_: On<Activate>, mut next: ResMut<NextState<AppState>>) {
    next.set(AppState::Menu);
}

/// Leaves the game for the main menu on Esc, or Android's back gesture.
fn back_to_menu(
    key: On<FocusedInput<KeyboardInput>>,
    windows: Query<(), With<Window>>,
    mut next: ResMut<NextState<AppState>>,
) {
    if is_back(&key, &windows) {
        next.set(AppState::Menu);
    }
}

/// The POV player's team's score, and the other team's, in the top bar.
#[derive(Component, Default, Clone)]
struct UsScore;

#[derive(Component, Default, Clone)]
struct ThemScore;

#[derive(Component, Default, Clone)]
struct ScoreText;

/// The restart button, shown only once the game is over.
#[derive(Component, Default, Clone)]
struct RestartButton;
