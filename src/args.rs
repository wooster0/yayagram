//! Parses the argument to the program, if present.
//!
//! The first and only argument to the program should be either:
//!
//! * A filename pointing to a file containing a valid grid.
//! * A grid size in range `1..=100`.

use crate::util;
use std::{
    env, fs,
    io::{self, Write},
};
use terminal::util::Size;

/// The values that can be created out of the argument.
pub enum Arg {
    File {
        writer: io::BufWriter<fs::File>,
        name: String,
        content: String,
    },
    GridSize(Size),
    Help,
}

enum SizeError {
    OutOfRange,
    Other(&'static str),
}

fn parse_size(str: &str) -> Result<Option<Arg>, SizeError> {
    if let Ok(parsed_size) = str.parse::<u16>() {
        match parsed_size {
            1..=100 => Ok(Some(Arg::GridSize(Size {
                width: parsed_size,
                height: parsed_size,
            }))),
            _ => Err(SizeError::OutOfRange),
        }
    } else if util::is_numeric(str) {
        // A value >u16::MAX will not parse but might still be a number
        Err(SizeError::OutOfRange)
    } else {
        Err(SizeError::Other("file not found"))
    }
}

fn get_writer(file: fs::File, content: &str) -> Result<io::BufWriter<fs::File>, &'static str> {
    let mut writer = io::BufWriter::new(file);

    // To make cheating a little bit harder, leave the file empty while the game is running
    // so that the user can't cheat by looking at the file

    // This will happen immediately
    util::clear_file(&mut writer)?;

    // But this will not.
    // The content will only be written back once the writer is flushed which will happen when it is dropped.
    // It's to be dropped at the end of the program. This is handled in `main`.
    writer
        .write_all(content.as_bytes())
        .map_err(|_| "file writing failed")?;

    Ok(writer)
}

fn parse_string(string: String) -> Result<Option<Arg>, &'static str> {
    // Check for a file first so that filenames consisting of numbers can be accepted too
    let mut open_options = fs::OpenOptions::new();
    open_options.read(true).write(true);

    match open_options.open(&string) {
        Ok(mut file) => {
            fn valid_extension(str: &str) -> bool {
                let path = std::path::Path::new(str);
                if let Some(extension) = path.extension() {
                    extension == "yaya"
                } else {
                    false
                }
            }

            if !valid_extension(&string) {
                return Err("filename extension must be \"yaya\"");
            }

            let content = util::read_file_content(&mut file).map_err(|_| "file reading error")?;

            match get_writer(file, &content) {
                Ok(writer) => Ok(Some(Arg::File {
                    writer,
                    name: string,
                    content,
                })),
                Err(err) => Err(err),
            }
        }
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => {
                if string == "--help" || string == "-h" {
                    return Ok(Some(Arg::Help));
                } else {
                    match parse_size(&string) {
                        Ok(size) => Ok(size),
                        Err(SizeError::OutOfRange) => Err("grid size must be in range 1 to 100"),
                        Err(SizeError::Other(message)) => Err(message),
                    }
                }
            }
            _ => Err("file opening error"),
        },
    }
}

pub fn parse() -> Result<Option<Arg>, &'static str> {
    // See https://github.com/rust-lang/rust/pull/84551#discussion_r620728070
    // on why it's better to use `env::args_os` than `env::args`.
    let mut args = env::args_os();

    args.next(); // This is usually the program name

    if let Some(arg) = args.next() {
        if let Ok(string) = arg.into_string() {
            parse_string(string)
        } else {
            Err("argument is not valid UTF-8")
        }
    } else {
        Ok(None)
    }
}
