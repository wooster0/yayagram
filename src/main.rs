mod args;
mod editor;
mod event;
mod grid;
mod undo_redo_buffer;
mod util;

use event::State;
use grid::{builder::Builder, Grid};
use std::{borrow::Cow, io::Write, process, sync, time::Duration};
use terminal::{
    util::{Color, Point, Size},
    Terminal,
};

// Things that could be implemented but might not be worth it:
// -A main menu
// -An interactive tutorial
// -Currently whole clue rows are grayed out once all cells for those clues have been solved.
//  Make them gray out individually. (Maybe itertools' `pad_using` is helpful)
// -Make the builder be able to build non-squared grids. Currently it doesn't work when width != height.

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

fn run() -> Result<(), Cow<'static, str>> {
    let arg = args::parse();

    let grid = match get_grid(&arg) {
        Ok(grid) => grid,
        Err(err) => {
            drop(arg);
            return Err(err);
        }
    };

    // It's important to flush the writer only right before program end
    // because it has content written to it that should only be flushed at end.
    // See also: `crate::args::get_writer`
    let writer_arc = if let Ok(Some(args::Arg::File {
        writer,
        name: _,
        content: _,
    })) = arg
    {
        let writer_arc = sync::Arc::new(sync::Mutex::new(writer));

        let ctrlc_writer = writer_arc.clone();
        ctrlc::set_handler(move || {
            ctrlc_writer.lock().unwrap().flush().unwrap();
        })
        .unwrap();

        Some(writer_arc)
    } else {
        None
    };

    match get_terminal() {
        Ok(mut terminal) => {
            if let State::Continue = event::await_fitting_window_size(&mut terminal, &grid) {
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

    if let Some(writer) = writer_arc {
        writer.lock().unwrap().flush().unwrap();
    }

    Ok(())
}

fn draw_help(terminal: &mut Terminal, builder: &Builder) {
    terminal.set_foreground_color(Color::DarkGray);
    draw_text(
        terminal,
        &builder,
        "Q: Undo, E: Redo, R: Reset",
        builder.cursor.point.y + builder.grid.size.height,
    );
    terminal.reset_colors();
}

fn get_grid(arg: &Result<Option<args::Arg>, &'static str>) -> Result<Grid, Cow<'static, str>> {
    match arg {
        Ok(arg) => match arg {
            Some(args::Arg::File {
                writer: _,
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
                    Size::new(10, 10)
                };
                Ok(Grid::random(grid_size))
            }
        },
        Err(err) => Err((*err).into()),
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

/// One hour in seconds.
const HOUR: u64 = 60 * 60;

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
            let elapsed_seconds = total_elapsed_seconds % 60;
            let elapsed_minutes = total_elapsed_seconds / 60;
            let elapsed_hours = elapsed_minutes / 60;
            format!(
                "Solved in {:02}:{:02}:{:02}",
                elapsed_hours, elapsed_minutes, elapsed_seconds
            )
            .into()
        }
    };
    terminal.set_foreground_color(Color::White);
    draw_text(terminal, &builder, &text, y - 1);
    terminal.reset_colors();

    terminal.flush();

    event::await_key(terminal);
}
