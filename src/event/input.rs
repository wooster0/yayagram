use super::State;
use crate::{
    editor::Editor,
    grid::{builder::Builder, Cell, Grid},
    undo_redo_buffer, util,
};
use std::{borrow::Cow, time::Instant};
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
    fill: &mut bool,
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

                *cell = if let Some(plot_mode) = *plot_mode {
                    if *cell == plot_mode {
                        builder.draw_grid(terminal);

                        // We know that this point is hovered
                        super::draw_highlighted_cells(terminal, &builder, some_hovered_cell_point);

                        return State::Continue;
                    }

                    plot_mode
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

                    if *fill {
                        let cell = *cell;
                        crate::grid::tools::fill::fill(
                            &mut builder.grid,
                            cell_point,
                            cell,
                            new_plot_mode,
                        );

                        builder
                            .grid
                            .undo_redo_buffer
                            .push(undo_redo_buffer::Operation::Fill {
                                point: cell_point,
                                first_cell: cell,
                                fill_cell: new_plot_mode,
                            });

                        *fill = false;

                        let all_clues_solved = builder.draw_all(terminal);

                        if all_clues_solved {
                            return State::Solved(starting_time.elapsed());
                        } else {
                            return State::ClearAlert;
                        }
                    }

                    new_plot_mode
                };
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

                    // The grid shouldn't be solved while editing it
                    #[allow(unused_must_use)]
                    {
                        builder.draw_all(terminal);
                    }
                } else {
                    let all_clues_solved = builder.draw_all(terminal);

                    if all_clues_solved {
                        return State::Solved(starting_time.elapsed());
                    }
                }

                // We know that this point is hovered
                super::draw_highlighted_cells(terminal, &builder, some_hovered_cell_point);
            } else {
                // `plot_mode` won't be reset
            }
        }
        MouseEvent {
            kind: MouseEventKind::Move,
            point,
        } => {
            builder.draw_grid(terminal);

            if builder.contains(point) {
                *hovered_cell_point = Some(point);
                let some_hovered_cell_point = point;

                // We know that this point is hovered
                super::draw_highlighted_cells(terminal, &builder, some_hovered_cell_point);
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
    last_alert: Option<&Cow<'static, str>>,
    starting_time: &mut Option<Instant>,
    hovered_cell_point: &mut Option<Point>,
    measurement_point: &mut Option<Point>,
    fill: &mut bool,
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
            fill,
        ),
        Event::Key(key_event) => handle_key(
            terminal,
            key_event,
            builder,
            editor,
            *hovered_cell_point,
            measurement_point,
        ),
        Event::Resize => handle_window_resize(terminal, builder, last_alert),
    }
}

fn handle_window_resize(
    terminal: &mut Terminal,
    builder: &mut Builder,
    last_alert: Option<&Cow<'static, str>>,
) -> State {
    terminal.clear();

    let state = await_fitting_window_size(terminal, &builder.grid);

    builder.point = crate::grid::builder::centered_point(terminal, &builder.grid);

    // No grid mutation happened
    #[allow(unused_must_use)]
    {
        builder.draw_all(terminal);
    }

    crate::draw_basic_controls_help(terminal, &builder);
    if let Some(alert) = last_alert {
        super::alert::draw(terminal, builder, &alert);
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
                // An undo won't cause the grid to be solved at this point because otherwise it would've already been solved before when that operation was done.
                #[allow(unused_must_use)]
                {
                    builder.draw_all(terminal);
                }
            }

            State::Continue
        }
        KeyEvent::Char('d', None) | KeyEvent::Char('D', None) => {
            if builder.grid.redo_last_cell() {
                // A redo won't cause the grid to be solved at this point because otherwise it would've already been solved before when that operation was done.
                #[allow(unused_must_use)]
                {
                    builder.draw_all(terminal);
                }
            }

            State::Continue
        }
        KeyEvent::Char('c', None) | KeyEvent::Char('C', None) => {
            builder.grid.clear();
            builder
                .grid
                .undo_redo_buffer
                .push(undo_redo_buffer::Operation::Clear);

            // A clear won't cause the grid to be solved at this point because otherwise it would've already been solved initially when the grid was empty.
            #[allow(unused_must_use)]
            {
                builder.draw_all(terminal);
            }

            State::Continue
        }
        KeyEvent::Char('f', None) | KeyEvent::Char('F', None) => State::Fill,
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

                    builder.draw_picture(terminal);
                    builder.draw_grid(terminal);

                    // We know that this point is hovered
                    super::draw_highlighted_cells(terminal, &builder, hovered_cell_point);

                    *measurement_point = None;

                    State::ClearAlert
                } else {
                    *measurement_point = Some(hovered_cell_point);

                    State::Alert("Set second measurement point".into())
                }
            } else {
                State::Continue
            }
        }
        KeyEvent::Tab => {
            editor.toggle();

            if editor.toggled {
                terminal.set_title("yayagram Editor");
                State::Alert("Editor enabled".into())
            } else {
                terminal.set_title("yayagram");
                State::Alert("Editor disabled".into())
            }
        }
        KeyEvent::Char('s', None) | KeyEvent::Char('S', None) if editor.toggled => {
            if let Err(err) = editor.save_grid(&builder) {
                State::Alert(err.into())
            } else {
                State::Alert(format!("Grid saved as {}", editor.filename).into())
            }
        }
        KeyEvent::Esc => State::Exit,
        _ => State::Continue,
    }
}

const PROGRESS_BAR_HEIGHT: u16 = 1;
const TOP_TEXT_HEIGHT: u16 = 2;
const BOTTOM_TEXT_HEIGHT: u16 = 2;

pub fn await_fitting_window_size(terminal: &mut Terminal, grid: &Grid) -> State {
    const fn terminal_height_is_within_grid_height(grid: &Grid, terminal: &Terminal) -> bool {
        terminal.size.height
            >= grid.size.height
                + crate::get_picture_height(grid)
                + PROGRESS_BAR_HEIGHT
                + TOP_TEXT_HEIGHT
                + BOTTOM_TEXT_HEIGHT
    }

    const fn terminal_width_is_within_grid_width(grid: &Grid, terminal: &Terminal) -> bool {
        terminal.size.width >= grid.size.width * 2 + grid.max_clues_size.width
    }

    let mut state = State::Continue;

    match (
        terminal_width_is_within_grid_width(&grid, terminal),
        terminal_height_is_within_grid_height(&grid, terminal),
    ) {
        (true, true) => state,
        (within_width, within_height) => {
            terminal.set_cursor(Point::default());
            let length = if !within_width {
                "width"
            } else if !within_height {
                "height"
            } else {
                unreachable!()
            };
            terminal.write(&format!(
                "Please increase window {} or decrease text size (Ctrl and -)",
                length
            ));
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

pub fn await_key(terminal: &mut Terminal) {
    loop {
        let event = terminal.read_event();
        if let Some(Event::Key(_)) = event {
            break;
        }
    }
}
