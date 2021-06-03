use crate::grid::builder::Builder;
use terminal::Terminal;

/// Gets the Y-axis of the notifications which is above the grid.
const fn get_y(builder: &Builder) -> u16 {
    builder.cursor.point.y - builder.grid.max_clues_size.height - 1
}

/// Clears the previous notification.
pub fn clear(terminal: &mut Terminal, builder: &Builder, notification_len: usize) {
    crate::draw_text(
        terminal,
        &builder,
        &" ".repeat(notification_len),
        get_y(&builder),
    );
}

/// Draws a notification above the grid.
pub fn draw(terminal: &mut Terminal, builder: &Builder, notification: &'static str) {
    crate::draw_text(terminal, &builder, notification, get_y(&builder));
}
