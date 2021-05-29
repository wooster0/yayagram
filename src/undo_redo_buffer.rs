use crate::grid::{Cell, Grid};
use terminal::util::Point;

#[derive(Clone, Debug)]
pub enum Operation {
    SetCell { point: Point, cell: Cell },
    Measure(Vec<Point>),
    Clear,
}

#[derive(Default, Debug)]
pub struct UndoRedoBuffer {
    pub buffer: Vec<Operation>,
    pub index: usize,
}

impl UndoRedoBuffer {
    pub fn push(&mut self, operation: Operation) {
        if self.index != self.buffer.len() {
            self.buffer.truncate(self.index);
        }
        self.buffer.push(operation);
        self.index += 1;
    }
}

impl Grid {
    /// Tries to undo the last placed cell and returns `true` if that was successful.
    pub fn undo_last_cell(&mut self) -> bool {
        if self.undo_redo_buffer.index > 0 {
            self.undo_redo_buffer.index -= 1;

            self.rebuild();
            true
        } else {
            false
        }
    }

    /// Tries to redo the last undone cell and returns `true` if that was successful.
    pub fn redo_last_cell(&mut self) -> bool {
        if self.undo_redo_buffer.index != self.undo_redo_buffer.buffer.len() {
            self.undo_redo_buffer.index += 1;

            self.rebuild();
            true
        } else {
            false
        }
    }

    fn rebuild(&mut self) {
        self.cells.fill_with(Default::default);

        for operation in self.undo_redo_buffer.buffer.clone()[..self.undo_redo_buffer.index].iter()
        {
            match operation {
                Operation::SetCell { point, cell } => {
                    let grid_cell = self.get_mut_cell(point.x, point.y);
                    *grid_cell = *cell;
                }
                Operation::Measure(line_points) => {
                    crate::event::set_measured_cells(self, line_points);
                }
                Operation::Clear => {
                    self.cells.fill_with(Default::default);
                }
            }
        }
    }
}
