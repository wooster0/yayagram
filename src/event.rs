mod alert;
pub mod input;

use crate::{
    args::{valid_extension, FILE_EXTENSION},
    editor::{self, Editor},
    grid::{builder::Builder, CellPlacement, Grid},
    start_game,
};
use alert::Alert;
use std::{borrow::Cow, fs, time::Duration};
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
    /// NOTE: alert messages do not end in a period.
    Alert(Cow<'static, str>),
    /// Clear the alert if present.
    ClearAlert,
    /// Halt the game to load a new grid.
    LoadGrid,
    /// Exit the program.
    Exit,
}

fn draw_alert(
    terminal: &mut Terminal,
    builder: &mut Builder,
    alert: &mut Option<Alert>,
    message: Cow<'static, str>,
) {
    if let Some(ref mut current_alert) = alert {
        terminal.reset_colors();
        current_alert.clear(terminal, builder);

        current_alert.message = message;
        current_alert.reset_clear_delay();

        current_alert.draw(terminal, builder);
    } else {
        let new_alert = Alert::new(message);
        new_alert.draw(terminal, builder);
        *alert = Some(new_alert);
    }
}

pub fn r#loop(terminal: &mut Terminal, builder: &mut Builder) -> State {
    let mut editor = Editor::default();

    let mut alert: Option<Alert> = None;

    let mut cell_placement = CellPlacement::default();

    'main: loop {
        if let Some(event) = terminal.read_event() {
            // The order of statements matters

            if let Some(ref mut alert_to_clear) = alert {
                if alert_to_clear.clear_delay == 0 {
                    alert_to_clear.clear(terminal, builder);
                    alert = None;
                } else {
                    alert_to_clear.clear_delay -= 1;
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

            match state {
                State::Continue => {
                    terminal.flush();
                    continue;
                }
                State::Alert(alert_message) => {
                    // Draw a new alert. Alerts are cleared after some time.

                    draw_alert(terminal, builder, &mut alert, alert_message);
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
                    let message = format!(
                        "Drag & drop a `.{}` grid file onto this window to load. Esc to abort",
                        FILE_EXTENSION,
                    )
                    .into();

                    draw_alert(terminal, builder, &mut alert, message);

                    terminal.disable_mouse_capture();

                    terminal.flush();

                    let mut path = String::new();

                    while !valid_extension(&path) {
                        let input = terminal.read_event();

                        match input {
                            Some(Event::Key(Key::Char(char))) => {
                                if char == '\'' && path.is_empty() {
                                    // In some terminals the paths start and end with apostrophes.
                                    // We simply ignore the first one.
                                    // `valid_extension` will then recognize the path before we push the last apostrophe.
                                } else {
                                    path.push(char);
                                }
                            }
                            Some(Event::Key(Key::Esc)) => {
                                draw_alert(terminal, builder, &mut alert, "Aborting".into());

                                terminal.enable_mouse_capture();
                                terminal.flush();

                                continue 'main;
                            }
                            _ => {
                                draw_alert(
                                    terminal,
                                    builder,
                                    &mut alert,
                                    "Invalid input. Aborting".into(),
                                );

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
                        // Currently the new game simply runs inside of this existing game and the new game creates an entirely new state.
                        // At some point we would hit a stack overflow if the user keeps loading new levels within the same session.

                        terminal.clear();
                        start_game(terminal, grid);

                        break State::Exit;
                    } else {
                        draw_alert(terminal, builder, &mut alert, "Loading failed".into());
                        terminal.flush();
                    }
                }
                State::Solved(_) | State::Exit => break state,
            }
        }
    }
}
