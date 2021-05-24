use rand::distributions::{Bernoulli, Distribution};
use rand::rngs::SmallRng;
use rand::SeedableRng;

use super::{Cell, Grid};
use terminal::util::Size;

fn random_cells(size: u32) -> Vec<Cell> {
    let mut cells = Vec::<Cell>::with_capacity(size as usize);
    let mut small_rng = SmallRng::from_entropy();
    let distribution = Bernoulli::new(0.75).unwrap();

    for _ in 0..size {
        cells.push(Cell::from(distribution.sample(&mut small_rng)));
    }

    cells
}

impl Grid {
    pub fn random(size: Size) -> Grid {
        Self::new(size.clone(), random_cells(size.product()))
    }
}
