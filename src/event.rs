mod alert;
pub mod input;

use crate::{
    editor::Editor,
    grid::{builder::Builder, Cell, CellPlacement, Grid},
};
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
    /// The next cell placement will flood-fill.
    Fill,
    /// Exit the program.
    Exit,
}

pub fn r#loop(terminal: &mut Terminal, builder: &mut Builder) -> State {
    let mut editor = Editor::default();

    // TODO: make into one struct
    let mut alert: Option<Cow<'static, str>> = None;
    let mut alert_clear_delay = 0_usize;

    let mut cell_placement = CellPlacement::default();

    loop {
        if let Some(event) = terminal.read_event() {
            // The order of statements matters

            if alert_clear_delay != 0 {
                alert_clear_delay -= 1;
                if alert_clear_delay == 0 {
                    if let Some(alert_to_clear) = alert {
                        alert::clear(terminal, builder, alert_to_clear.len());
                        alert = None;
                    }
                }
            }

            let state = input::handle(
                terminal,
                event,
                builder,
                &mut editor,
                alert.as_ref(),
                &mut cell_placement,
            );

            #[cfg(debug_assertions)]
            {
                crate::grid::debug::display(terminal, builder);
            }

            terminal.flush();

            match state {
                State::Continue => continue,
                State::Alert(new_alert) => {
                    // Draw a new alert. Alerts are cleared after some time.

                    if let Some(previous_alert) = alert {
                        alert::clear(terminal, builder, previous_alert.len());
                    }
                    terminal.reset_colors();
                    alert::draw(terminal, builder, &new_alert);
                    alert = Some(new_alert);
                    alert_clear_delay = 75;
                    terminal.flush();
                }
                State::ClearAlert => {
                    if let Some(alert_to_clear) = alert {
                        alert::clear(terminal, builder, alert_to_clear.len());
                        alert = None;
                    }
                }
                State::Fill => {
                    let new_alert = "Set place to fill";
                    terminal.reset_colors();
                    alert::draw(terminal, builder, new_alert);
                    alert = Some(new_alert.into());
                    alert_clear_delay = 0;
                    cell_placement.fill = true;
                }
                State::Solved(_) | State::Exit => break state,
            }
        }
    }
}
