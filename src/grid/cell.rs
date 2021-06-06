use std::borrow::Cow;
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

    pub fn draw(
        &self,
        terminal: &mut Terminal,
        point: Point, // TODO: CellPoint
        highlight: bool,
    ) {
        /// Every 5 cells, the color changes to make the grid and its cells easier to look at and distinguish.
        const SEPARATION_POINT: u16 = 5;

        fn draw(
            terminal: &mut Terminal,
            foreground_color: Option<Color>,
            background_color: Color,
            content: Cow<'static, str>,
        ) {
            if let Some(foreground_color) = foreground_color {
                terminal.set_foreground_color(foreground_color);
            }
            terminal.set_background_color(background_color);
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
