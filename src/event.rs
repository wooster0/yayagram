mod alert;
pub mod input;

use crate::{
    editor::Editor,
    grid::{builder::Builder, Cell, Grid},
};
use std::{
    borrow::Cow,
    time::{Duration, Instant},
};
use terminal::{util::Point, Terminal};

fn draw_highlighted_cells(terminal: &mut Terminal, builder: &Builder, hovered_cell_point: Point) {
    fn highlight_cell(terminal: &mut Terminal, mut cursor_point: Point, builder: &Builder) {
        if (cursor_point.x - builder.point.x) % 2 != 0 {
            cursor_point.x -= 1;
        }
        terminal.set_cursor(cursor_point);
        let cell_point = get_cell_point_from_cursor_point(cursor_point, builder);
        let cell = builder.grid.get_cell(cell_point);
        cell.draw(terminal, cell_point, true);
    }

    // From the left of the grid to the pointer
    for x in builder.point.x..=hovered_cell_point.x - 2 {
        let point = Point {
            x,
            ..hovered_cell_point
        };

        highlight_cell(terminal, point, builder);
    }
    // From the pointer to the right of the grid
    for x in hovered_cell_point.x + 2..builder.point.x + builder.grid.size.width * 2 {
        let point = Point {
            x,
            ..hovered_cell_point
        };

        highlight_cell(terminal, point, builder);
    }
    // From the top of the grid to the pointer
    for y in builder.point.y..hovered_cell_point.y {
        let point = Point {
            y,
            ..hovered_cell_point
        };

        highlight_cell(terminal, point, builder);
    }
    // From the pointer to the bottom of the grid
    for y in hovered_cell_point.y + 1..builder.point.y + builder.grid.size.height {
        let point = Point {
            y,
            ..hovered_cell_point
        };

        highlight_cell(terminal, point, builder);
    }

    terminal.reset_colors();
}

const fn get_cell_point_from_cursor_point(cursor_point: Point, builder: &Builder) -> Point {
    Point {
        x: (cursor_point.x - builder.point.x) / 2,
        y: cursor_point.y - builder.point.y,
    }
}

/// Reconstructs the clues associated with the given `cell_point`.
fn rebuild_clues(terminal: &mut Terminal, builder: &mut Builder, cell_point: Point) {
    builder.clear_clues(terminal);
    builder.grid.horizontal_clues_solutions[cell_point.y as usize] =
        builder.grid.get_horizontal_clues(cell_point.y).collect();
    builder.grid.vertical_clues_solutions[cell_point.x as usize] =
        builder.grid.get_vertical_clues(cell_point.x).collect();
}

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
    let mut plot_mode = None;
    let mut editor = Editor::default();

    let mut alert: Option<Cow<'static, str>> = None;
    let mut alert_clear_delay = 0_usize;

    let mut starting_time: Option<Instant> = None;

    let mut hovered_cell_point: Option<Point> = None;
    let mut measurement_point: Option<Point> = None;

    let mut fill = false;

    // TODO: refactor above variables into one big struct and/or multiple structs

    loop {
        //terminal.deinitialize();
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
                &mut plot_mode,
                &mut editor,
                alert.as_ref(),
                &mut starting_time,
                &mut hovered_cell_point,
                &mut measurement_point,
                &mut fill,
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
                    alert::draw(terminal, builder, new_alert);
                    alert = Some(new_alert.into());
                    alert_clear_delay = 0;
                    fill = true;
                }
                State::Solved(_) | State::Exit => break state,
            }
        }
    }
}
