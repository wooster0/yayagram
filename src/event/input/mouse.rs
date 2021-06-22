use super::State;
use crate::grid::{self, builder::Builder, Cell, CellPlacement};
use terminal::{
    event::{MouseButton, MouseEvent, MouseEventKind},
    Terminal,
};

/// This handles all mouse input.
pub fn handle_event(
    terminal: &mut Terminal,
    event: MouseEvent,
    builder: &mut Builder,
    editor_toggled: bool,
    cell_placement: &mut CellPlacement,
) -> State {
    match event {
        MouseEvent {
            kind: MouseEventKind::Drag(mouse_button) | MouseEventKind::Press(mouse_button),
            point: selected_cell_point,
        } => {
            if builder.contains(selected_cell_point) {
                let cell_to_place = match mouse_button {
                    MouseButton::Left => Cell::Filled,
                    MouseButton::Middle => Cell::Maybed,
                    MouseButton::Right => Cell::Crossed,
                };

                cell_placement.selected_cell_point = Some(selected_cell_point);

                cell_placement.place(
                    terminal,
                    builder,
                    selected_cell_point,
                    cell_to_place,
                    editor_toggled,
                )
            } else {
                State::Continue
            }
        }
        MouseEvent {
            kind: MouseEventKind::Move,
            point,
        } => {
            builder.draw_grid(terminal);

            if builder.contains(point) {
                cell_placement.selected_cell_point = Some(point);
                let some_selected_cell_point = point;

                // We know that this point is hovered
                grid::draw_highlighted_cells(terminal, &builder, some_selected_cell_point);
            }
            State::Continue
        }
        _ => {
            cell_placement.cell = None;
            State::Continue
        }
    }
}
