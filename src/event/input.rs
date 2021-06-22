use super::State;
use crate::{
    editor::Editor,
    grid::{self, builder::Builder, Cell, CellPlacement, Grid},
    undo_redo_buffer, util,
};
use std::borrow::Cow;
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
    editor_toggled: bool,
    cell_placement: &mut CellPlacement,
) -> State {
    match event {
        MouseEvent {
            kind: MouseEventKind::Drag(mouse_button) | MouseEventKind::Press(mouse_button),
            point: selected_cell_point,
        } => {
            if builder.contains(selected_cell_point) {
                let cell_to_place = match mouse_button {
                    MouseButton::Left => Cell::Filled,
                    MouseButton::Middle => Cell::Maybed,
                    MouseButton::Right => Cell::Crossed,
                };

                cell_placement.selected_cell_point = Some(selected_cell_point);

                cell_placement.place(
                    terminal,
                    builder,
                    selected_cell_point,
                    cell_to_place,
                    editor_toggled,
                )
            } else {
                State::Continue
            }
        }
        MouseEvent {
            kind: MouseEventKind::Move,
            point,
        } => {
            builder.draw_grid(terminal);

            if builder.contains(point) {
                cell_placement.selected_cell_point = Some(point);
                let some_selected_cell_point = point;

                // We know that this point is hovered
                grid::draw_highlighted_cells(terminal, &builder, some_selected_cell_point);
            }
            State::Continue
        }
        _ => {
            cell_placement.cell = None;
            State::Continue
        }
    }
}

/// Handles the event and returns a `State`.
pub fn handle(
    terminal: &mut Terminal,
    event: Event,
    builder: &mut Builder,
    editor: &mut Editor,
    last_alert: Option<&Cow<'static, str>>,
    cell_placement: &mut CellPlacement,
) -> State {
    match event {
        Event::Mouse(mouse_event) => handle_mouse(
            terminal,
            mouse_event,
            builder,
            editor.toggled,
            cell_placement,
        ),
        Event::Key(key_event) => handle_key(terminal, key_event, builder, editor, cell_placement),
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

    builder.point = grid::builder::centered_point(terminal, &builder.grid);

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
    cell_placement: &mut CellPlacement,
) -> State {
    match key_event {
        KeyEvent::Char('a' | 'A', None) => {
            if builder.grid.undo_last_cell() {
                // An undo won't cause the grid to be solved at this point because otherwise it would've already been solved before when that operation was done.
                #[allow(unused_must_use)]
                {
                    builder.draw_all(terminal);
                }
            }

            State::Continue
        }
        KeyEvent::Char('d' | 'D', None) => {
            if builder.grid.redo_last_cell() {
                // A redo won't cause the grid to be solved at this point because otherwise it would've already been solved before when that operation was done.
                #[allow(unused_must_use)]
                {
                    builder.draw_all(terminal);
                }
            }

            State::Continue
        }
        KeyEvent::Char('c' | 'C', None) => {
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
        KeyEvent::Char('f' | 'F', None) => State::Fill,
        KeyEvent::Char('x' | 'X', None) => {
            // TODO: maybe move this and other stuff to cellplacement too
            if let Some(selected_cell_point) = cell_placement.selected_cell_point {
                if let Some(measurement_point) = cell_placement.measurement_point {
                    // The points we have are screen points so now we convert them to values that we can use
                    // to index the grid.
                    let start_point =
                        grid::get_cell_point_from_cursor_point(measurement_point, builder);
                    let end_point =
                        grid::get_cell_point_from_cursor_point(selected_cell_point, builder);

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
                    grid::draw_highlighted_cells(terminal, &builder, selected_cell_point);

                    cell_placement.measurement_point = None;

                    State::ClearAlert
                } else {
                    cell_placement.measurement_point = Some(selected_cell_point);

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
        KeyEvent::Char('s' | 'S', None) if editor.toggled => {
            if let Err(err) = editor.save_grid(&builder) {
                State::Alert(err.into())
            } else {
                State::Alert(format!("Grid saved as {}", editor.filename).into())
            }
        }
        KeyEvent::Char(char, None) => {
            if let Some(selected_cell_point) = cell_placement.selected_cell_point {
                let cell_to_place = match char {
                    'q' | 'Q' => Cell::Filled,
                    'w' | 'W' => Cell::Maybed,
                    'e' | 'E' => Cell::Crossed,
                    _ => return State::Continue,
                };

                let state = cell_placement.place(
                    terminal,
                    builder,
                    selected_cell_point,
                    cell_to_place,
                    editor.toggled,
                );

                cell_placement.cell = None;

                state
            } else {
                State::Continue
            }
        }
        KeyEvent::Esc => State::Exit,
        KeyEvent::Up | KeyEvent::Down | KeyEvent::Left | KeyEvent::Right => {
            let selected_cell_point = if let Some(selected_cell_point) =
                &mut cell_placement.selected_cell_point
            {
                match key_event {
                    KeyEvent::Up => {
                        selected_cell_point.y -= 1;

                        if !(builder.point.y..builder.point.y + builder.grid.size.height)
                            .contains(&selected_cell_point.y)
                        {
                            selected_cell_point.y = builder.point.y + builder.grid.size.height - 1;
                        }
                    }
                    KeyEvent::Down => {
                        selected_cell_point.y += 1;

                        if !(builder.point.y..builder.point.y + builder.grid.size.height)
                            .contains(&selected_cell_point.y)
                        {
                            selected_cell_point.y = builder.point.y;
                        }
                    }
                    KeyEvent::Left => {
                        selected_cell_point.x -= 2;

                        if !(builder.point.x..builder.point.x + builder.grid.size.width * 2)
                            .contains(&selected_cell_point.x)
                        {
                            selected_cell_point.x =
                                builder.point.x + builder.grid.size.width * 2 - 2;
                        }
                    }
                    KeyEvent::Right => {
                        selected_cell_point.x += 2;

                        if !(builder.point.x..builder.point.x + builder.grid.size.width * 2)
                            .contains(&selected_cell_point.x)
                        {
                            selected_cell_point.x = builder.point.x
                        }
                    }
                    _ => unreachable!(),
                }

                *selected_cell_point
            } else {
                let grid_center = builder.get_center();
                cell_placement.selected_cell_point = Some(grid_center);

                grid_center
            };

            builder.draw_grid(terminal);

            // We know that this point is hovered
            grid::draw_highlighted_cells(terminal, &builder, selected_cell_point);

            State::Continue
        }
        _ => State::Continue,
    }
}

// TODO: move all the window stuff into input/window.rs
// and maybe mouse and key stuff separately too?
pub fn await_fitting_window_size(terminal: &mut Terminal, grid: &Grid) -> State {
    const fn terminal_width_is_within_grid_width(grid: &Grid, terminal: &Terminal) -> bool {
        terminal.size.width >= grid.size.width * 2 + grid.max_clues_size.width
    }

    fn terminal_height_is_within_grid_height(grid: &Grid, terminal: &Terminal) -> bool {
        terminal.size.height > crate::total_height(grid)
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
            let message = format!(
                "Please increase window {} or decrease text size (Ctrl and -)",
                length
            );
            terminal.write(&message);
            terminal.flush();

            let state = loop {
                match (
                    terminal_width_is_within_grid_width(&grid, terminal),
                    terminal_height_is_within_grid_height(&grid, terminal),
                ) {
                    (true, true) => break state,
                    _ => {
                        state = await_window_resize(terminal);
                        if let State::Exit = state {
                            break state;
                        }
                    }
                }
            };

            terminal.set_cursor(Point::default());
            for _ in 0..message.len() {
                terminal.write(" ");
            }

            state
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
