use super::State;
use crate::{
    editor::Editor,
    grid::CellPlacement,
    grid::{self, builder::Builder, Cell},
    undo_redo_buffer,
};
use terminal::{
    event::{Event, Key},
    Terminal,
};

/// This handles all key input.
pub fn handle_event(
    terminal: &mut Terminal,
    key_event: Key,
    builder: &mut Builder,
    editor: &mut Editor,
    cell_placement: &mut CellPlacement,
) -> State {
    match key_event {
        Key::Char('a' | 'A') => {
            if builder.grid.undo_last_cell() {
                // An undo won't cause the grid to be solved at this point because otherwise it would've already been solved before when that operation was done.
                #[allow(unused_must_use)]
                {
                    builder.draw_all(terminal);
                }
            }

            State::Continue
        }
        Key::Char('d' | 'D') => {
            if builder.grid.redo_last_cell() {
                // A redo won't cause the grid to be solved at this point because otherwise it would've already been solved before when that operation was done.
                #[allow(unused_must_use)]
                {
                    builder.draw_all(terminal);
                }
            }

            State::Continue
        }
        Key::Char('c' | 'C') => {
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
        Key::Char('f' | 'F') => {
            cell_placement.fill = true;
            State::Alert("Set place to fill".into())
        }
        Key::Char('x' | 'X') => cell_placement.place_measured_cells(terminal, builder),
        Key::Tab => {
            editor.toggle();

            if editor.toggled {
                terminal.set_title("yayagram Editor");
                State::Alert("Editor enabled".into())
            } else {
                terminal.set_title("yayagram");
                State::Alert("Editor disabled".into())
            }
        }
        Key::Char('s' | 'S') if editor.toggled => {
            if let Err(err) = editor.save_grid(&builder) {
                State::Alert(err.into())
            } else {
                State::Alert(format!("Grid saved as {}", editor.filename).into())
            }
        }
        Key::Char('l' | 'L') => State::LoadGrid,
        Key::Char(char) => {
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
        Key::Up | Key::Down | Key::Left | Key::Right => {
            let selected_cell_point = if let Some(selected_cell_point) =
                &mut cell_placement.selected_cell_point
            {
                match key_event {
                    Key::Up => {
                        selected_cell_point.y -= 1;

                        if !(builder.point.y..builder.point.y + builder.grid.size.height)
                            .contains(&selected_cell_point.y)
                        {
                            selected_cell_point.y = builder.point.y + builder.grid.size.height - 1;
                        }
                    }
                    Key::Down => {
                        selected_cell_point.y += 1;

                        if !(builder.point.y..builder.point.y + builder.grid.size.height)
                            .contains(&selected_cell_point.y)
                        {
                            selected_cell_point.y = builder.point.y;
                        }
                    }
                    Key::Left => {
                        selected_cell_point.x -= 2;

                        if !(builder.point.x..builder.point.x + builder.grid.size.width * 2)
                            .contains(&selected_cell_point.x)
                        {
                            selected_cell_point.x =
                                builder.point.x + builder.grid.size.width * 2 - 2;
                        }
                    }
                    Key::Right => {
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
        Key::Esc => State::Exit(cell_placement.starting_time),
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
