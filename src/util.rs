use std::{
    fs,
    io::{self, Read, Seek},
};
use terminal::util::Point;

/// Returns an iterator over the points from `start_point` to `point2`.
pub fn get_line_points(start_point: Point, end_point: Point) -> impl Iterator<Item = Point> {
    line_drawing::Bresenham::new(
        (start_point.x as i16, start_point.y as i16),
        (end_point.x as i16, end_point.y as i16),
    )
    .map(|(x, y)| Point {
        x: x as u16,
        y: y as u16,
    })
}

/// Erases all of the writer's file's contents.
pub fn clear_file(writer: &mut io::BufWriter<fs::File>) -> Result<(), &'static str> {
    fn inner(writer: &mut io::BufWriter<fs::File>) -> io::Result<()> {
        // Truncate the underlying file to zero bytes
        writer.get_ref().set_len(0)?;

        // `set_len` leaves the cursor unchanged.
        // Set the cursor to the start.
        writer.seek(io::SeekFrom::Start(0))?;

        Ok(())
    }

    match inner(writer) {
        Ok(()) => Ok(()),
        Err(_) => Err("file clear failed"),
    }
}

/// Checks whether `str` is a number consisting of ASCII digits, regardless of the length, negative or not.
///
/// Note that an empty string returns `true`.
///
/// ```
/// assert!(is_numeric("---123"));
/// assert!(is_numeric("-123456789012345678901234567890"));
/// assert!(is_numeric("123"));
/// assert!(is_numeric("0"));
///
/// assert!(!is_numeric("---123-"));
/// assert!(!is_numeric("hello"));
/// assert!(!is_numeric(" "));
/// assert!(!is_numeric("-"));
/// ```
pub fn is_numeric(str: &str) -> bool {
    let mut digit_encountered = false;
    str.chars().all(|char| {
        if char.is_ascii_digit() {
            digit_encountered = true;
            true
        } else {
            char == '-' && !digit_encountered
        }
    }) && digit_encountered
}

/// Returns the optimal string capacity based on the file's length.
pub fn optimal_string_capacity(file: &fs::File) -> io::Result<usize> {
    Ok(file.metadata()?.len() as usize + 1)
}

/// Reads the file's content into a string.
pub fn read_file_content(file: &mut fs::File) -> io::Result<String> {
    let mut string = String::with_capacity(optimal_string_capacity(&file)?);
    file.read_to_string(&mut string)?;
    Ok(string)
}
