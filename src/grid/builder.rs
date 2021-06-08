use super::Grid;
use itertools::Itertools;
use terminal::{
    util::{Color, Point},
    Terminal,
};

/// Gets a point to the first cell of the grid which is together with its clues centered on the screen.
pub const fn centered_point(terminal: &Terminal, grid: &Grid) -> Point {
    let grid_width_half = grid.size.width; // No division because blocks are 2 characters
    let grid_height_half = grid.size.height / 2;

    let max_clues_width_half = grid.max_clues_size.width / 2;
    let max_clues_height_half = grid.max_clues_size.height / 2;

    Point {
        x: terminal.size.width / 2 - grid_width_half + max_clues_width_half,
        y: terminal.size.height / 2 - grid_height_half + max_clues_height_half,
    }
}

const HIGHLIGHTED_CLUE_BACKGROUND_COLOR: Color = Color::Byte(238);

/// Builds and draws the grid to the screen.
pub struct Builder {
    pub grid: Grid,
    pub point: Point,
}

impl Builder {
    pub fn new(terminal: &Terminal, grid: Grid) -> Self {
        let point = centered_point(terminal, &grid);

        Self { grid, point }
    }

    /// Checks whether the point is within the grid on the screen.
    pub fn contains(&self, point: Point) -> bool {
        (self.point.y..self.point.y + self.grid.size.height).contains(&point.y)
            && (self.point.x..self.point.x + self.grid.size.width * 2).contains(&point.x)
    }

    /// Draws the top clues while also returning the amount of solved clue rows.
    fn draw_top_clues(&mut self, terminal: &mut Terminal) -> usize {
        let mut highlighted = true;
        let mut solved_rows = 0;
        for (x, vertical_clues_solution) in self.grid.vertical_clues_solutions.iter().enumerate() {
            let vertical_clues = self.grid.get_vertical_clues(x as u16);
            let solved = vertical_clues.eq(vertical_clues_solution.iter().copied());

            if highlighted {
                terminal.set_background_color(HIGHLIGHTED_CLUE_BACKGROUND_COLOR);
            }
            if solved {
                terminal.set_foreground_color(Color::DarkGray);
                solved_rows += 1;
            }

            let previous_point_y = self.point.y;
            for clue in vertical_clues_solution.iter().rev() {
                self.point.y -= 1;
                terminal.set_cursor(self.point);
                terminal.write(&format!("{:<2}", clue));
            }
            // We need to reset the colors because we don't always set both the background and foreground color
            terminal.reset_colors();
            highlighted = !highlighted;
            self.point.y = previous_point_y;
            self.point.x += 2;
        }

        solved_rows
    }
    /// Clears the top clues, only graphically.
    fn clear_top_clues(&mut self, terminal: &mut Terminal) {
        let mut highlighted = true;
        for vertical_clues_solution in self.grid.vertical_clues_solutions.iter() {
            let previous_point_y = self.point.y;

            for _ in vertical_clues_solution.iter().rev() {
                self.point.y -= 1;
                terminal.set_cursor(self.point);
                terminal.write("  ");
            }
            highlighted = !highlighted;

            self.point.y = previous_point_y;
            self.point.x += 2;
        }
    }

    /// Draws the left clues while also returning the amount of solved clue rows.
    fn draw_left_clues(&mut self, terminal: &mut Terminal) -> usize {
        self.point.x -= 2;
        terminal.set_cursor(self.point);
        let mut highlighted = true;
        let mut solved_rows = 0;
        for (y, horizontal_clues_solution) in
            self.grid.horizontal_clues_solutions.iter().enumerate()
        {
            let horizontal_clues = self.grid.get_horizontal_clues(y as u16);
            let solved = horizontal_clues.eq(horizontal_clues_solution.iter().copied());

            if highlighted {
                terminal.set_background_color(HIGHLIGHTED_CLUE_BACKGROUND_COLOR);
            }
            if solved {
                terminal.set_foreground_color(Color::DarkGray);
                solved_rows += 1;
            }

            let previous_point_x = self.point.x;
            for clue in horizontal_clues_solution.iter().rev() {
                terminal.write(&format!("{:>2}", clue));
                terminal.move_cursor_left(4);
                self.point.x -= 4;
            }
            // We need to reset the colors because we don't always set both the background and foreground color
            terminal.reset_colors();
            highlighted = !highlighted;
            self.point.x = previous_point_x;
            self.point.y += 1;
            terminal.set_cursor(self.point);
        }

        solved_rows
    }
    /// Clears the left clues, only graphically.
    fn clear_left_clues(&mut self, terminal: &mut Terminal) {
        self.point.x -= 2;
        terminal.set_cursor(self.point);
        let mut highlighted = true;
        for horizontal_clues_solution in self.grid.horizontal_clues_solutions.iter() {
            let previous_point_x = self.point.x;
            for _ in horizontal_clues_solution.iter().rev() {
                terminal.write("  ");
                terminal.move_cursor_left(4);
                self.point.x -= 4;
            }
            terminal.reset_colors();
            highlighted = !highlighted;
            self.point.x = previous_point_x;
            self.point.y += 1;
            terminal.set_cursor(self.point);
        }
    }

    /// Draws the top clues and the left clues while also returning the amount of solved clue rows.
    fn draw_clues(&mut self, terminal: &mut Terminal) -> usize {
        let previous_point = self.point;
        let solved_top_rows = self.draw_top_clues(terminal);
        self.point = previous_point;

        let previous_point = self.point;
        let solved_left_rows = self.draw_left_clues(terminal);
        self.point = previous_point;

        solved_top_rows + solved_left_rows
    }
    /// Clears all clues, only graphically.
    pub fn clear_clues(&mut self, terminal: &mut Terminal) {
        let previous_point = self.point;
        self.clear_top_clues(terminal);
        self.point = previous_point;

        let previous_point = self.point;
        self.clear_left_clues(terminal);
        self.point = previous_point;
    }

    /// Draws the grid.
    pub fn draw_grid(&mut self, terminal: &mut Terminal) {
        let previous_point_y = self.point.y;
        for (y, row) in self
            .grid
            .cells
            .chunks(self.grid.size.width as usize)
            .enumerate()
        {
            terminal.set_cursor(self.point);
            let previous_point_x = self.point.x;
            for (x, cell) in row.iter().enumerate() {
                let point = Point {
                    x: x as u16,
                    y: y as u16,
                };
                cell.draw(terminal, point, false);
                terminal.reset_colors();
                self.point.x += 2;
            }
            self.point.x = previous_point_x;
            self.point.y += 1;
        }
        self.point.y = previous_point_y;
    }

    fn draw_half_block(terminal: &mut Terminal) {
        terminal.write("▄");
    }

    /// Draws the grid in smaller form on the top left, making it easier to see the whole picture.
    ///
    /// NOTE: Perhaps at some point in the future [sixel](https://en.wikipedia.org/wiki/Sixel) can be supported.
    ///       Maybe exclusively for cases where the window size does not suffice.
    ///
    /// NOTE: Perhaps at some point, if stabilized, `array_chunks` can be used to implement this.
    pub fn draw_picture(&mut self, terminal: &mut Terminal) {
        let previous_point = self.point;

        self.point.x -= self.grid.size.width;
        self.point.y -= self.grid.size.height / 2;
        self.point.y -= 1;

        let mut chunks = self.grid.cells.chunks(self.grid.size.width as usize);

        if self.grid.size.height % 2 == 1 {
            let uneven_chunk = chunks.next().unwrap();

            terminal.set_cursor(self.point);
            for cell in uneven_chunk {
                terminal.set_foreground_color(cell.get_color());
                Self::draw_half_block(terminal);
            }
        }

        for (first_row, second_row) in chunks.tuples() {
            self.point.y += 1;
            terminal.set_cursor(self.point);
            for (first_cell, second_cell) in first_row.iter().zip(second_row) {
                terminal.set_background_color(first_cell.get_color());
                terminal.set_foreground_color(second_cell.get_color());
                Self::draw_half_block(terminal);
            }
        }

        self.point = previous_point;
    }

    /// Draws the progress of solved clue rows as a bar at the bottom.
    fn draw_progress_bar(&mut self, terminal: &mut Terminal, solved_rows: usize) {
        let previous_point_y = self.point.y;

        self.point.y += self.grid.size.height;

        terminal.set_cursor(self.point);

        let grid_width = self.grid.size.width * 2;
        let width = ((solved_rows as f64 / (self.grid.size.width + self.grid.size.height) as f64)
            * grid_width as f64) as u16;

        terminal.set_foreground_color(Color::Gray);
        for _ in 0..width {
            Self::draw_half_block(terminal);
        }

        let rest = grid_width - width;
        if rest > 0 {
            terminal.set_foreground_color(Color::DarkGray);
            for _ in 0..rest {
                Self::draw_half_block(terminal);
            }
        }

        self.point.y = previous_point_y;
    }

    /// Draws the grid, the picture and the clues while also returning whether all the drawn clues were solved ones (i.e. whether the grid was solved).
    #[must_use]
    pub fn draw_all(&mut self, terminal: &mut Terminal) -> bool {
        self.draw_picture(terminal);

        self.draw_grid(terminal);

        let solved_rows = self.draw_clues(terminal);

        self.draw_progress_bar(terminal, solved_rows);

        solved_rows == (self.grid.size.width + self.grid.size.height) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Cell;
    use std::io;
    use terminal::util::Size;

    fn get_terminal_and_builder(stdout: io::StdoutLock) -> (Terminal, Builder) {
        let size = Size::new(10, 5);
        let grid = Grid::new(size.clone(), vec![Cell::Empty; size.product() as usize]);
        let terminal = Terminal::new(stdout).unwrap();
        let builder = Builder::new(&terminal, grid);
        (terminal, builder)
    }

    #[test]
    fn test_contains() {
        let stdout = io::stdout();
        let (_, builder) = get_terminal_and_builder(stdout.lock());

        assert!(!builder.contains(Point {
            x: builder.point.x - 1,
            y: builder.point.y - 1
        }));
        assert!(builder.contains(builder.point));
        assert!(!builder.contains(Point {
            x: builder.point.x + builder.grid.size.width,
            y: builder.point.y + builder.grid.size.height
        }));
    }

    #[test]
    fn test_clear_clues() {
        let stdout = io::stdout();
        let (mut terminal, mut builder) = get_terminal_and_builder(stdout.lock());

        let previous_point = builder.point;
        builder.clear_clues(&mut terminal);
        assert_eq!(previous_point, builder.point);
    }

    #[test]
    fn test_draw_grid() {
        let stdout = io::stdout();
        let (mut terminal, mut builder) = get_terminal_and_builder(stdout.lock());

        let previous_point = builder.point;
        builder.draw_grid(&mut terminal);
        assert_eq!(previous_point, builder.point);
    }

    #[test]
    fn test_draw_picture() {
        let stdout = io::stdout();
        let (mut terminal, mut builder) = get_terminal_and_builder(stdout.lock());

        let previous_point = builder.point;
        builder.draw_picture(&mut terminal);
        assert_eq!(previous_point, builder.point);
    }

    #[test]
    fn test_draw_all() {
        let stdout = io::stdout();
        let (mut terminal, mut builder) = get_terminal_and_builder(stdout.lock());

        let previous_point = builder.point;
        #[allow(unused_must_use)]
        {
            builder.draw_all(&mut terminal);
        }
        assert_eq!(previous_point, builder.point);
    }
}
