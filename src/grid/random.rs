use super::{Cell, Grid};
use terminal::util::Size;

fn random_cells(size: u32) -> Vec<Cell> {
    let mut cells = Vec::<Cell>::with_capacity(size as usize);

    for _ in 0..size {
        cells.push(Cell::from(fastrand::bool()));
    }

    cells
}

impl Grid {
    pub fn random(size: Size) -> Grid {
        Self::new(size, random_cells(size.product()))
    }
}
