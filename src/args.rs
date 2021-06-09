//! Parses the arguments to the program, if present.

use crate::util;
use std::{borrow::Cow, env, fs, io};
use terminal::util::Size;

/// The maximum grid size must not have more than 2 digits
/// because such numbers cannot be displayed correctly on the grid
/// due to the grid being based on two characters for numbers.
const MAX_GRID_SIZE: u16 = 99;

/// The values that can be created out of the arguments.
#[derive(Debug)]
pub enum Arg {
    File { name: String, content: String },
    GridSize(Size),
    Help,
    Version,
}

#[derive(Debug)]
enum SizeError {
    OutOfRange(&'static str),
    FileNotFound,
}

fn parse_squared_size(size_str: &str) -> Result<Option<Arg>, SizeError> {
    if let Ok(parsed_size) = size_str.parse::<u16>() {
        match parsed_size {
            1..=MAX_GRID_SIZE => Ok(Some(Arg::GridSize(Size {
                width: parsed_size,
                height: parsed_size,
            }))),
            _ => Err(SizeError::OutOfRange("size")),
        }
    } else if util::is_numeric(size_str) {
        // A value >u16::MAX will not parse but might still be a number
        Err(SizeError::OutOfRange("size"))
    } else {
        Err(SizeError::FileNotFound)
    }
}

fn parse_size(width_str: &str, height_str: &str) -> Result<Option<Arg>, SizeError> {
    if let Ok(parsed_width) = width_str.parse::<u16>() {
        if let Ok(parsed_height) = height_str.parse::<u16>() {
            if !(1..=MAX_GRID_SIZE).contains(&parsed_width) {
                return Err(SizeError::OutOfRange("width"));
            }
            if !(1..=MAX_GRID_SIZE).contains(&parsed_height) {
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

    Err(SizeError::FileNotFound)
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
                return Err("Filename extension must be \"yaya\"".into());
            }

            let content = util::read_file_content(&mut file).map_err(|_| "File reading error")?;

            Ok(Some(Arg::File {
                name: first_string,
                content,
            }))
        }
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => {
                if first_string == "--help" || first_string == "-h" {
                    Ok(Some(Arg::Help))
                } else if first_string == "--version" || first_string == "-V" {
                    Ok(Some(Arg::Version))
                } else {
                    let result = if let Some(second_string) = second_string {
                        parse_size(&first_string, &second_string)
                    } else {
                        parse_squared_size(&first_string)
                    };

                    match result {
                        Ok(size) => Ok(size),
                        Err(SizeError::OutOfRange(thing)) => Err(format!(
                            "Grid {} must be in range 1 to {}",
                            thing, MAX_GRID_SIZE
                        )
                        .into()),
                        Err(SizeError::FileNotFound) => Err("File not found".into()),
                    }
                }
            }
            _ => Err("File opening error".into()),
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
                    Err("Second argument is not valid UTF-8".into())
                }
            } else {
                parse_strings(first_string, None)
            }
        } else {
            Err("First argument is not valid UTF-8".into())
        }
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_strings() {
        assert!(matches!(
            parse_strings(String::from("example.yaya"), None),
            Ok(Some(Arg::File {
                name: _,
                content: _
            }))
        ));
    }

    #[test]
    fn test_parse_squared_size() {
        assert!(matches!(
            parse_squared_size("99"),
            Ok(Some(Arg::GridSize(Size {
                width: 99,
                height: 99
            })))
        ));

        assert!(!matches!(
            parse_squared_size("100"),
            Ok(Some(Arg::GridSize(Size {
                width: 100,
                height: 100
            })))
        ));

        assert!(!matches!(
            parse_squared_size("0"),
            Ok(Some(Arg::GridSize(Size {
                width: 0,
                height: 0
            })))
        ));
    }

    #[test]
    fn test_parse_size() {
        assert!(matches!(
            parse_size("25", "50"),
            Ok(Some(Arg::GridSize(Size {
                width: 25,
                height: 50
            })))
        ));

        assert!(!matches!(
            parse_size("100", "99"),
            Ok(Some(Arg::GridSize(Size {
                width: 100,
                height: 99
            })))
        ));

        assert!(!matches!(
            parse_size("0", "0"),
            Ok(Some(Arg::GridSize(Size {
                width: 0,
                height: 0
            })))
        ));
    }
}
