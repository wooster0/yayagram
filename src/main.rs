mod args;
mod editor;
mod event;
mod grid;
mod undo_redo_buffer;
mod util;

use event::State;
use grid::{builder::Builder, Grid};
use std::{borrow::Cow, io, process, time::Duration};
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

    let stdout = io::stdout();
    match get_terminal(stdout.lock()) {
        Ok(mut terminal) => {
            if let State::Continue = event::input::await_fitting_window_size(&mut terminal, &grid) {
                let mut builder = Builder::new(&terminal, grid);

                let all_clues_solved = builder.draw_all(&mut terminal);
                draw_basic_controls_help(&mut terminal, &builder);

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

pub const BASIC_CONTROLS_HELP: &[&str] = &["A: Undo, D: Redo, C: Clear", "X: Measure, F: Fill"];

fn draw_basic_controls_help(terminal: &mut Terminal, builder: &Builder) {
    terminal.set_foreground_color(Color::DarkGray);
    for (index, text) in BASIC_CONTROLS_HELP.iter().enumerate() {
        draw_bottom_text(terminal, &builder, text, index as u16);
    }
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
                    size
                } else {
                    Size::new(5, 5)
                };
                Ok(Grid::random(grid_size))
            }
        },
        Err(err) => Err(err.clone()),
    }
}

/// Creates a new initialized `Terminal` instance if possible and sets the window title.
///
/// This `Terminal` is what allows manipulating the terminal in all kinds of ways such as setting colors, writing data, moving the cursor etc.
fn get_terminal(stdout: io::StdoutLock) -> Result<Terminal, &'static str> {
    if let Ok(mut terminal) = Terminal::new(stdout) {
        terminal.initialize();
        terminal.set_title("yayagram");
        Ok(terminal)
    } else {
        Err("This is not a terminal")
    }
}

pub const fn get_picture_height(grid: &Grid) -> u16 {
    let mut picture_height = grid.size.height / 2;
    if grid.size.height % 2 == 1 {
        picture_height += 1;
    }
    picture_height
}

/// Draws centered text on the top.
pub fn draw_top_text(terminal: &mut Terminal, builder: &Builder, text: &str, y_alignment: u16) {
    let picture_height = get_picture_height(&builder.grid);
    let y = builder.point.y - picture_height - 1;

    terminal.set_cursor(Point {
        x: builder.point.x + builder.grid.size.width - text.len() as u16 / 2,
        y: y - y_alignment,
    });
    terminal.write(text);
}

/// Draws centered text on the bottom.
pub fn draw_bottom_text(terminal: &mut Terminal, builder: &Builder, text: &str, y_alignment: u16) {
    let mut y = builder.point.y + builder.grid.size.height;
    y += 1; // Make way for the progress bar

    terminal.set_cursor(Point {
        x: builder.point.x + builder.grid.size.width - text.len() as u16 / 2,
        y: y + y_alignment,
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
    terminal.reset_colors();
    draw_top_text(terminal, &builder, "Press any key to continue", 0);

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
    draw_top_text(terminal, &builder, &text, 1);
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
