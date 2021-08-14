mod alert;
pub mod input;

use crate::{
    editor::{self, Editor},
    grid::{builder::Builder, CellPlacement, Grid},
    start_game,
};
use alert::Alert;
use std::{borrow::Cow, ffi::OsStr, fs, path::Path, time::Duration};
use terminal::{
    event::{Event, Key},
    Terminal,
};

#[must_use]
pub enum State {
    /// Execution is to be continued normally.
    Continue,
    /// The grid has been solved.
    Solved(Duration),
    /// Display an alert.
    ///
    /// NOTE: alerts do not end in a period.
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

    'main: loop {
        if let Some(event) = terminal.read_event() {
            // The order of statements matters

            // NOTE: fix the clear delay in another commit
            if let Some(ref mut alert_to_clear) = alert {
                if alert_to_clear.clear_delay != 0 {
                    if alert_to_clear.clear_delay == 0 {
                        alert_to_clear.clear(terminal, builder);
                        alert = None;
                    } else {
                        alert_to_clear.clear_delay -= 1;
                    }
                }
            }

            let state = input::handle(
                terminal,
                event,
                builder,
                &mut editor,
                &alert,
                &mut cell_placement,
            );

            #[cfg(debug_assertions)]
            {
                crate::grid::debug::display(terminal, builder);
            }

            terminal.flush();

            match state {
                State::Continue => continue,
                State::Alert(alert_message) => {
                    // Draw a new alert. Alerts are cleared after some time.

                    if let Some(mut previous_alert) = alert {
                        previous_alert.clear(terminal, builder);
                    }
                    terminal.reset_colors();

                    let new_alert = Alert::new(alert_message);

                    new_alert.draw(terminal, builder);
                    terminal.flush();

                    alert = Some(new_alert);
                }
                State::ClearAlert => {
                    if let Some(mut alert_to_clear) = alert {
                        alert_to_clear.clear(terminal, builder);
                        alert = None;
                    }
                }
                State::LoadGrid => {
                    if let Some(ref mut alert_to_clear) = alert {
                        alert_to_clear.clear(terminal, builder);
                    }

                    let new_alert = Alert::new(
                        "Drop a `.yaya` grid file onto this window to load. Esc to abort".into(),
                    );

                    new_alert.draw(terminal, builder);
                    terminal.disable_mouse_capture();
                    terminal.flush();
                    alert = Some(new_alert);

                    let mut path = String::new();

                    fn grid_found(mut str: &str) -> bool {
                        // In some terminals the paths start and end with apostrophes.
                        // We simply ignore those.
                        str = str
                            .strip_prefix('\'')
                            .unwrap_or(str)
                            .strip_suffix('\'')
                            .unwrap_or(str);

                        let path = Path::new(str);

                        path.exists() && path.extension() == Some(OsStr::new("yaya"))
                    }

                    while !(grid_found(&path)) {
                        let input = terminal.read_event();

                        match input {
                            Some(Event::Key(Key::Char(char))) => {
                                path.push(char);
                            }
                            Some(Event::Key(Key::Esc)) => {
                                if let Some(ref mut alert_to_clear) = alert {
                                    alert_to_clear.clear(terminal, builder);
                                }
                                let new_alert = Alert::new("Aborting".into());
                                new_alert.draw(terminal, builder);
                                alert = Some(new_alert);
                                terminal.enable_mouse_capture();
                                terminal.flush();

                                continue 'main;
                            }
                            _ => {
                                if let Some(ref mut alert_to_clear) = alert {
                                    alert_to_clear.clear(terminal, builder);
                                }
                                let new_alert = Alert::new("Invalid input. Aborting".into());
                                new_alert.draw(terminal, builder);
                                alert = Some(new_alert);
                                terminal.enable_mouse_capture();
                                terminal.flush();

                                continue 'main;
                            }
                        }
                    }

                    terminal.enable_mouse_capture(); // The accompanying flush to this follows later

                    fn load(path: &str) -> Option<Grid> {
                        let content = fs::read_to_string(&path).ok()?;

                        let grid = editor::load_grid(&content).ok()?;

                        Some(grid)
                    }

                    if let Some(grid) = load(&path) {
                        // Currently the new game simply runs inside of this existing game and creates the new game creates an entirely new state.
                        // Someday we would hit a stack overflow if the user keeps loading new levels within the same session.

                        terminal.clear();
                        start_game(terminal, grid);

                        break State::Exit;
                    } else {
                        if let Some(ref mut alert_to_clear) = alert {
                            alert_to_clear.clear(terminal, builder);
                        }
                        let new_alert = Alert::new("Loading failed".into());
                        new_alert.draw(terminal, builder);
                        alert = Some(new_alert);
                        terminal.flush();
                    }
                }
                State::Solved(_) | State::Exit => break state,
            }
        }
    }
}
