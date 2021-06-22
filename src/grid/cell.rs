use std::{borrow::Cow, time::Instant};
use terminal::{
    util::{Color, Point},
    Terminal,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    /// An umarked cell.
    Empty,
    /// Used to mark filled cells.
    Filled,
    /// Used to mark cells that may be filled. Useful for doing "what if" reasoning.
    Maybed,
    /// Used to mark cells that are certainly empty.
    Crossed,
    /// Used for indicating cells that were measured using the measurement tool.
    ///
    /// When this cell is saved, the index is not preserved.
    Measured(Option<usize>),
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Empty
    }
}

impl From<bool> for Cell {
    fn from(filled: bool) -> Self {
        filled.then(|| Cell::Filled).unwrap_or_default()
    }
}

impl Cell {
    pub fn get_color(&self) -> Color {
        match self {
            Cell::Empty => Color::default(),
            Cell::Filled => Color::White,
            Cell::Maybed => Color::Blue,
            Cell::Crossed => Color::Red,
            Cell::Measured(_) => Color::Green,
        }
    }

    pub fn get_highlighted_color(&self) -> Color {
        match self {
            Cell::Empty => Color::DarkGray,
            Cell::Filled => Color::Gray,
            Cell::Maybed => Color::DarkBlue,
            Cell::Crossed => Color::DarkRed,
            Cell::Measured(_) => Color::DarkGreen,
        }
    }

    pub fn draw(&self, terminal: &mut Terminal, point: Point, highlight: bool) {
        /// Every 5 cells, the color changes to make the grid and its cells easier to look at and distinguish.
        const SEPARATION_POINT: u16 = 5;

        fn draw(
            terminal: &mut Terminal,
            foreground_color: Option<Color>,
            background_color: Color,
            content: Cow<'static, str>,
        ) {
            terminal.set_background_color(background_color);
            if let Some(foreground_color) = foreground_color {
                terminal.set_foreground_color(foreground_color);
            }
            terminal.write(&content);
        }

        let mut background_color = if highlight {
            self.get_highlighted_color()
        } else {
            self.get_color()
        };

        let (foreground_color, background_color, content) = match self {
            Cell::Empty => {
                let x_reached_point = point.x / SEPARATION_POINT % 2 == 0;
                let y_reached_point = point.y / SEPARATION_POINT % 2 == 0;
                let mut background_color_byte = if x_reached_point ^ y_reached_point {
                    238
                } else {
                    240
                };

                if highlight {
                    background_color_byte -= 3;
                }

                background_color = Color::Byte(background_color_byte);

                (None, background_color, "  ".into())
            }
            Cell::Measured(index) => {
                let (foreground_color, content) = if let Some(index) = index {
                    (Some(Color::Black), format!("{:>2}", index).into())
                } else {
                    (None, "  ".into())
                };

                (foreground_color, background_color, content)
            }
            _ => (None, background_color, "  ".into()),
        };

        draw(terminal, foreground_color, background_color, content);
    }
}

#[derive(Default)]
pub struct CellPlacement {
    pub cell: Option<Cell>,
    /// The time of when the first cell was placed.
    pub starting_time: Option<Instant>,
    pub selected_cell_point: Option<Point>,
    pub measurement_point: Option<Point>,
    pub fill: bool,
}

use crate::{grid::builder::Builder, undo_redo_buffer, State};

pub const fn get_cell_point_from_cursor_point(cursor_point: Point, builder: &Builder) -> Point {
    Point {
        x: (cursor_point.x - builder.point.x) / 2,
        y: cursor_point.y - builder.point.y,
    }
}

pub fn draw_highlighted_cells(
    terminal: &mut Terminal,
    builder: &Builder,
    hovered_cell_point: Point,
) {
    fn highlight_cell(terminal: &mut Terminal, mut cursor_point: Point, builder: &Builder) {
        if (cursor_point.x - builder.point.x) % 2 != 0 {
            cursor_point.x -= 1;
        }
        terminal.set_cursor(cursor_point);
        let cell_point = get_cell_point_from_cursor_point(cursor_point, builder);
        let cell = builder.grid.get_cell(cell_point);
        cell.draw(terminal, cell_point, true);
    }

    // From the left of the grid to the pointer
    for x in builder.point.x..=hovered_cell_point.x - 2 {
        let point = Point {
            x,
            ..hovered_cell_point
        };
        highlight_cell(terminal, point, builder);
    }
    // From the pointer to the right of the grid
    for x in hovered_cell_point.x + 2..builder.point.x + builder.grid.size.width * 2 {
        let point = Point {
            x,
            ..hovered_cell_point
        };
        highlight_cell(terminal, point, builder);
    }
    // From the top of the grid to the pointer
    for y in builder.point.y..hovered_cell_point.y {
        let point = Point {
            y,
            ..hovered_cell_point
        };
        highlight_cell(terminal, point, builder);
    }
    // From the pointer to the bottom of the grid
    for y in hovered_cell_point.y + 1..builder.point.y + builder.grid.size.height {
        let point = Point {
            y,
            ..hovered_cell_point
        };
        highlight_cell(terminal, point, builder);
    }

    terminal.reset_colors();
}

impl CellPlacement {
    pub fn place(
        &mut self,
        terminal: &mut Terminal,
        builder: &mut Builder,
        selected_cell_point: Point,
        mut cell_to_place: Cell,
        editor_toggled: bool,
    ) -> State {
        let starting_time = self.starting_time.get_or_insert(Instant::now());

        let cell_point = get_cell_point_from_cursor_point(selected_cell_point, builder);

        let grid_cell = builder.grid.get_mut_cell(cell_point);

        *grid_cell = if let Some(cell) = self.cell {
            if *grid_cell == cell {
                builder.draw_grid(terminal);

                // We know that this point is hovered
                draw_highlighted_cells(terminal, &builder, selected_cell_point);

                return State::Continue;
            }

            cell
        } else {
            if *grid_cell == cell_to_place {
                cell_to_place = Cell::default();
            }
            self.cell = Some(cell_to_place);

            if self.fill {
                let cell = *grid_cell;

                crate::grid::tools::fill::fill(&mut builder.grid, cell_point, cell, cell_to_place);

                builder
                    .grid
                    .undo_redo_buffer
                    .push(undo_redo_buffer::Operation::Fill {
                        point: cell_point,
                        first_cell: cell,
                        fill_cell: cell_to_place,
                    });

                self.fill = false;

                let all_clues_solved = builder.draw_all(terminal);

                if all_clues_solved {
                    return State::Solved(starting_time.elapsed());
                } else {
                    return State::ClearAlert;
                }
            }

            cell_to_place
        };
        let cell = *grid_cell;

        builder
            .grid
            .undo_redo_buffer
            .push(undo_redo_buffer::Operation::SetCell {
                point: cell_point,
                cell,
            });

        if editor_toggled {
            builder.rebuild_clues(terminal, cell_point);

            // The grid shouldn't be solved while editing it
            #[allow(unused_must_use)]
            {
                builder.draw_all(terminal);
            }
        } else {
            let all_clues_solved = builder.draw_all(terminal);

            if all_clues_solved {
                return State::Solved(starting_time.elapsed());
            }
        }

        // We know that this point is hovered
        draw_highlighted_cells(terminal, &builder, selected_cell_point);

        State::Continue
    }
}
