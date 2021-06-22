mod alert;
pub mod input;

use crate::{
    editor::Editor,
    grid::{builder::Builder, Cell, CellPlacement, Grid},
};
use alert::Alert;
use std::{borrow::Cow, time::Duration};
use terminal::{util::Point, Terminal};

pub fn set_measured_cells(grid: &mut Grid, line_points: &[Point]) {
    for (index, point) in line_points.iter().enumerate() {
        let cell = grid.get_mut_cell(*point);

        if let Cell::Empty | Cell::Measured(_) = cell {
            *cell = Cell::Measured(Some(index + 1));
        }
    }
}

#[must_use]
pub enum State {
    /// Execution is to be continued normally.
    Continue,
    /// The grid has been solved.
    Solved(Duration),
    /// Display an alert.
    Alert(Cow<'static, str>),
    /// Clear the alert if present.
    ClearAlert,
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
                State::Solved(_) | State::Exit => break state,
            }
        }
    }
}
