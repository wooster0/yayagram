use super::super::alert;
use super::{Alert, State};
use crate::{
    args::{valid_extension, FILE_EXTENSION},
    grid::{self, builder::Builder, Grid},
};
use std::time::Instant;
use terminal::{
    event::{Event, Key},
    util::Point,
    Terminal,
};

pub fn handle_resize(
    terminal: &mut Terminal,
    builder: &mut Builder,
    alert: &Option<Alert>,
    starting_time: Option<Instant>,
) -> State {
    terminal.clear();

    let state = await_fitting_size(terminal, &builder.grid, starting_time);

    builder.point = grid::builder::centered_point(terminal, &builder.grid);

    // The grid wasn't mutated
    #[allow(unused_must_use)]
    {
        builder.draw_all(terminal);
    }

    crate::draw_basic_controls_help(terminal, builder);
    if let Some(alert) = alert {
        alert.draw(terminal, builder);
    }

    state
}

pub fn await_fitting_size(
    terminal: &mut Terminal,
    grid: &Grid,
    starting_time: Option<Instant>,
) -> State {
    const fn terminal_width_is_within_grid_width(grid: &Grid, terminal: &Terminal) -> bool {
        terminal.size.width >= grid.size.width * 2 + grid.max_clues_size.width
    }

    fn terminal_height_is_within_grid_height(grid: &Grid, terminal: &Terminal) -> bool {
        terminal.size.height > crate::total_height(grid)
    }

    let mut state = State::Continue;

    match (
        terminal_width_is_within_grid_width(grid, terminal),
        terminal_height_is_within_grid_height(grid, terminal),
    ) {
        (true, true) => state,
        (within_width, within_height) => {
            terminal.set_cursor(Point::default());
            let length = if !within_width {
                "width"
            } else if !within_height {
                "height"
            } else {
                unreachable!()
            };
            let message = format!(
                "Please increase window {} or decrease text size (Ctrl and -)",
                length
            );
            terminal.write(&message);
            terminal.flush();

            let state = loop {
                match (
                    terminal_width_is_within_grid_width(grid, terminal),
                    terminal_height_is_within_grid_height(grid, terminal),
                ) {
                    (true, true) => break state,
                    _ => {
                        state = await_resize(terminal, starting_time);
                        if let State::Exit(_) = state {
                            break state;
                        }
                    }
                }
            };

            terminal.set_cursor(Point::default());
            for _ in 0..message.len() {
                terminal.write(" ");
            }

            state
        }
    }
}

fn await_resize(terminal: &mut Terminal, starting_time: Option<Instant>) -> State {
    loop {
        let event = terminal.read_event();
        match event {
            Some(Event::Key(Key::Esc)) => break State::Exit(starting_time),
            Some(Event::Key(_)) => break State::Continue,
            Some(Event::Resize) => break State::Continue,
            _ => {}
        }
    }
}

/// Awaits a grid file path dropped onto the window.
///
/// As opposed to [`confirmation_prompt`], this does not disable mouse capturing to change the pointer icon because
/// the user is, differently from the prompt, supposed to do something with their mouse.
pub fn await_dropped_grid_file_path(
    terminal: &mut Terminal,
    builder: &Builder,
    alert: &mut Option<Alert>,
) -> Result<String, &'static str> {
    let message = format!(
        "Drag & drop a `.{}` grid file onto this window to load. Esc to cancel",
        FILE_EXTENSION
    )
    .into();
    alert::draw(terminal, builder, alert, message);

    terminal.flush();

    let mut path = String::new();

    while !valid_extension(&path) {
        let input = terminal.read_event();

        match input {
            Some(Event::Key(Key::Char(char))) => {
                if path.is_empty() && char == '\'' || char == '"' {
                    // In some terminals the path starts and ends with an apostrophe or a double quote.
                    // We simply ignore the first apostrophe or double quote, if there is one.
                    // `valid_extension` will recognize the path before we push the last character,
                    // meaning we don't need to care about the final apostrophe or double quote.
                } else {
                    path.push(char);
                }
            }
            Some(Event::Key(Key::Esc)) => {
                return Err("Canceled");
            }
            Some(Event::Resize | Event::Mouse(_)) => {}
            _ => {
                return Err("Invalid input. Canceled");
            }
        }
    }

    Ok(path)
}

/// Draws an alert asking the user to confirm the given thing and returns whether the user confirmed the action.
///
/// Despite the alert saying that Esc cancels, every other key apart from Enter will cancel as well.
///
/// NOTE: mouse capturing is disabled for the duration of the prompt and a flush is required after this call to reenable it.
pub fn confirmation_prompt(
    terminal: &mut Terminal,
    builder: &mut Builder,
    alert: &mut Option<Alert>,
    thing_to_confirm: &str,
) -> bool {
    let message = format!("Press Enter to confirm {}. Esc to cancel", thing_to_confirm).into();
    alert::draw(terminal, builder, alert, message);

    // We could also just ignore `Event::Mouse(_)` in the loop below but disabling mouse capture changes the pointer icon
    // in some terminals, which helps inform the user that the game is halted and that they can't use their mouse and need to respond to the confirmation prompt.
    terminal.disable_mouse_capture();

    terminal.flush();

    let confirmed = loop {
        let input = terminal.read_event();

        match input {
            Some(Event::Key(Key::Enter)) => break true,
            Some(Event::Resize) => {}
            _ => break false,
        }
    };

    terminal.enable_mouse_capture();

    confirmed
}
