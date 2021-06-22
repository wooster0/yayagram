use super::State;
use crate::{
    editor::Editor,
    grid::CellPlacement,
    grid::{self, builder::Builder, Cell},
    undo_redo_buffer, util,
};
use terminal::{
    event::{Event, KeyEvent},
    util::Point,
    Terminal,
};

/// This handles all key input for actions like undo, redo, reset and so on.
pub fn handle_event(
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
        KeyEvent::Char('f' | 'F', None) => {
            cell_placement.fill = true;
            State::Alert("Set place to fill".into())
        }
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

                    grid::set_measured_cells(&mut builder.grid, &line_points);

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
        KeyEvent::Esc => State::Exit,
        _ => State::Continue,
    }
}

pub fn r#await(terminal: &mut Terminal) {
    loop {
        let event = terminal.read_event();
        if let Some(Event::Key(_)) = event {
            break;
        }
    }
}
