pub mod builder;
mod cell;
mod random;
pub mod tools;

use crate::undo_redo_buffer::UndoRedoBuffer;
pub use cell::*;
use itertools::Itertools;
use terminal::util::{Point, Size};

/// A single clue specifying how many cells there are in a row at some point.
type Clue = u16;
/// A complete set of clues.
type Clues = Vec<Clue>;

pub struct Grid {
    pub size: Size,
    /// This is where the player's input is stored. It is initially empty.
    pub cells: Vec<Cell>,
    /// The horizontal clue solutions generated out of the initial input.
    pub horizontal_clues_solutions: Vec<Clues>,
    /// The vertical clue solutions generated out of the initial input.
    pub vertical_clues_solutions: Vec<Clues>,
    pub max_clues_size: Size,
    pub undo_redo_buffer: UndoRedoBuffer,
    pub measurement_counter: usize,
}

fn get_index(grid_width: u16, point: Point) -> usize {
    point.y as usize * grid_width as usize + point.x as usize
}

fn get_horizontal_clues(
    cells: &[Cell],
    grid_width: u16,
    y: u16,
) -> impl Iterator<Item = Clue> + '_ {
    (0..grid_width)
        .map(move |x| cells[get_index(grid_width, Point { x, y })] == Cell::Filled)
        .dedup_with_count()
        .filter(|(_, filled)| *filled)
        .map(|(count, _)| count as Clue)
}

fn get_vertical_clues(cells: &[Cell], grid_size: Size, x: u16) -> impl Iterator<Item = Clue> + '_ {
    (0..grid_size.height)
        .map(move |y| cells[get_index(grid_size.width, Point { x, y })] == Cell::Filled)
        .dedup_with_count()
        .filter(|(_, filled)| *filled)
        .map(|(count, _)| count as Clue)
}

impl Grid {
    /// Creates a new grid. `cells`' `len` must be equal to the product of the width and height of `size`.
    pub fn new(size: Size, mut cells: Vec<Cell>) -> Self {
        debug_assert_eq!(cells.len(), size.product() as usize);

        let mut horizontal_clues_solutions = Vec::<Clues>::new();
        for y in 0..size.height {
            let horizontal_clues_solution: Clues =
                get_horizontal_clues(&cells, size.width, y).collect();
            horizontal_clues_solutions.push(horizontal_clues_solution);
        }
        let max_clues_width = horizontal_clues_solutions
            .iter()
            .map(|horizontal_clues_solution| horizontal_clues_solution.len() * 2)
            .max()
            .unwrap() as u16; // The iterator won't be empty

        let mut vertical_clues_solutions = Vec::<Clues>::new();
        for x in 0..size.width {
            let vertical_clues_solution: Clues = get_vertical_clues(&cells, size, x).collect();
            vertical_clues_solutions.push(vertical_clues_solution);
        }
        let max_clues_height = vertical_clues_solutions
            .iter()
            .map(|vertical_clues_solution| vertical_clues_solution.len())
            .max()
            .unwrap() as u16; // The iterator won't be empty

        for cell in &mut cells {
            if *cell == Cell::Filled {
                *cell = Cell::Empty;
            }
        }

        let max_clues_size = Size {
            width: max_clues_width,
            height: max_clues_height,
        };

        let undo_redo_buffer = UndoRedoBuffer::default();

        let measurement_counter = 0;

        Self {
            size,
            cells,
            horizontal_clues_solutions,
            vertical_clues_solutions,
            max_clues_size,
            undo_redo_buffer,
            measurement_counter,
        }
    }

    fn cell_panic(point: Point, index: usize) -> ! {
        panic!(
            "cell access at {} with index {} is out of bounds",
            point, index
        );
    }

    fn get_cell(&self, point: Point) -> Cell {
        let index = get_index(self.size.width, point);
        *self
            .cells
            .get(index)
            .unwrap_or_else(|| Self::cell_panic(point, index))
    }

    pub fn get_mut_cell(&mut self, point: Point) -> &mut Cell {
        let index = get_index(self.size.width, point);
        self.cells
            .get_mut(index)
            .unwrap_or_else(|| Self::cell_panic(point, index))
    }

    fn get_horizontal_clues(&self, y: u16) -> impl Iterator<Item = Clue> + '_ {
        get_horizontal_clues(&self.cells, self.size.width, y)
    }

    fn get_vertical_clues(&self, x: u16) -> impl Iterator<Item = Clue> + '_ {
        get_vertical_clues(&self.cells, self.size, x)
    }

    pub fn clear(&mut self) {
        self.cells.fill_with(Default::default);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Grid {
        /// Creates a new grid from the given line of strings.
        /// A `1` represents a filled cell, a ` ` represents an empty cell.
        ///
        /// # Examples
        ///
        /// ```
        /// let lines = [
        ///    "111 1",
        ///    "11111",
        ///    "1 111"
        /// ]
        /// let grid = Grid::from_lines(lines);
        /// ```
        fn from_lines(lines: &[&str]) -> Grid {
            let width = lines.iter().map(|line| line.len()).max().unwrap();
            let height = lines.len();
            let size = Size {
                width: width as u16,
                height: height as u16,
            };
            let mut cells = Vec::<Cell>::with_capacity(size.product() as usize);
            for line in lines {
                for char in line.chars() {
                    cells.push(match char {
                        '1' => Cell::Filled,
                        ' ' => Cell::Empty,
                        _ => panic!("the strings must only contain '1' or ' '"),
                    });
                }
            }
            Grid::new(size, cells)
        }
    }

    #[test]
    fn test_squared_grid() {
        #[rustfmt::skip]
        let grid = Grid::from_lines(&[
            "1 1 111 1 ",
            " 1 11 111 ",
            "1111 11  1",
            "1 11 1  11",
            "1  111  11",
        ]);

        assert_eq!(
            grid.horizontal_clues_solutions,
            [
                vec![1, 1, 3, 1],
                vec![1, 2, 3],
                vec![4, 2, 1],
                vec![1, 2, 1, 2],
                vec![1, 3, 2],
            ]
        );

        assert_eq!(
            grid.vertical_clues_solutions,
            [
                vec![1, 3],
                vec![2],
                vec![1, 2],
                vec![4],
                vec![2, 1],
                vec![1, 3],
                vec![3],
                vec![1],
                vec![2, 2],
                vec![3]
            ]
        );
    }

    #[test]
    fn test_non_squared_grid() {
        #[rustfmt::skip]
        let grid = Grid::from_lines(&[
            " 111",
            " 1 1",
            "11 1",
            "1 1 ",
            "1  1",
            "  1 ",
        ]);

        assert_eq!(
            grid.horizontal_clues_solutions,
            [
                vec![3],
                vec![1, 1],
                vec![2, 1],
                vec![1, 1],
                vec![1, 1],
                vec![1],
            ]
        );

        assert_eq!(
            grid.vertical_clues_solutions,
            [vec![3], vec![3], vec![1, 1, 1], vec![3, 1]]
        );
    }

    #[test]
    fn test_clear() {
        #[rustfmt::skip]
        let mut grid = Grid::from_lines(&[
            "1111",
            "1111",
            "1111",
        ]);

        grid.clear();

        assert!(grid.cells.iter().all(|cell| *cell == Cell::Empty));
    }
}
