use super::{super::alert, Alert, State};
use crate::grid::{self, builder::Builder, Cell, CellPlacement, Grid};
use terminal::{
    event::{Event, Key, MouseButton, MouseEvent, MouseEventKind},
    util::{Point, Size},
    Terminal,
};

/// This handles all mouse input.
pub fn handle_event(
    terminal: &mut Terminal,
    event: MouseEvent,
    builder: &mut Builder,
    editor_toggled: bool,
    cell_placement: &mut CellPlacement,
    alert: &mut Option<Alert>,
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
                let grid_corner = Point {
                    x: builder.point.x + builder.grid.size.width * 2,
                    y: builder.point.y + builder.grid.size.height,
                };
                let resize_icon = Point {
                    x: grid_corner.x + 1,
                    ..grid_corner
                };

                if selected_cell_point == resize_icon {
                    resize_grid(terminal, builder, alert, resize_icon)
                } else {
                    State::Continue
                }
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

fn resize_grid(
    terminal: &mut Terminal,
    builder: &mut Builder,
    alert: &mut Option<Alert>,
    resize_icon: Point,
) -> State {
    let original_grid_size = builder.grid.size.clone();

    crate::clear_basic_controls_help(terminal, builder);

    builder.clear_progress_bar_and_resize_icon(terminal);

    loop {
        let event = terminal.read_event();

        match event {
            Some(Event::Mouse(MouseEvent {
                kind: MouseEventKind::Drag(_),
                point,
            })) => {
                fn draw(terminal: &mut Terminal, builder: &mut Builder) {
                    builder.draw_empty_grid(terminal);
                    terminal.reset_colors();
                    terminal.flush();
                }

                use std::cmp::Ordering;

                match point.x.cmp(&resize_icon.x) {
                    Ordering::Greater => {
                        builder.clear_empty_grid(terminal);

                        builder.grid.size.width =
                            original_grid_size.width + (point.x - resize_icon.x) / 2;

                        draw(terminal, builder);
                    }
                    Ordering::Less => {
                        builder.clear_empty_grid(terminal);

                        builder.grid.size.width = original_grid_size
                            .width
                            .saturating_sub(resize_icon.x.saturating_sub(point.x) / 2);

                        if builder.grid.size.width < 1 {
                            builder.grid.size.width = 1;
                        }

                        draw(terminal, builder);
                    }
                    Ordering::Equal => {}
                }

                match point.y.cmp(&resize_icon.y) {
                    Ordering::Greater |
                    Ordering::Equal // This prevents some weird behavior on expansion or contraction of the grid over the original grid size
                    => {
                        builder.clear_empty_grid(terminal);

                        builder.grid.size.height =
                            original_grid_size.height + (point.y - resize_icon.y);

                        draw(terminal, builder);
                    }
                    Ordering::Less => {
                        builder.clear_empty_grid(terminal);

                        builder.grid.size.height = original_grid_size
                            .height
                            .saturating_sub(resize_icon.y.saturating_sub(point.y));

                        if builder.grid.size.height < 1 {
                            builder.grid.size.height = 1;
                        }

                        draw(terminal, builder);
                    }
                }
            }
            Some(Event::Mouse(_)) => break,
            _ => {}
        }
    }

    if original_grid_size == builder.grid.size {
        // The grid wasn't mutated
        #[allow(unused_must_use)]
        {
            builder.draw_all(terminal);
        }

        crate::draw_basic_controls_help(terminal, builder);

        State::Continue
    } else {
        let confirmed = confirmation_prompt(terminal, builder, original_grid_size.clone(), alert);

        if confirmed {
            // Currently the new game simply runs inside of this existing game and the new game creates an entirely new state.
            // At some point we would probably hit a stack overflow if the user keeps resizing the grid within the same session.

            terminal.clear();
            crate::start_game(terminal, Grid::random(builder.grid.size.clone()));

            State::Exit(None)
        } else {
            builder.grid.size = original_grid_size;

            terminal.clear();

            // Only the grid's size was mutated
            #[allow(unused_must_use)]
            {
                builder.draw_all(terminal);
            }

            crate::draw_basic_controls_help(terminal, builder);

            State::Alert("Aborted".into())
        }
    }
}

fn confirmation_prompt(
    terminal: &mut Terminal,
    builder: &mut Builder,
    original_grid_size: Size,
    alert: &mut Option<Alert>,
) -> bool {
    let message = "Press Enter to confirm new random grid in this size. Esc to abort".into();

    // Temporarily set the builder grid size back to the old size to render the alert properly.
    let new_grid_size = builder.grid.size.clone();
    builder.grid.size = original_grid_size;
    alert::draw(terminal, builder, alert, message);
    builder.grid.size = new_grid_size;

    terminal.flush();

    loop {
        let input = terminal.read_event();

        match input {
            Some(Event::Key(Key::Enter)) => break true,
            Some(Event::Resize | Event::Mouse(_)) => {}
            _ => break false,
        }
    }
}
