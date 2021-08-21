use crate::{
    args::FILE_EXTENSION,
    grid::{builder::Builder, Cell, Grid},
    util,
};
use std::{
    fs,
    io::{self, Write},
    path::Path,
};
use terminal::util::Size;

#[derive(Default)]
pub struct Editor {
    pub toggled: bool,
    writer: Option<io::BufWriter<fs::File>>,
    pub filename: String,
}

impl Editor {
    pub fn toggle(&mut self) {
        self.toggled = !self.toggled;
    }

    fn serialize(grid: &Grid, writer: &mut io::BufWriter<fs::File>) -> io::Result<()> {
        fn write_dash_line(writer: &mut io::BufWriter<fs::File>, width: u16) -> io::Result<()> {
            writer.write_all(b"+")?;
            for _ in 0..width {
                writer.write_all(b"----")?;
            }
            writer.write_all(b"+\n")?;

            Ok(())
        }

        let mut help: [Option<&str>; 4] = [None; 4];

        write_dash_line(writer, grid.size.width)?;

        for cells in grid.cells.chunks(grid.size.width as usize) {
            for _ in 0..2 {
                writer.write_all(b"|")?;
                for cell in cells {
                    let cell_half = match cell {
                        Cell::Empty => {
                            "    " // Represents emptiness.
                        }
                        Cell::Filled => {
                            help[0] = Some("1: filled");
                            "1111" // Represents true, i.e. filled.
                        }
                        Cell::Crossed => {
                            help[1] = Some("X: crossed");
                            "XXXX" // Looks like a cross.
                        }
                        Cell::Maybed => {
                            help[2] = Some("?: maybed");
                            "????" // Represents unclearness.
                        }
                        Cell::Measured(_) => {
                            help[3] = Some("R: measured");
                            "RRRR" // Resembles 尺 which is a unit of measure.
                        }
                    };
                    writer.write_all(cell_half.as_bytes())?;
                }
                writer.write_all(b"|\n")?;
            }
        }

        write_dash_line(writer, grid.size.width)?;

        writer.write_all(b"\n")?;

        Self::write_help(writer, help)?;

        writer.flush()?;

        Ok(())
    }

    #[allow(unstable_name_collisions)] // in the future `intersperse` will be in the std
    fn write_help(writer: &mut io::BufWriter<fs::File>, help: [Option<&str>; 4]) -> io::Result<()> {
        use itertools::Itertools;

        for part in help.iter().filter_map(|part| *part).intersperse(", ") {
            writer.write_all(part.as_bytes())?;
        }

        Ok(())
    }

    fn new_writer(&mut self, builder: &Builder) -> Result<io::BufWriter<fs::File>, &'static str> {
        let mut open_options = fs::OpenOptions::new();
        open_options.create_new(true).write(true);

        let mut index = 1;
        let file = loop {
            self.filename = format!("grid-{}.{}", index, FILE_EXTENSION);
            let file = open_options.open(&self.filename);
            match file {
                Err(err) => match err.kind() {
                    io::ErrorKind::AlreadyExists => {
                        if index == 9 {
                            return Err("Too many grid files");
                        }
                        index += 1;
                    }
                    io::ErrorKind::PermissionDenied => return Err("Permission denied"),
                    _ => return Err("File saving error"),
                },
                Ok(file) => break file,
            }
        };

        let writer = io::BufWriter::with_capacity(builder.grid.size.product() as usize, file);

        Ok(writer)
    }

    /// Saves the grid to the hard drive, returning the filename or an error.
    pub fn save_grid(&mut self, builder: &Builder) -> Result<(), &'static str> {
        let writer = self.writer.take();

        let mut writer = match writer {
            Some(mut writer) => {
                // We saved this grid previously so we already have a writer
                // but does the file for it still exist?
                if !Path::new(&self.filename).exists() {
                    match self.new_writer(builder) {
                        Ok(writer) => (writer),
                        Err(err) => {
                            return Err(err);
                        }
                    }
                } else {
                    // The file still exists so we will overwrite it

                    util::clear_file(&mut writer)?;

                    writer
                }
            }
            None => {
                // This is the first time we are saving the grid
                match self.new_writer(builder) {
                    Ok(writer) => (writer),
                    Err(err) => {
                        return Err(err);
                    }
                }
            }
        };

        if Self::serialize(&builder.grid, &mut writer).is_err() {
            return Err("Save failed");
        }

        self.writer = Some(writer);

        Ok(())
    }
}

pub struct LoadError {
    pub message: &'static str,
    pub line_number: Option<usize>,
}

fn deserialize(str: &str) -> Result<(Size, Vec<Cell>), LoadError> {
    let mut lines = str.lines();

    // Skip dash line
    lines.next().ok_or(LoadError {
        message: "expected line",
        line_number: Some(1),
    })?;

    let mut cells = Vec::<Cell>::new();

    let mut width: Option<u16> = None;
    let mut height: Option<u16> = None;

    for (index, line) in lines.step_by(2).enumerate() {
        let mut chars = line.chars();

        match chars.next() {
            Some('|') => {}
            Some('+') => break,
            _ => {
                return Err(LoadError {
                    message: "expected '|' or '+' at start of line",
                    line_number: Some(index),
                })
            }
        }

        let mut line_width: Option<u16> = None;

        for char in chars.step_by(4) {
            if char == '|' {
                break;
            }
            let cell = match char {
                ' ' => Cell::Empty,
                '1' => Cell::Filled,
                'X' => Cell::Crossed,
                '?' => Cell::Maybed,
                'R' => Cell::Measured(None),
                _ => {
                    return Err(LoadError {
                        message: "expected ' ', '1', 'X', '?' or 'R'",
                        line_number: Some(index),
                    })
                }
            };
            cells.push(cell);

            if let Some(line_width) = &mut line_width {
                *line_width += 1;
            } else {
                line_width = Some(1);
            }
        }

        if width.is_none() {
            width = Some(line_width.ok_or(LoadError {
                message: "no width",
                line_number: Some(index),
            })?);
        }

        if let Some(height) = &mut height {
            *height += 1;
        } else {
            height = Some(1);
        }
    }

    let width = width.ok_or(LoadError {
        message: "no width",
        line_number: None,
    })?;
    let height = height.ok_or(LoadError {
        message: "no height",
        line_number: None,
    })?;

    Ok((Size { width, height }, cells))
}

pub fn load_grid(file_content: &str) -> Result<Grid, LoadError> {
    let (size, cells) = deserialize(file_content)?;
    Ok(Grid::new(size, cells))
}
