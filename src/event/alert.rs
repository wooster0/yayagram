use crate::grid::builder::Builder;
use std::borrow::Cow;
use terminal::Terminal;

const CLEAR_DELAY: usize = 75;

pub struct Alert {
    pub message: Cow<'static, str>,
    pub clear_delay: usize,
}

impl Alert {
    pub fn new(message: Cow<'static, str>) -> Self {
        Self {
            message,
            clear_delay: CLEAR_DELAY,
        }
    }

    /// Clears the previous alert.
    pub fn clear(&mut self, terminal: &mut Terminal, builder: &Builder) {
        crate::set_cursor_for_top_text(terminal, builder, self.message.len(), 0, None);
        for _ in 0..self.message.len() {
            terminal.write(" ");
        }
    }

    /// Draws an alert above the grid.
    pub fn draw(&self, terminal: &mut Terminal, builder: &Builder) {
        crate::set_cursor_for_top_text(terminal, builder, self.message.len(), 0, None);
        terminal.write(&self.message);
    }

    pub fn reset_clear_delay(&mut self) {
        self.clear_delay = CLEAR_DELAY;
    }
}

pub fn draw(
    terminal: &mut Terminal,
    builder: &Builder,
    alert: &mut Option<Alert>,
    message: Cow<'static, str>,
) {
    // In some cases we might have colors so we always safely reset them beforehand
    terminal.reset_colors();

    if let Some(ref mut current_alert) = alert {
        current_alert.clear(terminal, builder);

        current_alert.message = message;
        current_alert.reset_clear_delay();

        current_alert.draw(terminal, builder);
    } else {
        let new_alert = Alert::new(message);
        new_alert.draw(terminal, builder);
        *alert = Some(new_alert);
    }
}

pub fn handle_clear_delay(terminal: &mut Terminal, builder: &Builder, alert: &mut Option<Alert>) {
    if let Some(ref mut alert_to_clear) = alert {
        if alert_to_clear.clear_delay == 0 {
            alert_to_clear.clear(terminal, builder);
            *alert = None;
        } else {
            alert_to_clear.clear_delay -= 1;
        }
    }
}
