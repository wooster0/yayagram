use std::{
    fs,
    io::{self, Seek},
};

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
