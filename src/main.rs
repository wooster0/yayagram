mod args;
mod editor;
mod event;
mod grid;
mod undo_redo_buffer;
mod util;

use event::State;
use grid::{builder::Builder, Grid};
use std::{borrow::Cow, process, time::Duration};
use terminal::{
    util::{Color, Point, Size},
    Terminal,
};

// Things that could be implemented but might not be worth it:
// -A main menu
// -An interactive tutorial
// -Currently whole clue rows are grayed out once all cells for those clues have been solved
//  Make them gray out individually. (Maybe itertools' `pad_using` is helpful)
// -Ability to change grid size and load grids (with F5?) within the game without the command line
// -Ability to save records to a file and determine new records with that
// -Ability to continue after solving the puzzle/ability to play it again

fn main() {
    let code = match run() {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("{}", err);
            1
        }
    };

    process::exit(code);
}

const HELP: &[&str] = &[
    "Play nonograms/picross in your terminal.",
    "For the arguments please check <https://github.com/r00ster91/yayagram#command-line-arguments>.",
];

fn run() -> Result<(), Cow<'static, str>> {
    let arg = args::parse();

    let grid = match arg {
        Ok(Some(args::Arg::Help)) => {
            for line in HELP {
                println!("{}", line);
            }

            return Ok(());
        }
        Ok(Some(args::Arg::Version)) => {
            let version = env!("CARGO_PKG_VERSION");

            println!("{}", version);

            return Ok(());
        }
        arg => match get_grid(arg) {
            Ok(grid) => grid,
            Err(err) => {
                return Err(err);
            }
        },
    };

    match get_terminal() {
        Ok(mut terminal) => {
            if let State::Continue = event::input::await_fitting_window_size(&mut terminal, &grid) {
                let mut builder = Builder::new(&terminal, grid);

                let all_clues_solved = builder.draw(&mut terminal);
                draw_help(&mut terminal, &builder);

                if all_clues_solved {
                    solved_screen(&mut terminal, &builder, Duration::from_nanos(0), true);
                } else {
                    terminal.flush();

                    let state = event::r#loop(&mut terminal, &mut builder);

                    match state {
                        State::Solved(duration) => {
                            solved_screen(&mut terminal, &builder, duration, false);
                        }
                        State::Exit => {}
                        _ => unreachable!(),
                    }
                }
            }

            terminal.deinitialize();
        }
        Err(err) => {
            return Err(err.into());
        }
    }

    Ok(())
}

fn draw_help(terminal: &mut Terminal, builder: &Builder) {
    terminal.set_foreground_color(Color::DarkGray);
    let mut y = builder.cursor.point.y + builder.grid.size.height;
    draw_text(terminal, &builder, "Q: Undo, E: Redo, R: Reset", y);
    y += 1;
    draw_text(terminal, &builder, "X: Measurement Point", y);
    terminal.reset_colors();
}

fn get_grid(arg: Result<Option<args::Arg>, Cow<'static, str>>) -> Result<Grid, Cow<'static, str>> {
    match arg {
        Ok(arg) => match arg {
            Some(args::Arg::File {
                name: filename,
                content,
            }) => match editor::load_grid(&content) {
                Ok(grid) => Ok(grid),
                Err(err) => {
                    if let Some(line_number) = err.line_number {
                        Err(format!(
                            "invalid grid data in {}:{}: {}",
                            filename, line_number, err.message
                        )
                        .into())
                    } else {
                        Err(format!("invalid grid data in {}: {}", filename, err.message).into())
                    }
                }
            },
            arg => {
                let grid_size = if let Some(args::Arg::GridSize(size)) = arg {
                    size.clone()
                } else {
                    Size::new(5, 5)
                };
                Ok(Grid::random(grid_size))
            }
        },
        Err(err) => Err(err.clone()),
    }
}

/// Creates a new `Terminal` instance if possible and sets the window title.
///
/// This `Terminal` allows us to manipulate the terminal in all kinds of ways like setting colors or moving the cursor.
fn get_terminal() -> Result<Terminal, &'static str> {
    if let Ok(mut terminal) = Terminal::new() {
        terminal.initialize();
        terminal.set_title("yayagram");
        Ok(terminal)
    } else {
        Err("this is not a terminal")
    }
}

/// The amount of text lines drawn above the grid.
const TEXT_LINE_COUNT: u16 = 2;

/// Draws text on the screen where the x-coordinate is centered but y has to be given.
pub fn draw_text(terminal: &mut Terminal, builder: &Builder, text: &str, y: u16) {
    terminal.set_cursor(Point {
        x: builder.cursor.point.x + builder.grid.size.width - text.len() as u16 / 2,
        y,
    });
    terminal.write(text);
}

/// One hour in seconds.
const HOUR: u64 = 60 * 60;

/// The screen that appears when the grid was solved.
fn solved_screen(
    terminal: &mut Terminal,
    builder: &Builder,
    duration: Duration,
    did_nothing: bool,
) {
    let y = builder.cursor.point.y - builder.grid.max_clues_size.height - 1;

    draw_text(terminal, &builder, "Press any key to continue", y);

    let text: Cow<'static, str> = if did_nothing {
        "You won by doing nothing".into()
    } else {
        let total_elapsed_seconds = duration.as_secs();
        if total_elapsed_seconds > HOUR * 99 {
            "That took too long".into()
        } else {
            format!("Solved in {}", format_seconds(total_elapsed_seconds)).into()
        }
    };
    terminal.set_foreground_color(Color::White);
    draw_text(terminal, &builder, &text, y - 1);
    terminal.reset_colors();

    terminal.flush();

    event::input::await_key(terminal);
}

/// Formats the given seconds to an hour, minute and second format.
///
/// # Examples
///
/// ```
/// assert_eq!(format_seconds(60 * 70 + 5), "01:10:05");
/// assert_eq!(format_seconds(45 * 60 + 15), "00:45:15");
/// assert_eq!(format_seconds(60 * 60 * 99), "99:00:00");
/// assert_eq!(format_seconds(60 * 80), "01:20:00");
/// assert_eq!(format_seconds(60 * 60 + 60 * 5 + 30), "01:05:30");
/// ```
fn format_seconds(total_seconds: u64) -> String {
    let seconds = total_seconds % 60;
    let minutes = total_seconds / 60 % 60;
    let hours = total_seconds / HOUR;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
