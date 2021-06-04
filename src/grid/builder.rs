use super::Grid;
use terminal::{
    util::{Color, Point},
    Terminal,
};

#[derive(Clone, PartialEq, Debug)]
pub struct Cursor {
    pub point: Point,
}

impl Cursor {
    pub fn centered(terminal: &Terminal, grid: &Grid) -> Self {
        let grid_width = grid.size.width; // No division because blocks are 2 characters
        let grid_height = grid.size.height / 2;

        let max_clues_width = grid.max_clues_size.width / 2;
        let max_clues_height = grid.max_clues_size.height / 2;

        Self {
            point: Point {
                x: terminal.size.width / 2 - grid_width + max_clues_width,
                y: terminal.size.height / 2 - grid_height + max_clues_height,
            },
        }
    }

    fn update(&mut self, terminal: &mut Terminal) {
        terminal.set_cursor(self.point);
    }
}

/// Builds and draws the grid to the screen.
pub struct Builder {
    pub grid: Grid,
    pub cursor: Cursor,
}

impl Builder {
    pub fn new(terminal: &Terminal, grid: Grid) -> Self {
        let cursor = Cursor::centered(terminal, &grid);

        Self { grid, cursor }
    }

    /// Checks whether the point is within the grid.
    pub fn contains(&self, point: Point) -> bool {
        (self.cursor.point.y..self.cursor.point.y + self.grid.size.height).contains(&point.y)
            && (self.cursor.point.x..self.cursor.point.x + self.grid.size.width * 2)
                .contains(&point.x)
    }

    /// Draws the top clues while also returning whether all of them were solved ones.
    fn draw_top_clues(&mut self, terminal: &mut Terminal) -> bool {
        let mut highlighted = true;
        let mut all_solved = true;
        for (x, vertical_clues_solution) in self.grid.vertical_clues_solutions.iter().enumerate() {
            let vertical_clues = self.grid.get_vertical_clues(x as u16);
            let solved = vertical_clues.eq(vertical_clues_solution.iter().copied());

            if highlighted {
                terminal.set_background_color(Color::Byte(238));
            }
            if solved {
                terminal.set_foreground_color(Color::DarkGray);
            } else if !vertical_clues_solution.is_empty() {
                all_solved = false;
            }

            let previous_cursor_y = self.cursor.point.y;
            for clue in vertical_clues_solution.iter().rev() {
                self.cursor.point.y -= 1;
                self.cursor.update(terminal);
                terminal.write(&format!("{:<2}", clue));
            }
            // We need to reset the colors because we don't always set both the background and foreground color
            terminal.reset_colors();
            highlighted = !highlighted;
            self.cursor.point.y = previous_cursor_y;
            self.cursor.point.x += 2;
        }

        all_solved
    }
    /// Clears the top clues, only graphically.
    fn clear_top_clues(&mut self, terminal: &mut Terminal) {
        let mut highlighted = true;
        for vertical_clues_solution in self.grid.vertical_clues_solutions.iter() {
            let previous_cursor_y = self.cursor.point.y;

            for _ in vertical_clues_solution.iter().rev() {
                self.cursor.point.y -= 1;
                self.cursor.update(terminal);
                terminal.write("  ");
            }
            highlighted = !highlighted;

            self.cursor.point.y = previous_cursor_y;
            self.cursor.point.x += 2;
        }
    }

    /// Draws the left clues while also returning whether all of them were solved ones.
    fn draw_left_clues(&mut self, terminal: &mut Terminal) -> bool {
        terminal.move_cursor_left(2);
        self.cursor.point.x -= 2;
        let mut highlighted = true;
        let mut all_solved = true;
        for (y, horizontal_clues_solution) in
            self.grid.horizontal_clues_solutions.iter().enumerate()
        {
            let horizontal_clues = self.grid.get_horizontal_clues(y as u16);
            let solved = horizontal_clues.eq(horizontal_clues_solution.iter().copied());

            if highlighted {
                terminal.set_background_color(Color::Byte(238));
            }
            if solved {
                terminal.set_foreground_color(Color::DarkGray);
            } else if !horizontal_clues_solution.is_empty() {
                all_solved = false;
            }

            let previous_cursor_x = self.cursor.point.x;
            for clue in horizontal_clues_solution.iter().rev() {
                terminal.write(&format!("{:>2}", clue));
                terminal.move_cursor_left(4);
                self.cursor.point.x -= 4;
            }
            // We need to reset the colors because we don't always set both the background and foreground color
            terminal.reset_colors();
            highlighted = !highlighted;
            self.cursor.point.x = previous_cursor_x;
            self.cursor.point.y += 1;
            self.cursor.update(terminal);
        }

        all_solved
    }
    /// Clears the left clues, only graphically.
    fn clear_left_clues(&mut self, terminal: &mut Terminal) {
        terminal.move_cursor_left(2);
        self.cursor.point.x -= 2;
        let mut highlighted = true;
        for horizontal_clues_solution in self.grid.horizontal_clues_solutions.iter() {
            let previous_cursor_x = self.cursor.point.x;
            for _ in horizontal_clues_solution.iter().rev() {
                terminal.write("  ");
                terminal.move_cursor_left(4);
                self.cursor.point.x -= 4;
            }
            terminal.reset_colors();
            highlighted = !highlighted;
            self.cursor.point.x = previous_cursor_x;
            self.cursor.point.y += 1;
            self.cursor.update(terminal);
        }
    }

    /// Draws all clues, the top clues and the left clues while also returning whether all the drawn clues were solved ones.
    fn draw_clues(&mut self, terminal: &mut Terminal) -> bool {
        self.cursor.update(terminal);

        let previous_cursor_point = self.cursor.point;
        let all_top_clues_solved = self.draw_top_clues(terminal);
        self.cursor.point = previous_cursor_point;

        self.cursor.update(terminal);

        let previous_cursor_point = self.cursor.point;
        let all_left_clues_solved = self.draw_left_clues(terminal);
        self.cursor.point = previous_cursor_point;

        self.cursor.update(terminal);

        all_top_clues_solved && all_left_clues_solved
    }
    /// Clears all clues, only graphically.
    pub fn clear_clues(&mut self, terminal: &mut Terminal) {
        self.cursor.update(terminal);

        let previous_cursor_point = self.cursor.point;
        self.clear_top_clues(terminal);
        self.cursor.point = previous_cursor_point;

        self.cursor.update(terminal);

        let previous_cursor_point = self.cursor.point;
        self.clear_left_clues(terminal);
        self.cursor.point = previous_cursor_point;

        self.cursor.update(terminal);
    }

    fn draw_cells(&mut self, terminal: &mut Terminal) {
        let previous_cursor_y = self.cursor.point.y;
        for (y, cells) in self
            .grid
            .cells
            .chunks(self.grid.size.width as usize)
            .enumerate()
        {
            let previous_cursor_x = self.cursor.point.x;
            for (x, cell) in cells.iter().enumerate() {
                let point = Point {
                    x: x as u16,
                    y: y as u16,
                };
                cell.draw(terminal, point, false);
                terminal.reset_colors();
                self.cursor.point.x += 2;
            }
            self.cursor.point.x = previous_cursor_x;
            self.cursor.point.y += 1;
            self.cursor.update(terminal);
        }
        self.cursor.point.y = previous_cursor_y;
    }

    /// Draws the clues and the cells while also returning whether all the drawn clues were solved ones.
    #[must_use]
    pub fn draw(&mut self, terminal: &mut Terminal) -> bool {
        let all_clues_solved = self.draw_clues(terminal);

        self.draw_cells(terminal);

        all_clues_solved
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Cell;
    use terminal::util::Size;

    #[test]
    fn test_draw() {
        let mut terminal = Terminal::new().unwrap();
        let size = Size::new(5, 5);
        let grid = Grid::new(size.clone(), vec![Cell::Empty; size.product() as usize]);
        let mut builder = Builder::new(&terminal, grid);

        let previous_cursor = builder.cursor.clone();
        let all_clues_solved = builder.draw(&mut terminal);
        assert!(all_clues_solved);
        assert_eq!(builder.cursor, previous_cursor);
    }
}
