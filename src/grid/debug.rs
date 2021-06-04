use super::{builder::Builder, Cell, Grid};
use std::fmt;
use terminal::{util::Point, Terminal};

impl fmt::Debug for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Grid")
            .field("size", &self.size)
            .field(
                "cells (empty omitted)",
                &self
                    .cells
                    .iter()
                    .filter(|cell| *cell != &Cell::Empty)
                    .collect::<Vec<&Cell>>(),
            )
            .field(
                "horizontal_clues_solutions (empty omitted)",
                &self
                    .horizontal_clues_solutions
                    .iter()
                    .filter(|cell| !cell.is_empty())
                    .collect::<Vec<&Vec<u16>>>(),
            )
            .field(
                "vertical_clues_solutions (empty omitted)",
                &self
                    .vertical_clues_solutions
                    .iter()
                    .filter(|cell| !cell.is_empty())
                    .collect::<Vec<&Vec<u16>>>(),
            )
            .field("max_clues_size", &self.max_clues_size)
            .field("undo_redo_buffer.index", &self.undo_redo_buffer.index)
            .field("undo_redo_buffer.buffer", &"omitted")
            .finish()
    }
}

/// NOTE: A hack to allow keeping this variable out of release builds.
static mut LAST_DEBUG_GRID_DISPLAY_LEN: usize = 0;

pub fn display(terminal: &mut Terminal, builder: &mut Builder) {
    terminal.save_cursor_point();

    // This length ensures that the text does not touch the grid.
    // `builder.point.x` will be the point of the first cell.
    let max_length = (builder.point.x - builder.grid.max_clues_size.width) as usize;

    let clear_spaces = &" ".repeat(unsafe { LAST_DEBUG_GRID_DISPLAY_LEN });
    draw_chunks(terminal, clear_spaces, max_length);

    let string = format!("{:?}", builder.grid);
    draw_chunks(terminal, &string, max_length);

    unsafe {
        LAST_DEBUG_GRID_DISPLAY_LEN = string.len();
    }

    terminal.restore_cursor_point();
}

fn draw_chunks(terminal: &mut Terminal, string: &str, max_length: usize) {
    for (index, line) in string.as_bytes().chunks(max_length).enumerate() {
        terminal.set_cursor(Point {
            x: 0,
            y: index as u16 + 1,
        });
        terminal.write_bytes(line);
    }
}
