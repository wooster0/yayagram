use crate::grid::builder::Builder;
use std::borrow::Cow;
use terminal::Terminal;

pub struct Alert {
    pub message: Cow<'static, str>,
    pub clear_delay: usize,
}

impl Alert {
    pub fn new(message: Cow<'static, str>) -> Self {
        Self {
            message,
            clear_delay: 75,
        }
    }

    /// Clears the previous alert.
    pub fn clear(&mut self, terminal: &mut Terminal, builder: &mut Builder) {
        crate::set_cursor_for_top_text(terminal, &builder, self.message.len(), 0);
        for _ in 0..self.message.len() {
            terminal.write(" ");
        }
    }

    /// Draws an alert above the grid.
    pub fn draw(&self, terminal: &mut Terminal, builder: &Builder) {
        crate::set_cursor_for_top_text(terminal, &builder, self.message.len(), 0);
        terminal.write(&self.message);
    }
}
