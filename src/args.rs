//! Parses the arguments to the program, if present.

use crate::util;
use std::{
    borrow::Cow,
    env, fs,
    io::{self, Write},
};
use terminal::util::Size;

/// The values that can be created out of the arguments.
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
    OutOfRange(&'static str),
    Other(&'static str),
}

fn parse_squared_size(size_str: &str) -> Result<Option<Arg>, SizeError> {
    if let Ok(parsed_size) = size_str.parse::<u16>() {
        match parsed_size {
            1..=100 => Ok(Some(Arg::GridSize(Size {
                width: parsed_size,
                height: parsed_size,
            }))),
            _ => Err(SizeError::OutOfRange("size")),
        }
    } else if util::is_numeric(size_str) {
        // A value >u16::MAX will not parse but might still be a number
        Err(SizeError::OutOfRange("size"))
    } else {
        Err(SizeError::Other("file not found"))
    }
}

fn parse_size(width_str: &str, height_str: &str) -> Result<Option<Arg>, SizeError> {
    if let Ok(parsed_width) = width_str.parse::<u16>() {
        if let Ok(parsed_height) = height_str.parse::<u16>() {
            if !(1..=100).contains(&parsed_width) {
                return Err(SizeError::OutOfRange("width"));
            }
            if !(1..=100).contains(&parsed_height) {
                return Err(SizeError::OutOfRange("height"));
            }
            return Ok(Some(Arg::GridSize(Size {
                width: parsed_width,
                height: parsed_height,
            })));
        } else if util::is_numeric(height_str) {
            // A value >u16::MAX will not parse but might still be a number
            return Err(SizeError::OutOfRange("height"));
        }
    } else if util::is_numeric(width_str) {
        // A value >u16::MAX will not parse but might still be a number
        return Err(SizeError::OutOfRange("width"));
    }

    Err(SizeError::Other("file not found"))
}

fn get_writer(file: fs::File, content: &str) -> Result<io::BufWriter<fs::File>, &'static str> {
    let mut writer = io::BufWriter::new(file);

    // To make cheating a little bit harder, leave the file empty while the game is running
    // so that the user can't see the solution by looking at the file

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

fn parse_strings(
    first_string: String,
    second_string: Option<String>,
) -> Result<Option<Arg>, Cow<'static, str>> {
    // Check for a file first so that filenames consisting of numbers can be accepted too
    let mut open_options = fs::OpenOptions::new();
    open_options.read(true).write(true);

    match open_options.open(&first_string) {
        Ok(mut file) => {
            fn valid_extension(str: &str) -> bool {
                let path = std::path::Path::new(str);
                if let Some(extension) = path.extension() {
                    extension == "yaya"
                } else {
                    false
                }
            }

            if !valid_extension(&first_string) {
                return Err("filename extension must be \"yaya\"".into());
            }

            let content = util::read_file_content(&mut file).map_err(|_| "file reading error")?;

            match get_writer(file, &content) {
                Ok(writer) => Ok(Some(Arg::File {
                    writer,
                    name: first_string,
                    content,
                })),
                Err(err) => Err(err.into()),
            }
        }
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => {
                if first_string == "--help" || first_string == "-h" {
                    Ok(Some(Arg::Help))
                } else {
                    let result = if let Some(second_string) = second_string {
                        parse_size(&first_string, &second_string)
                    } else {
                        parse_squared_size(&first_string)
                    };

                    match result {
                        Ok(size) => Ok(size),
                        Err(SizeError::OutOfRange(thing)) => {
                            Err(format!("grid {} must be in range 1 to 100", thing).into())
                        }
                        Err(SizeError::Other(message)) => Err(message.into()),
                    }
                }
            }
            _ => Err("file opening error".into()),
        },
    }
}

pub fn parse() -> Result<Option<Arg>, Cow<'static, str>> {
    // See https://github.com/rust-lang/rust/pull/84551#discussion_r620728070
    // on why it's better to use `env::args_os` than `env::args`.
    let mut args = env::args_os();

    args.next(); // This is usually the program name

    if let Some(arg) = args.next() {
        if let Ok(first_string) = arg.into_string() {
            if let Some(arg) = args.next() {
                if let Ok(second_string) = arg.into_string() {
                    parse_strings(first_string, Some(second_string))
                } else {
                    Err("second argument is not valid UTF-8".into())
                }
            } else {
                parse_strings(first_string, None)
            }
        } else {
            Err("first argument is not valid UTF-8".into())
        }
    } else {
        Ok(None)
    }
}
