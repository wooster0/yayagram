use crate::grid::builder::Builder;
use terminal::Terminal;

/// Gets the Y-axis of the alert which is above the grid.
const fn get_y(builder: &Builder) -> u16 {
    builder.point.y - builder.grid.max_clues_size.height - 1
}

/// Clears the previous alert.
pub fn clear(terminal: &mut Terminal, builder: &Builder, alert_len: usize) {
    crate::draw_text(terminal, &builder, &" ".repeat(alert_len), get_y(&builder));
}

/// Draws a alert above the grid.
pub fn draw(terminal: &mut Terminal, builder: &Builder, alert: &'static str) {
    crate::draw_text(terminal, &builder, alert, get_y(&builder));
}
