use super::State;
use crate::{
    editor::Editor,
    grid::{
        builder::{Builder, Cursor},
        Cell, Grid,
    },
    undo_redo_buffer, util, TEXT_LINE_COUNT,
};
use std::time::Instant;
use terminal::{
    event::{Event, KeyEvent, MouseButton, MouseEvent, MouseEventKind},
    util::Point,
    Terminal,
};

/// Handles the event and returns a `bool` determing whether execution should be aborted.
fn handle_mouse(
    terminal: &mut Terminal,
    event: MouseEvent,
    builder: &mut Builder,
    plot_mode: &mut Option<Cell>,
    editor_toggled: bool,
    starting_time: &mut Option<Instant>,
    hovered_cell_point: &mut Option<Point>,
) -> State {
    match event {
        MouseEvent {
            kind: MouseEventKind::Drag(mouse_button),
            point,
        }
        | MouseEvent {
            kind: MouseEventKind::Press(mouse_button),
            point,
        } => {
            if builder.contains(point) {
                *hovered_cell_point = Some(point);
                let some_hovered_cell_point = point;

                let starting_time = starting_time.get_or_insert(Instant::now());

                let cell_point = super::get_cell_point_from_cursor_point(point, builder);
                let cell = builder.grid.get_mut_cell(cell_point);

                if let Some(plot_mode) = *plot_mode {
                    if *cell == plot_mode {
                        let cell = *cell;

                        // No grid mutation happened
                        let _all_clues_solved = builder.draw(terminal);

                        // Overdraw this hovered cell with a dark color
                        super::draw_dark_cell_color(
                            terminal,
                            &builder,
                            point,
                            cell,
                            some_hovered_cell_point,
                        );

                        return State::Continue;
                    }
                    *cell = plot_mode;
                } else {
                    let mut new_plot_mode = match mouse_button {
                        MouseButton::Left => Cell::Filled,
                        MouseButton::Middle => Cell::Maybed,
                        MouseButton::Right => Cell::Crossed,
                    };
                    if *cell == new_plot_mode {
                        new_plot_mode = Cell::default();
                    }
                    *plot_mode = Some(new_plot_mode);
                    *cell = new_plot_mode;
                }
                let cell = *cell;

                builder
                    .grid
                    .undo_redo_buffer
                    .push(undo_redo_buffer::Operation::SetCell {
                        point: cell_point,
                        cell,
                    });

                if editor_toggled {
                    super::rebuild_clues(terminal, builder, cell_point);

                    // The solved screen shouldn't be triggered within the editor
                    let _all_clues_solved = builder.draw(terminal);
                } else {
                    let all_clues_solved = builder.draw(terminal);

                    if all_clues_solved {
                        return State::Solved(starting_time.elapsed());
                    }
                }

                // Overdraw this hovered cell with a dark color
                super::draw_dark_cell_color(
                    terminal,
                    &builder,
                    point,
                    cell,
                    some_hovered_cell_point,
                );
            } else {
                // `plot_mode` won't be reset
            }
        }
        MouseEvent {
            kind: MouseEventKind::Move,
            point,
        } => {
            // No grid mutation happened
            let _all_clues_solved = builder.draw(terminal);

            if builder.contains(point) {
                *hovered_cell_point = Some(point);
                let some_hovered_cell_point = point;

                let cell_point = super::get_cell_point_from_cursor_point(point, builder);
                let cell = builder.grid.get_cell(cell_point);
                super::draw_dark_cell_color(
                    terminal,
                    &builder,
                    point,
                    cell,
                    some_hovered_cell_point,
                );
            }
        }
        _ => {
            *plot_mode = None;
        }
    }

    State::Continue
}

/// Handles the event and returns a `State`.
pub fn handle(
    // TODO: this function has too many arguments and should be refactored
    terminal: &mut Terminal,
    event: Event,
    builder: &mut Builder,
    plot_mode: &mut Option<Cell>,
    editor: &mut Editor,
    last_notification: Option<&'static str>,
    starting_time: &mut Option<Instant>,
    hovered_cell_point: &mut Option<Point>,
    measurement_point: &mut Option<Point>,
) -> State {
    match event {
        Event::Mouse(mouse_event) => handle_mouse(
            terminal,
            mouse_event,
            builder,
            plot_mode,
            editor.toggled,
            starting_time,
            hovered_cell_point,
        ),
        Event::Key(key_event) => handle_key(
            terminal,
            key_event,
            builder,
            editor,
            *hovered_cell_point,
            measurement_point,
        ),
        Event::Resize => handle_window_resize(terminal, builder, last_notification),
    }
}

fn handle_window_resize(
    terminal: &mut Terminal,
    builder: &mut Builder,
    last_notification: Option<&'static str>,
) -> State {
    let state = await_fitting_window_size(terminal, &builder.grid);

    builder.cursor = Cursor::centered(terminal, &builder.grid);

    terminal.clear();

    // No grid mutation happened
    let _all_clues_solved = builder.draw(terminal);

    crate::draw_help(terminal, &builder);
    if let Some(notification) = last_notification {
        super::notification::draw(terminal, builder, &notification);
    }

    state
}

/// This handles all key input for actions like undo, redo, reset and so on.
fn handle_key(
    terminal: &mut Terminal,
    key_event: KeyEvent,
    builder: &mut Builder,
    editor: &mut Editor,
    hovered_cell_point: Option<Point>,
    measurement_point: &mut Option<Point>,
) -> State {
    match key_event {
        KeyEvent::Char('a', None) | KeyEvent::Char('A', None) => {
            if builder.grid.undo_last_cell() {
                // It would've already been solved before
                let _all_clues_solved = builder.draw(terminal);
            }

            State::Continue
        }
        KeyEvent::Char('d', None) | KeyEvent::Char('D', None) => {
            if builder.grid.redo_last_cell() {
                // It would've already been solved before
                let _all_clues_solved = builder.draw(terminal);
            }

            State::Continue
        }
        KeyEvent::Char('c', None) | KeyEvent::Char('C', None) => {
            builder.grid.clear();
            builder
                .grid
                .undo_redo_buffer
                .push(undo_redo_buffer::Operation::Clear);

            // It would've already been solved from the start
            let _all_clues_solved = builder.draw(terminal);

            State::Continue
        }
        KeyEvent::Char('x', None) | KeyEvent::Char('X', None) => {
            if let Some(hovered_cell_point) = hovered_cell_point {
                if let Some(some_measurement_point) = *measurement_point {
                    // The points we have are screen points so now we convert them to values that we can use
                    // to index the grid.
                    let start_point =
                        super::get_cell_point_from_cursor_point(some_measurement_point, builder);
                    let end_point =
                        super::get_cell_point_from_cursor_point(hovered_cell_point, builder);

                    let line_points: Vec<Point> =
                        util::get_line_points(start_point, end_point).collect();

                    super::set_measured_cells(&mut builder.grid, &line_points);

                    builder
                        .grid
                        .undo_redo_buffer
                        .push(undo_redo_buffer::Operation::Measure(line_points));

                    // Measured cells cannot solve the grid
                    let _all_clues_solved = builder.draw(terminal);

                    // The cell might not be a measured cell because they are only drawn on
                    // measured and empty cells
                    if let Cell::Measured(_) = builder.grid.get_cell(end_point) {
                        // Overdraw the hovered cell with a dark color
                        super::draw_dark_cell_color(
                            terminal,
                            &builder,
                            hovered_cell_point,
                            Cell::Measured(None),
                            hovered_cell_point,
                        );
                    }

                    *measurement_point = None;

                    State::ClearAlert
                } else {
                    *measurement_point = Some(hovered_cell_point);

                    State::Alert("Set second measurement point")
                }
            } else {
                State::Continue
            }
        }
        KeyEvent::Tab => {
            editor.toggle();

            if editor.toggled {
                // TODO: maybe this info should be shown all the time (make it part of window title?)
                State::Alert("Editor enabled")
            } else {
                State::Alert("Editor disabled")
            }
        }
        KeyEvent::Char('s', None) | KeyEvent::Char('S', None) if editor.toggled => {
            if let Err(err) = editor.save_grid(&builder) {
                State::Alert(err)
            } else {
                super::set_title(
                    terminal,
                    &format!("yayagram - Grid saved as {}", editor.filename),
                );
                State::Continue
            }
        }
        KeyEvent::Esc => State::Exit,
        _ => State::Continue,
    }
}

pub fn await_fitting_window_size(terminal: &mut Terminal, grid: &Grid) -> State {
    fn terminal_height_is_within_grid_height(grid: &Grid, terminal: &Terminal) -> bool {
        terminal.size.height >= grid.size.height + grid.max_clues_size.height + TEXT_LINE_COUNT * 2
    }

    fn terminal_width_is_within_grid_width(grid: &Grid, terminal: &Terminal) -> bool {
        terminal.size.width >= grid.size.width + grid.max_clues_size.width
    }

    let mut state = State::Continue;

    match (
        terminal_width_is_within_grid_width(&grid, terminal),
        terminal_height_is_within_grid_height(&grid, terminal),
    ) {
        (true, true) => state,
        (within_width, within_height) => {
            terminal.set_cursor(Point::default());
            if !within_width {
                terminal.write("Please increase window width or decrease text size (Ctrl and -)");
            } else if !within_height {
                terminal.write("Please increase window height or decrease text size (Ctrl and -)");
            } else {
                unreachable!();
            }
            terminal.flush();
            loop {
                match (
                    terminal_width_is_within_grid_width(&grid, terminal),
                    terminal_height_is_within_grid_height(&grid, terminal),
                ) {
                    (true, true) => break state,
                    _ => {
                        state = await_window_resize(terminal);
                        if let State::Exit = state {
                            return state;
                        }
                    }
                }
            }
        }
    }
}

pub fn await_key(terminal: &mut Terminal) {
    loop {
        let event = terminal.read_event();
        if let Some(Event::Key(_)) = event {
            break;
        }
    }
}

fn await_window_resize(terminal: &mut Terminal) -> State {
    loop {
        let event = terminal.read_event();
        match event {
            Some(Event::Key(KeyEvent::Esc)) => break State::Exit,
            Some(Event::Key(_)) => break State::Continue,
            Some(Event::Resize) => break State::Continue,
            _ => {}
        }
    }
}
