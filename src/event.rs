mod alert;
pub mod input;

use crate::{
    editor::{self, Editor},
    event,
    grid::{builder::Builder, CellPlacement, Grid},
};
use std::{
    borrow::Cow,
    fs, path,
    time::{Duration, Instant},
};
use terminal::{
    event::Event::{self},
    event::Key,
    Terminal,
};

#[must_use]
pub enum State {
    /// Execution is to be continued normally.
    Continue,
    /// The grid has been solved.
    /// The duration specifies how long it took to solve the grid.
    Solved(Duration),
    /// Display an alert. Alerts are cleared after some time.
    ///
    /// NOTE: alert messages do not end in a period.
    Alert(Cow<'static, str>),
    /// Clear the alert if present.
    ClearAlert,
    /// Halt the game to load a new grid.
    LoadGrid,
    /// Exit the program.
    /// Once the state is evaluated, the instant is immediately converted to a duration which determines whether an exit confirmation prompt needs to be shown.
    Exit(Option<Instant>),
}

pub fn r#loop(terminal: &mut Terminal, builder: &mut Builder) -> State {
    let mut editor = Editor::default();

    let mut alert = None;

    let mut cell_placement = CellPlacement::default();

    loop {
        if let Some(event) = terminal.read_event() {
            // The order of statements matters

            alert::handle_clear_delay(terminal, builder, &mut alert);

            let state = input::handle(
                terminal,
                event,
                builder,
                &mut editor,
                &mut alert,
                &mut cell_placement,
            );

            match state {
                State::Continue => {
                    terminal.flush();
                    continue;
                }
                State::Alert(alert_message) => {
                    alert::draw(terminal, builder, &mut alert, alert_message);
                    terminal.flush();
                }
                State::ClearAlert => {
                    if let Some(mut alert_to_clear) = alert {
                        alert_to_clear.clear(terminal, builder);
                        alert = None;
                    }
                    terminal.flush();
                }
                State::LoadGrid => {
                    match event::input::window::await_dropped_grid_file_path(
                        terminal, builder, &mut alert,
                    ) {
                        Ok(path) => {
                            fn load(path: &str) -> Option<Grid> {
                                let content = fs::read_to_string(&path).ok()?;
                                let grid = editor::load_grid(&content).ok()?;

                                Some(grid)
                            }

                            if let Some(grid) = load(&path) {
                                // Currently the new game simply runs inside of this existing game and the new game creates an entirely new state.
                                // At some point we would probably hit a stack overflow if the user keeps loading new grid files within the same session.

                                terminal.clear();
                                crate::start_game(terminal, grid);

                                break State::Exit(None);
                            } else {
                                let err = if !path.contains(path::MAIN_SEPARATOR) {
                                    // The user likely dropped a grid file onto the window without having pressed
                                    // the L key first so that the path can be properly captured.
                                    "Press L before loading"
                                } else {
                                    "Loading failed"
                                };
                                alert::draw(terminal, builder, &mut alert, err.into());
                                terminal.flush();
                            }
                        }
                        Err(err) => {
                            alert::draw(terminal, builder, &mut alert, err.into());
                            terminal.flush();
                        }
                    }
                }
                State::Solved(_) => break state,
                State::Exit(instant) => {
                    if let Some(instant) = instant {
                        if instant.elapsed().as_secs() > 60 {
                            // If the player played for more than 1 minute, the game is considered to have some kind of value to the player,
                            // so we make sure the player really wants to exit.

                            let message = "Press Enter to confirm exit. Esc to abort".into();

                            alert::draw(terminal, builder, &mut alert, message);

                            terminal.flush();

                            let message = loop {
                                let input = terminal.read_event();

                                match input {
                                    Some(Event::Key(Key::Enter)) => return State::Exit(None),
                                    Some(Event::Key(Key::Esc)) => break "Aborted",
                                    Some(Event::Resize | Event::Mouse(_)) => {}
                                    _ => break "Invalid input. Aborted",
                                }
                            };

                            alert::draw(terminal, builder, &mut alert, message.into());

                            terminal.flush();

                            continue;
                        }
                    }

                    return State::Exit(None);
                }
            }
        }
    }
}
