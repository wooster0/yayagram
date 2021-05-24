use crate::{
    editor::Editor,
    grid::{
        builder::{Builder, Cursor},
        Cell, Grid,
    },
    undo_redo_buffer, TEXT_LINE_COUNT,
};
use std::{
    thread,
    time::{Duration, Instant},
};
use terminal::{
    event::{Event, KeyEvent, MouseButton, MouseEvent, MouseEventKind},
    util::Point,
    Terminal,
};

fn draw_dark_cell_color(terminal: &mut Terminal, mut cursor_point: Point, grid: &Grid, cell: Cell) {
    let center_x = Cursor::centered(terminal, grid).point.x;
    if (cursor_point.x - center_x) % 2 != 0 {
        cursor_point.x -= 1;
    }
    terminal.set_cursor(cursor_point);

    terminal.set_background_color(cell.get_dark_color());
    terminal.write("  ");
    terminal.reset_colors();
}

fn get_cell_point_by_cursor_point(cursor_point: Point, builder: &Builder) -> Point {
    Point {
        x: (cursor_point.x - builder.cursor.point.x) / 2,
        y: cursor_point.y - builder.cursor.point.y,
    }
}

/// Handles the event and returns a `bool` determing whether execution should be aborted.
pub fn handle_mouse(
    terminal: &mut Terminal,
    event: MouseEvent,
    builder: &mut Builder,
    plot_mode: &mut Option<Cell>,
    editor_toggled: bool,
    starting_time: &mut Option<Instant>,
) -> State {
    match event {
        MouseEvent {
            kind: MouseEventKind::Drag(mouse_button),
            point,
        }
        | MouseEvent {
            kind: MouseEventKind::Press(mouse_button),
            point,
        } => {
            if builder.contains(point) {
                let cell_point = get_cell_point_by_cursor_point(point, builder);
                let cell = builder.grid.get_mut_cell(cell_point.x, cell_point.y);
                if let Some(plot_mode) = *plot_mode {
                    if *cell == plot_mode {
                        return State::Continue;
                    }
                    *cell = plot_mode;
                } else {
                    let mut new_plot_mode = match mouse_button {
                        MouseButton::Left => Cell::Filled,
                        MouseButton::Middle => Cell::Maybed,
                        MouseButton::Right => Cell::Crossed,
                    };
                    if *cell == new_plot_mode {
                        new_plot_mode = Cell::Empty;
                    }
                    *plot_mode = Some(new_plot_mode);
                    *cell = new_plot_mode;
                }
                let cell = *cell;

                builder
                    .grid
                    .undo_redo_buffer
                    .push(undo_redo_buffer::Operation::SetCell {
                        point: cell_point,
                        cell,
                    });

                if editor_toggled {
                    rebuild_clues(terminal, builder, cell_point);

                    // The solved screen shouldn't be triggered within the editor
                    let _all_clues_solved = builder.draw(terminal);
                } else {
                    let all_clues_solved = builder.draw(terminal);

                    let starting_time = starting_time.get_or_insert(Instant::now());

                    if all_clues_solved {
                        return State::Solved(starting_time.elapsed());
                    }

                    // Overdraw it with a dark color because we know that the cell is hovered
                    draw_dark_cell_color(terminal, point, &builder.grid, cell);
                }

                terminal.flush();
            } else {
                // `plot_mode` won't be reset
            }
        }
        MouseEvent {
            kind: MouseEventKind::Move,
            point,
        } => {
            // No grid mutation happened
            let _all_clues_solved = builder.draw(terminal);

            if builder.contains(point) {
                let cell_point = get_cell_point_by_cursor_point(point, builder);
                let cell = builder.grid.get_cell(cell_point.x, cell_point.y);
                draw_dark_cell_color(terminal, point, &builder.grid, cell);
            }
        }
        _ => {
            *plot_mode = None;
        }
    }

    State::Continue
}

fn rebuild_clues(terminal: &mut Terminal, builder: &mut Builder, cell_point: Point) {
    builder.clear_clues(terminal);
    builder.grid.horizontal_clues_solutions[cell_point.y as usize] =
        builder.grid.get_horizontal_clues(cell_point.y).collect();
    builder.grid.vertical_clues_solutions[cell_point.x as usize] =
        builder.grid.get_vertical_clues(cell_point.x).collect();
}

/// Handles the event and returns a `State`.
pub fn handle(
    terminal: &mut Terminal,
    event: Event,
    builder: &mut Builder,
    plot_mode: &mut Option<Cell>,
    editor: &mut Editor,
    last_notification: Option<&'static str>,
    starting_time: &mut Option<Instant>,
) -> State {
    match event {
        Event::Mouse(mouse_event) => handle_mouse(
            terminal,
            mouse_event,
            builder,
            plot_mode,
            editor.toggled,
            starting_time,
        ),
        Event::Key(key_event) => handle_key(terminal, key_event, builder, editor),
        Event::Resize => handle_window_resize(terminal, builder, last_notification),
    }
}

/// This handles all key input for actions like undo, redo, reset and so on.
pub fn handle_key(
    terminal: &mut Terminal,
    key_event: KeyEvent,
    builder: &mut Builder,
    editor: &mut Editor,
) -> State {
    match key_event {
        KeyEvent::Char('r', None) | KeyEvent::Char('R', None) => {
            builder.grid.cells.fill(Cell::Empty);
            builder
                .grid
                .undo_redo_buffer
                .push(undo_redo_buffer::Operation::Clear);

            // It would've already been solved from the start
            let _all_clues_solved = builder.draw(terminal);

            State::Continue
        }
        KeyEvent::Char('q', None) | KeyEvent::Char('Q', None) | KeyEvent::Left(None) => {
            if builder.grid.undo_last_cell() {
                // It would've already been solved before
                let _all_clues_solved = builder.draw(terminal);
            }

            State::Continue
        }
        KeyEvent::Char('e', None) | KeyEvent::Char('E', None) | KeyEvent::Right(None) => {
            if builder.grid.redo_last_cell() {
                // It would've already been solved before
                let _all_clues_solved = builder.draw(terminal);
            }

            State::Continue
        }
        KeyEvent::Tab => {
            editor.toggle();

            if editor.toggled {
                // TODO: maybe this should be shown all the time (make it part of window title?)
                State::Alert("Editor enabled")
            } else {
                State::Alert("Editor disabled")
            }
        }
        KeyEvent::Char('s', None) | KeyEvent::Char('S', None) | KeyEvent::Enter
            if editor.toggled =>
        {
            if let Err(err) = editor.save_grid(&builder) {
                State::Alert(err)
            } else {
                set_title(
                    terminal,
                    &format!("yayagram - Grid saved as {}", editor.filename),
                );
                State::Continue
            }
        }
        KeyEvent::Esc => State::Exit,
        _ => State::Continue,
    }
}

fn set_title(terminal: &mut Terminal, title: &str) {
    fn reset_title() {
        thread::spawn(|| {
            // NOTE: to be able to reuse the current `terminal`, it'd probably have to be in `RwLock` or `Mutex`?
            thread::sleep(Duration::from_secs(3));
            let mut terminal = Terminal::new().unwrap();
            terminal.set_title("yayagram");
        });
    }

    terminal.set_title(title);
    reset_title();
}

pub fn await_fitting_window_size(terminal: &mut Terminal, grid: &Grid) -> State {
    fn terminal_height_is_within_grid_height(grid: &Grid, terminal: &Terminal) -> bool {
        terminal.size.height >= grid.size.height + grid.max_clues_size.height + TEXT_LINE_COUNT * 2
    }

    fn terminal_width_is_within_grid_width(grid: &Grid, terminal: &Terminal) -> bool {
        terminal.size.width >= grid.size.width + grid.max_clues_size.width
    }

    let mut state = State::Continue;

    match (
        terminal_width_is_within_grid_width(&grid, terminal),
        terminal_height_is_within_grid_height(&grid, terminal),
    ) {
        (true, true) => state,
        (within_width, within_height) => {
            terminal.set_cursor(Point::default());
            if !within_width {
                terminal.write("Please increase terminal width");
            } else if !within_height {
                terminal.write("Please increase terminal height");
            } else {
                unreachable!();
            }
            terminal.flush();
            loop {
                match (
                    terminal_width_is_within_grid_width(&grid, terminal),
                    terminal_height_is_within_grid_height(&grid, terminal),
                ) {
                    (true, true) => break state,
                    _ => {
                        state = await_window_resize(terminal);
                        if let State::Exit = state {
                            return state;
                        }
                    }
                }
            }
        }
    }
}

fn handle_window_resize(
    terminal: &mut Terminal,
    builder: &mut Builder,
    last_notification: Option<&'static str>,
) -> State {
    let state = await_fitting_window_size(terminal, &builder.grid);

    builder.cursor = Cursor::centered(terminal, &builder.grid);

    terminal.clear();

    // No grid mutation happened
    let _all_clues_solved = builder.draw(terminal);

    crate::draw_help(terminal, &builder);
    if let Some(notification) = last_notification {
        draw_notification(terminal, builder, &notification);
    }

    state
}

pub fn await_key(terminal: &mut Terminal) {
    loop {
        let event = terminal.read_event();
        if let Some(Event::Key(_)) = event {
            break;
        }
    }
}

pub fn await_window_resize(terminal: &mut Terminal) -> State {
    loop {
        let event = terminal.read_event();
        match event {
            Some(Event::Key(KeyEvent::Esc)) => break State::Exit,
            Some(Event::Key(_)) => break State::Continue,
            Some(Event::Resize) => break State::Continue,
            _ => {}
        }
    }
}

#[must_use]
pub enum State {
    Continue,
    Solved(Duration),
    Alert(&'static str),
    Exit,
}

pub fn r#loop(terminal: &mut Terminal, builder: &mut Builder) -> State {
    let mut plot_mode = None;
    let mut editor = Editor::default();

    let mut notification: Option<&'static str> = None;
    let mut notification_clear_delay = 0_usize;

    let mut starting_time: Option<Instant> = None;

    while let Some(event) = terminal.read_event() {
        // The order of statements in this loop matters

        if notification_clear_delay != 0 {
            notification_clear_delay -= 1;
            if notification_clear_delay == 0 {
                if let Some(notification_to_clear) = notification {
                    clear_notification(terminal, builder, notification_to_clear.len());
                    notification = None;
                }
            }
        }

        let state = handle(
            terminal,
            event,
            builder,
            &mut plot_mode,
            &mut editor,
            notification,
            &mut starting_time,
        );

        #[cfg(debug_assertions)]
        {
            crate::grid::debug::display(terminal, builder);
        }

        terminal.flush();

        match state {
            State::Continue => continue,
            State::Solved(_) | State::Exit => return state,
            State::Alert(new_notification) => {
                // Draw a new notification. Notifications are cleared after some time.

                if let Some(previous_notification) = notification {
                    clear_notification(terminal, builder, previous_notification.len());
                }
                draw_notification(terminal, builder, new_notification);
                notification = Some(new_notification);
                notification_clear_delay = 75;
                terminal.flush();
            }
        }
    }

    unreachable!();
}

const fn get_notification_y(builder: &Builder) -> u16 {
    builder.cursor.point.y - builder.grid.max_clues_size.height - 1
}

/// Clears the previous notification.
fn clear_notification(terminal: &mut Terminal, builder: &Builder, notification_len: usize) {
    crate::draw_text(
        terminal,
        &builder,
        &" ".repeat(notification_len),
        get_notification_y(&builder),
    );
}

/// Draws a notification above the grid.
fn draw_notification(terminal: &mut Terminal, builder: &Builder, notification: &'static str) {
    crate::draw_text(
        terminal,
        &builder,
        notification,
        get_notification_y(&builder),
    );
}
