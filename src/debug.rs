#![allow(unused)]

use crate::grid::{Cell, Grid};
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

/// Sets up the given terminal for debugging usage.
pub fn with<F>(terminal: &mut Terminal, f: F)
where
    F: Fn(&mut Terminal) -> (),
{
    terminal.save_cursor_point();

    // Place cursor below the flush count that is printed by the terminal in debug mode.
    terminal.set_cursor(Point { x: 0, y: 1 });

    f(terminal);

    terminal.restore_cursor_point();
}

/// Prints a debugging message.
pub fn print(terminal: &mut Terminal, message: &str) {
    terminal.write(message);
    terminal.move_cursor_down();
    terminal.set_cursor_x(0);
}
