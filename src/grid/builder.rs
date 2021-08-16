use super::{Cell, Grid};
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

    pub fn get_center(&self) -> Point {
        let mut width = self.grid.size.width;

        if width % 2 == 1 {
            width -= 1;
        }

        Point {
            x: self.point.x + width,
            y: self.point.y + self.grid.size.height / 2,
        }
    }

    /// Reconstructs the clues associated with the given `cell_point`.
    pub fn rebuild_clues(&mut self, terminal: &mut Terminal, cell_point: Point) {
        self.clear_clues(terminal);
        self.grid.horizontal_clues_solutions[cell_point.y as usize] =
            self.grid.get_horizontal_clues(cell_point.y).collect();
        self.grid.vertical_clues_solutions[cell_point.x as usize] =
            self.grid.get_vertical_clues(cell_point.x).collect();
    }

    /// Draws the top clues while also returning the amount of solved clue rows.
    fn draw_top_clues(&mut self, terminal: &mut Terminal) -> usize {
        let previous_point = self.point;

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
            self.point.y = previous_point_y;

            // We need to reset the colors because we don't always set both the background and foreground color
            terminal.reset_colors();
            highlighted = !highlighted;
            self.point.x += 2;
        }

        self.point = previous_point;

        solved_rows
    }
    /// Clears the top clues, only graphically.
    fn clear_top_clues(&mut self, terminal: &mut Terminal) {
        let previous_point = self.point;

        let mut highlighted = true;
        for vertical_clues_solution in self.grid.vertical_clues_solutions.iter() {
            let previous_point_y = self.point.y;
            for _ in vertical_clues_solution.iter().rev() {
                self.point.y -= 1;
                terminal.set_cursor(self.point);
                terminal.write("  ");
            }
            self.point.y = previous_point_y;

            highlighted = !highlighted;
            self.point.x += 2;
        }

        self.point = previous_point;
    }

    /// Draws the left clues while also returning the amount of solved clue rows.
    fn draw_left_clues(&mut self, terminal: &mut Terminal) -> usize {
        let previous_point = self.point;

        self.point.x -= 2;
        let mut highlighted = true;
        let mut solved_rows = 0;
        for (y, horizontal_clues_solution) in
            self.grid.horizontal_clues_solutions.iter().enumerate()
        {
            terminal.set_cursor(self.point);
            let horizontal_clues = self.grid.get_horizontal_clues(y as u16);
            let solved = horizontal_clues.eq(horizontal_clues_solution.iter().copied());

            if highlighted {
                terminal.set_background_color(HIGHLIGHTED_CLUE_BACKGROUND_COLOR);
            }
            if solved {
                terminal.set_foreground_color(Color::DarkGray);
                solved_rows += 1;
            }

            for clue in horizontal_clues_solution.iter().rev() {
                terminal.write(&format!("{:>2}", clue));
                terminal.move_cursor_left_by(4);
            }
            // We need to reset the colors because we don't always set both the background and foreground color
            terminal.reset_colors();
            highlighted = !highlighted;
            self.point.y += 1;
        }

        self.point = previous_point;

        solved_rows
    }
    /// Clears the left clues, only graphically.
    fn clear_left_clues(&mut self, terminal: &mut Terminal) {
        let previous_point = self.point;

        self.point.x -= 2;
        let mut highlighted = true;
        for horizontal_clues_solution in self.grid.horizontal_clues_solutions.iter() {
            terminal.set_cursor(self.point);
            for _ in horizontal_clues_solution.iter().rev() {
                terminal.write("  ");
                terminal.move_cursor_left_by(4);
            }
            terminal.reset_colors();
            highlighted = !highlighted;
            self.point.y += 1;
        }

        self.point = previous_point;
    }

    /// Draws the top clues and the left clues while also returning the amount of solved clue rows.
    fn draw_clues(&mut self, terminal: &mut Terminal) -> usize {
        let solved_top_rows = self.draw_top_clues(terminal);

        let solved_left_rows = self.draw_left_clues(terminal);

        solved_top_rows + solved_left_rows
    }
    /// Clears all clues, only graphically.
    pub fn clear_clues(&mut self, terminal: &mut Terminal) {
        self.clear_top_clues(terminal);

        self.clear_left_clues(terminal);
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

    fn empty_grid<F>(&mut self, terminal: &mut Terminal, f: F)
    where
        F: Fn(&mut Terminal, Point),
    {
        let previous_point_y = self.point.y;
        for y in 0..self.grid.size.height {
            terminal.set_cursor(self.point);
            let previous_point_x = self.point.x;
            for x in 0..self.grid.size.width {
                f(terminal, Point { x, y });
                self.point.x += 2;
            }
            self.point.x = previous_point_x;
            self.point.y += 1;
        }
        self.point.y = previous_point_y;
    }

    /// Draws an empty grid.
    pub fn draw_empty_grid(&mut self, terminal: &mut Terminal) {
        self.empty_grid(terminal, |terminal, point| {
            Cell::Empty.draw(terminal, point, false);
        });
    }

    /// Clears the empty grid.
    pub fn clear_empty_grid(&mut self, terminal: &mut Terminal) {
        self.empty_grid(terminal, |terminal, _| {
            terminal.write("  ");
        });
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
            for (upper_cell, lower_cell) in first_row.iter().zip(second_row) {
                terminal.set_background_color(upper_cell.get_color());
                terminal.set_foreground_color(lower_cell.get_color());
                Self::draw_half_block(terminal);
            }
        }

        self.point = previous_point;
    }

    /// Draws the progress of solved clue rows as a bar at the bottom.
    fn draw_progress_bar(&mut self, terminal: &mut Terminal, solved_rows: usize) {
        terminal.set_cursor(Point {
            y: self.point.y + self.grid.size.height,
            ..self.point
        });

        let grid_width = self.grid.size.width * 2;
        let percentage = solved_rows as f64 / (self.grid.size.width + self.grid.size.height) as f64;
        let width = (percentage * grid_width as f64) as u16;

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
    }

    pub fn draw_resize_arrow(&mut self, terminal: &mut Terminal) {
        terminal.set_foreground_color(Color::DarkGray);

        #[cfg(not(windows))]
        terminal.write(" ↘");

        // The above doesn't render in many cases on Windows so at least for now,
        // until the situation improves (and https://en.wikipedia.org/wiki/Windows_Terminal becomes the new default on Windows?),
        // this is used as an alternative.
        // Someday the arrow could be used on Windows too.
        //
        // In the future it might be helpful to take a look at the market share of Windows 10 or 11 and decide by that.
        //
        // In this regard, for good Windows terminal compatibility,
        // I generally recommend limiting yourself to characters listed on https://en.wikipedia.org/wiki/Code_page_437
        #[cfg(windows)]
        terminal.write(" +");
    }

    /// Draws the grid, the picture and the clues while also returning whether all the drawn clues were solved ones (i.e. whether the grid was solved).
    #[must_use]
    pub fn draw_all(&mut self, terminal: &mut Terminal) -> bool {
        self.draw_picture(terminal);

        self.draw_grid(terminal);

        let solved_rows = self.draw_clues(terminal);

        self.draw_progress_bar(terminal, solved_rows);

        self.draw_resize_arrow(terminal);

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
        let size = Size {
            width: 10,
            height: 5,
        };
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
    fn test_draw_empty_grid() {
        let stdout = io::stdout();
        let (mut terminal, mut builder) = get_terminal_and_builder(stdout.lock());

        let previous_point = builder.point;
        builder.empty_grid(&mut terminal, |_, _| {});
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
