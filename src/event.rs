pub mod input;
mod notification;

use crate::{
    editor::Editor,
    grid::{
        builder::{Builder, Cursor},
        Cell, Grid,
    },
};
use std::{
    thread,
    time::{Duration, Instant},
};
use terminal::{
    util::{Color, Point},
    Terminal,
};

fn draw_dark_cell_color(
    terminal: &mut Terminal,
    builder: &Builder,
    cursor_point: Point,
    cell: Cell,
    hovered_cell_point: Point,
) {
    fn draw_color(terminal: &mut Terminal, mut cursor_point: Point, grid: &Grid, color: Color) {
        let center_x = Cursor::centered(terminal, grid).point.x;
        if (cursor_point.x - center_x) % 2 != 0 {
            cursor_point.x -= 1;
        }
        terminal.set_cursor(cursor_point);
        terminal.set_background_color(color);
        terminal.write("  ");
    }

    draw_color(
        terminal,
        cursor_point,
        &builder.grid,
        cell.get_darkest_color(),
    );

    // From the left of the grid to where the pointer is
    for x in builder.cursor.point.x..=hovered_cell_point.x - 2 {
        let point = Point {
            x,
            ..hovered_cell_point
        };

        let cell_point = get_cell_point_from_cursor_point(point, builder);
        let cell = builder.grid.get_cell(cell_point);

        draw_color(terminal, point, &builder.grid, cell.get_dark_color());
    }
    // From the pointer to the right of the grid
    for x in hovered_cell_point.x + 2..builder.cursor.point.x + builder.grid.size.width * 2 {
        let point = Point {
            x,
            ..hovered_cell_point
        };

        let cell_point = get_cell_point_from_cursor_point(point, builder);
        let cell = builder.grid.get_cell(cell_point);

        draw_color(terminal, point, &builder.grid, cell.get_dark_color());
    }
    // From the top of the grid to where the pointer is
    for y in builder.cursor.point.y..hovered_cell_point.y {
        let point = Point {
            y,
            ..hovered_cell_point
        };

        let cell_point = get_cell_point_from_cursor_point(point, builder);
        let cell = builder.grid.get_cell(cell_point);

        draw_color(terminal, point, &builder.grid, cell.get_dark_color());
    }

    for y in hovered_cell_point.y + 1..builder.cursor.point.y + builder.grid.size.height {
        let point = Point {
            y,
            ..hovered_cell_point
        };

        let cell_point = get_cell_point_from_cursor_point(point, builder);
        let cell = builder.grid.get_cell(cell_point);

        draw_color(terminal, point, &builder.grid, cell.get_dark_color());
    }

    terminal.reset_colors();
}

fn get_cell_point_from_cursor_point(cursor_point: Point, builder: &Builder) -> Point {
    Point {
        x: (cursor_point.x - builder.cursor.point.x) / 2,
        y: cursor_point.y - builder.cursor.point.y,
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

fn set_title(terminal: &mut Terminal, title: &str) {
    fn reset_title() {
        thread::spawn(|| {
            // NOTE: to be able to reuse the current `terminal`, it'd probably have to be in `RwLock` or `Mutex`?
            thread::sleep(Duration::from_secs(3));
            let mut terminal = Terminal::new().unwrap();
            terminal.set_title("yayagram");
        });
    }

    terminal.set_title(title);
    reset_title();
}

#[must_use]
pub enum State {
    /// Execution is to be continued normally.
    Continue,
    /// The grid has been solved.
    Solved(Duration),
    /// Display an alert.
    Alert(&'static str),
    /// Clear the alert if present.
    ClearAlert,
    /// Exit the program.
    Exit,
}

pub fn r#loop(terminal: &mut Terminal, builder: &mut Builder) -> State {
    let mut plot_mode = None;
    let mut editor = Editor::default();

    let mut notification: Option<&'static str> = None;
    let mut notification_clear_delay = 0_usize;

    let mut starting_time: Option<Instant> = None;

    let mut hovered_cell_point: Option<Point> = None;
    let mut measurement_point: Option<Point> = None;

    // TODO: refactor above variables into one big struct and/or multiple structs

    loop {
        //terminal.deinitialize();
        if let Some(event) = terminal.read_event() {
            // The order of statements matters
            if notification_clear_delay != 0 {
                notification_clear_delay -= 1;
                if notification_clear_delay == 0 {
                    if let Some(notification_to_clear) = notification {
                        notification::clear(terminal, builder, notification_to_clear.len());
                        notification = None;
                    }
                }
            }

            let state = input::handle(
                terminal,
                event,
                builder,
                &mut plot_mode,
                &mut editor,
                notification,
                &mut starting_time,
                &mut hovered_cell_point,
                &mut measurement_point,
            );

            #[cfg(debug_assertions)]
            {
                crate::grid::debug::display(terminal, builder);
            }

            terminal.flush();

            match state {
                State::Continue => continue,
                State::Alert(new_notification) => {
                    // Draw a new notification. Notifications are cleared after some time.

                    if let Some(previous_notification) = notification {
                        notification::clear(terminal, builder, previous_notification.len());
                    }
                    notification::draw(terminal, builder, new_notification);
                    notification = Some(new_notification);
                    notification_clear_delay = 75;
                    terminal.flush();
                }
                State::ClearAlert => {
                    if let Some(notification_to_clear) = notification {
                        notification::clear(terminal, builder, notification_to_clear.len());
                        notification = None;
                    }
                }
                State::Solved(_) | State::Exit => break state,
            }
        }
    }
}
