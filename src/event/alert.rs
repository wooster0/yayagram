use crate::grid::builder::Builder;
use terminal::Terminal;

/// Clears the previous alert.
pub fn clear(terminal: &mut Terminal, builder: &mut Builder, alert_len: usize) {
    crate::draw_top_text(terminal, &builder, &" ".repeat(alert_len), 0);
    // This redraws the picture because it was previously overdrawn by the alert.
    // TODO: this is rather ugly. Perhaps alerts should be shown somewhere else
    builder.draw_picture(terminal);
}

/// Draws an alert above the grid.
pub fn draw(terminal: &mut Terminal, builder: &Builder, alert: &str) {
    crate::draw_top_text(terminal, &builder, alert, 0);
}
