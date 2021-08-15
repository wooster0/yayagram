mod alert;
pub mod input;

use crate::{
    editor::{self, Editor},
    event,
    grid::{builder::Builder, CellPlacement, Grid},
    start_game,
};
use alert::Alert;
use std::{borrow::Cow, fs, path, time::Duration};
use terminal::Terminal;

#[must_use]
pub enum State {
    /// Execution is to be continued normally.
    Continue,
    /// The grid has been solved.
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
    Exit,
}

pub fn r#loop(terminal: &mut Terminal, builder: &mut Builder) -> State {
    let mut editor = Editor::default();

    let mut alert: Option<Alert> = None;

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
                &alert,
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
                                start_game(terminal, grid);

                                break State::Exit;
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
                State::Solved(_) | State::Exit => break state,
            }
        }
    }
}
