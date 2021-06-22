pub mod key;
mod mouse;
pub mod window;

use super::{alert::Alert, State};
use crate::{
    editor::Editor,
    grid::{builder::Builder, CellPlacement},
};
use terminal::{event::Event, Terminal};

/// Handles all input.
pub fn handle(
    terminal: &mut Terminal,
    event: Event,
    builder: &mut Builder,
    editor: &mut Editor,
    last_alert: &Option<Alert>,
    cell_placement: &mut CellPlacement,
) -> State {
    match event {
        Event::Mouse(mouse_event) => mouse::handle_event(
            terminal,
            mouse_event,
            builder,
            editor.toggled,
            cell_placement,
        ),
        Event::Key(key_event) => {
            key::handle_event(terminal, key_event, builder, editor, cell_placement)
        }
        Event::Resize => window::handle_resize(terminal, builder, last_alert),
    }
}
