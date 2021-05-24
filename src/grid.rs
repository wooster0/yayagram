use itertools::Itertools;
use terminal::util::{Color, Size};
pub mod builder;

#[cfg(debug_assertions)]
pub mod debug;

mod random;
use crate::undo_redo_buffer::UndoRedoBuffer;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    /// An umarked cell.
    Empty,
    /// Used to mark filled cells.
    Filled,
    /// Used to mark cells that may be filled. Useful for doing "what if" reasoning.
    ///
    /// NOTE: in VS Code's terminal there's some weird bug that can only be reproduced sometimes
    ///       where you set this cell with the middle mouse wheel and it somehow clears the whole grid.
    ///       I believe the terminal takes a middle click as both a middle click and an R key press, causing the grid to be reset.
    ///       Maybe report this?
    Maybed,
    /// Used to mark cells that are certainly empty.
    Crossed,
}

impl From<bool> for Cell {
    fn from(bool: bool) -> Self {
        if bool {
            Cell::Filled
        } else {
            Cell::Empty
        }
    }
}

impl Cell {
    fn get_color(&self) -> Color {
        match self {
            Cell::Empty => Color::default(),
            Cell::Filled => Color::White,
            Cell::Maybed => Color::Blue,
            Cell::Crossed => Color::Red,
        }
    }

    pub fn get_dark_color(&self) -> Color {
        match self {
            Cell::Empty => Color::DarkGray,
            Cell::Filled => Color::Gray,
            Cell::Maybed => Color::DarkBlue,
            Cell::Crossed => Color::DarkRed,
        }
    }
}

/// A single clue specifying how many cells there are in a row at some point.
type Clue = u16;
/// A complete set of clues.
type Clues = Vec<Clue>;

#[derive(Debug)]
pub struct SolvedClues {
    pub horizontal_clues: Vec<Cell>,
    pub vertical_clues: Vec<Cell>,
}

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
}

fn get_index(width: u16, x: u16, y: u16) -> usize {
    y as usize * width as usize + x as usize
}

pub fn get_horizontal_clues(
    cells: &[Cell],
    width: u16,
    y: u16,
) -> impl Iterator<Item = Clue> + '_ + Clone {
    (0..width)
        .map(move |x| cells[get_index(width, x, y)] == Cell::Filled)
        .dedup_with_count()
        .filter(|(_, filled)| *filled)
        .map(|(count, _)| count as Clue)
}

pub fn get_vertical_clues(
    cells: &[Cell],
    height: u16,
    x: u16,
) -> impl Iterator<Item = Clue> + '_ + Clone {
    (0..height)
        .map(move |y| cells[get_index(height, x, y)] == Cell::Filled)
        .dedup_with_count()
        .filter(|(_, filled)| *filled)
        .map(|(count, _)| count as Clue)
}

impl Grid {
    /// Creates a new grid. `cells` must have a length of `size.width * size.height`.
    pub fn new(size: Size, mut cells: Vec<Cell>) -> Self {
        assert_eq!(
            size.width, size.height,
            "width != height. currently non-squared grids don't work well"
        );
        assert_eq!(cells.len(), (size.width as usize * size.height as usize));

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
            .unwrap() as u16;

        let mut vertical_clues_solutions = Vec::<Clues>::new();
        for x in 0..size.width {
            let vertical_clues_solution: Clues =
                get_vertical_clues(&cells, size.height, x).collect();
            vertical_clues_solutions.push(vertical_clues_solution);
        }
        let max_clues_height = vertical_clues_solutions
            .iter()
            .map(|vertical_clues_solution| vertical_clues_solution.len())
            .max()
            .unwrap() as u16;

        for cell in &mut cells {
            if *cell == Cell::Filled {
                *cell = Cell::Empty;
            }
        }

        let max_clues_size = Size::new(max_clues_width, max_clues_height);

        let undo_redo_buffer = UndoRedoBuffer::default();

        Self {
            size,
            cells,
            horizontal_clues_solutions,
            vertical_clues_solutions,
            max_clues_size,
            undo_redo_buffer,
        }
    }

    /// Creates a new grid from the given line of strings.
    /// Each 1 represents a filled cell.
    /// Each space represents an empty cell.
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
    /// It would look like this when solved:
    /// ```text
    /// ██████  ██
    /// ██████████
    /// ██  ██████
    /// ```
    pub fn from_lines(lines: &[&str]) -> Grid {
        let width = lines.iter().map(|line| line.len()).max().unwrap();
        let height = lines.len();
        let size = Size::new(width as u16, height as u16);
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

    fn cell_panic(x: u16, y: u16, index: usize) -> ! {
        panic!(
            "cell access at ({}, {}) with index {} is out of bounds",
            x, y, index
        );
    }

    pub fn get_cell(&self, x: u16, y: u16) -> Cell {
        let index = get_index(self.size.width, x, y);
        *self
            .cells
            .get(index)
            .unwrap_or_else(|| Self::cell_panic(x, y, index))
    }

    pub fn get_mut_cell(&mut self, x: u16, y: u16) -> &mut Cell {
        let index = get_index(self.size.width, x, y);
        self.cells
            .get_mut(index)
            .unwrap_or_else(|| Self::cell_panic(x, y, index))
    }

    pub fn get_horizontal_clues(&self, y: u16) -> impl Iterator<Item = Clue> + '_ + Clone {
        get_horizontal_clues(&self.cells, self.size.width, y)
    }

    pub fn get_vertical_clues(&self, x: u16) -> impl Iterator<Item = Clue> + '_ + Clone {
        get_vertical_clues(&self.cells, self.size.width, x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solutions() {
        let grid = Grid::from_lines(&[
            "1 1 111 1 ",
            " 1 11 111 ",
            "1111 11  1",
            "1 11 1  11",
            "1  111  11",
            "          ",
            "          ",
            "          ",
            "          ",
            "          ",
        ]);

        assert_eq!(
            grid.horizontal_clues_solutions,
            [
                vec![1, 1, 3, 1],
                vec![1, 2, 3],
                vec![4, 2, 1],
                vec![1, 2, 1, 2],
                vec![1, 3, 2],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![]
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
}
