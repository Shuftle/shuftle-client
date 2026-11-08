//! What's shown over the table during a game: the bar at the top, with the
//! score and the way back to the menu, the dialog confirming the player wants
//! to leave, and the restart button once the game is over.

use bevy::{
    input::keyboard::KeyboardInput,
    input_focus::{FocusedInput, tab_navigation::TabIndex},
    prelude::*,
    ui_widgets::{Activate, ModalDialog, ModalDialogBarrier, RequestClose},
};

use super::{
    theme::{
        BAR, BUTTON_RADIUS, DIALOG_MAX_WIDTH, DIALOG_PADDING, FONT_BODY, FONT_HEADING, FULL,
        GUTTER, ITEM_GAP, SCRIM, TEXT,
    },
    widgets::{app_bar, back_arrow, full_screen, menu_button},
};
use crate::{
    game::logic::{GameOver, RestartGame, ShownScore, THEM_COLOR, US_COLOR},
    state::{AppState, LeaveDialog, is_back},
};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::InGame),
            (top_bar.spawn(), restart_overlay.spawn()),
        )
        .add_systems(OnEnter(LeaveDialog::Open), leave_dialog.spawn())
        .add_systems(
            Update,
            show_score.run_if(in_state(AppState::InGame).and_then(resource_changed::<ShownScore>)),
        )
        .add_observer(show_restart)
        .add_observer(back.run_if(in_state(AppState::InGame)));
    }
}

/// The bar at the top: the arrow back to the menu, and the score on the right.
fn top_bar() -> impl Scene {
    let scoped = DespawnOnExit(AppState::InGame);
    bsn! {
        @app_bar(bsn_list! {
            @back_arrow()
            on(ask_to_leave)
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

/// Asks whether to leave the game, in a box over the dimmed table, which
/// keeps going behind it and can't be tapped. Esc, or a tap outside the box,
/// closes it, and Enter picks the focused button, Stay at first.
fn leave_dialog() -> impl Scene {
    let scoped = DespawnOnExit(LeaveDialog::Open);
    bsn! {
        @full_screen()
        Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: {UiRect::all(px(GUTTER))},
        }
        BackgroundColor(SCRIM)
        ModalDialogBarrier
        scoped
        on(close)
        Children [
            Node {
                width: FULL,
                max_width: px(DIALOG_MAX_WIDTH),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(ITEM_GAP),
                padding: {UiRect::all(px(DIALOG_PADDING))},
                border_radius: {BorderRadius::all(px(BUTTON_RADIUS))},
            }
            BackgroundColor(BAR)
            ModalDialog
            on(ignore_released_esc)
            Children [
                Text("Leave the game?")
                TextFont { font_size: FontSize::Px(FONT_HEADING) }
                TextColor(TEXT)
                --
                @menu_button("Leave") TabIndex(0) on(leave)
                --
                @menu_button("Stay") TabIndex(0) on(stay)
            ]
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

/// Called when the back arrow in the top bar is activated: asks whether to
/// leave the game.
fn ask_to_leave(_: On<Activate>, mut next: ResMut<NextState<LeaveDialog>>) {
    next.set(LeaveDialog::Open);
}

/// Leaves the game for the main menu.
fn leave(_: On<Activate>, mut next: ResMut<NextState<AppState>>) {
    next.set(AppState::Menu);
}

/// Closes the dialog, back to the game.
fn stay(_: On<Activate>, mut next: ResMut<NextState<LeaveDialog>>) {
    next.set(LeaveDialog::Closed);
}

/// Stops a released Esc at the dialog's box, before it reaches the barrier,
/// which closes the dialog on releases too: the release of the Esc that
/// opened it would close it straight away.
// TODO: remove once Bevy's `ModalDialogBarrier` only closes on a pressed Esc.
fn ignore_released_esc(mut key: On<FocusedInput<KeyboardInput>>) {
    let input = &key.input;
    if input.key_code == KeyCode::Escape && !input.state.is_pressed() {
        key.propagate(false);
    }
}

/// Closes the dialog when it asks to be closed.
fn close(_: On<RequestClose>, mut next: ResMut<NextState<LeaveDialog>>) {
    next.set(LeaveDialog::Closed);
}

/// On Esc, or Android's back gesture, once it reaches the window, asks
/// whether to leave the game, or closes the dialog if it's asking already.
fn back(
    key: On<FocusedInput<KeyboardInput>>,
    windows: Query<(), With<Window>>,
    dialog: Res<State<LeaveDialog>>,
    mut next: ResMut<NextState<LeaveDialog>>,
) {
    if is_back(&key, &windows) {
        next.set(match dialog.get() {
            LeaveDialog::Closed => LeaveDialog::Open,
            LeaveDialog::Open => LeaveDialog::Closed,
        });
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
