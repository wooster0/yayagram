use super::Grid;
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

    /// Draws the top clues while also returning whether all of them were solved ones.
    fn draw_top_clues(&mut self, terminal: &mut Terminal) -> bool {
        let mut highlighted = true;
        let mut all_solved = true;
        for (x, vertical_clues_solution) in self.grid.vertical_clues_solutions.iter().enumerate() {
            let vertical_clues = self.grid.get_vertical_clues(x as u16);
            let solved = vertical_clues.eq(vertical_clues_solution.iter().copied());

            if highlighted {
                terminal.set_background_color(HIGHLIGHTED_CLUE_BACKGROUND_COLOR);
            }
            if solved {
                terminal.set_foreground_color(Color::DarkGray);
            } else if !vertical_clues_solution.is_empty() {
                all_solved = false;
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

        all_solved
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

    /// Draws the left clues while also returning whether all of them were solved ones.
    fn draw_left_clues(&mut self, terminal: &mut Terminal) -> bool {
        self.point.x -= 2;
        terminal.set_cursor(self.point);
        let mut highlighted = true;
        let mut all_solved = true;
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
            } else if !horizontal_clues_solution.is_empty() {
                all_solved = false;
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

        all_solved
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

    /// Draws all clues, the top clues and the left clues while also returning whether all the drawn clues were solved ones.
    fn draw_clues(&mut self, terminal: &mut Terminal) -> bool {
        let previous_point = self.point;
        let all_top_clues_solved = self.draw_top_clues(terminal);
        self.point = previous_point;

        let previous_point = self.point;
        let all_left_clues_solved = self.draw_left_clues(terminal);
        self.point = previous_point;

        all_top_clues_solved && all_left_clues_solved
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

    /// Draws the cells.
    pub fn draw_cells(&mut self, terminal: &mut Terminal) {
        let previous_point_y = self.point.y;
        for (y, cells) in self
            .grid
            .cells
            .chunks(self.grid.size.width as usize)
            .enumerate()
        {
            terminal.set_cursor(self.point);
            let previous_point_x = self.point.x;
            for (x, cell) in cells.iter().enumerate() {
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

    /// Draws the clues and the cells while also returning whether all the drawn clues were solved ones (i.e. whether the grid was solved).
    #[must_use]
    pub fn draw_all(&mut self, terminal: &mut Terminal) -> bool {
        self.draw_cells(terminal);

        let all_clues_solved = self.draw_clues(terminal);

        all_clues_solved
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Cell;
    use terminal::util::Size;

    fn get_terminal_and_builder() -> (Terminal, Builder) {
        let size = Size::new(10, 5);
        let grid = Grid::new(size.clone(), vec![Cell::Empty; size.product() as usize]);
        let terminal = Terminal::new().unwrap();
        let builder = Builder::new(&terminal, grid);
        (terminal, builder)
    }

    #[test]
    fn test_contains() {
        let (_, builder) = get_terminal_and_builder();

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
        let (mut terminal, mut builder) = get_terminal_and_builder();

        let previous_point = builder.point;
        builder.clear_clues(&mut terminal);
        assert_eq!(previous_point, builder.point);
    }

    #[test]
    fn test_draw_cells() {
        let (mut terminal, mut builder) = get_terminal_and_builder();

        let previous_point = builder.point;
        builder.draw_cells(&mut terminal);
        assert_eq!(previous_point, builder.point);
    }

    #[test]
    fn test_draw_all() {
        let (mut terminal, mut builder) = get_terminal_and_builder();

        let previous_point = builder.point;
        #[allow(unused_must_use)]
        {
            builder.draw_all(&mut terminal);
        }
        assert_eq!(previous_point, builder.point);
    }
}
