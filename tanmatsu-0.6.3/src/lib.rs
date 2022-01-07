pub mod event;
mod sys;
pub mod util;

use crate::util::{Point, Size};
use std::io::{self, Write};

// TODO: add `error` to abort program with message?

// TODO: return a result instead of `expect`ing?

// Once https://github.com/rust-lang/rust/pull/78515 is merged, some of this can be changed
#[derive(Debug)]
pub struct Terminal<'a> {
    pub stdout: io::BufWriter<io::StdoutLock<'a>>,
    pub size: Size,
    #[cfg(debug_assertions)]
    pub flush_count: usize,
    initialized: bool,
    with_mouse: bool,
    // #[cfg(not(target = "windows"))]
    // stdin: io::Stdin,
}

#[derive(Clone, Copy, Debug)]
#[non_exhaustive] // Prevent instantiation
pub struct NotTTY;

/// A terminal with an `io::Stdout` inside.
///
/// Every program can have only a single instance for writing.
/// The standard output stream is locked and no other instance can write.
impl<'a> Terminal<'a> {
    pub fn new(stdout: io::StdoutLock<'a>) -> Result<Self, NotTTY> {
        if !Self::is_tty(&stdout) {
            return Err(NotTTY);
        }

        let writer = io::BufWriter::new(stdout);

        Ok(Self {
            stdout: writer,
            size: Self::size(),
            #[cfg(debug_assertions)]
            flush_count: 0,
            initialized: false,
            with_mouse: false
            // #[cfg(not(target = "windows"))]
            // stdin: io::stdin(),
        })
    }

    pub fn write(&mut self, string: &str) {
        self.stdout.write_all(string.as_bytes()).unwrap();
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.stdout.write_all(bytes).unwrap();
    }

    pub fn flush(&mut self) {
        self.stdout.flush().unwrap();

        #[cfg(debug_assertions)]
        {
            if self.initialized {
                self.flush_count += 1;
                self.save_cursor_point();
                self.set_cursor(Point { x: 0, y: 0 });
                let flush_count = self.flush_count;
                self.write(&format!("Flush count: {}", flush_count));
                self.restore_cursor_point();
            }
        }
    }

    fn set_panic_hook(with_mouse: bool) {
        use std::panic;

        let current_panic_hook = panic::take_hook();

        panic::set_hook(Box::new(move |panic_info| {
            let stdout = io::stdout();

            let mut terminal = Terminal::new(stdout.lock()).unwrap();
            terminal.initialized = true;
            terminal.with_mouse = with_mouse;

            terminal.deinitialize();
            terminal.flush(); // Flush so that we can see the following output in the normal view

            current_panic_hook(panic_info);
        }));
    }

    /// Makes this terminal suitable for drawing and input.
    ///
    /// Note that this does not do anything until [`flush`] is used.
    pub fn initialize(&mut self, title: Option<&str>, with_mouse: bool) {
        self.enter_alternate_dimension();
        self.enable_raw_mode();
        self.hide_cursor();

        if let Some(title) = title {
            self.set_title(title);
        }

        if with_mouse {
            self.enable_mouse_capture();
        }

        Self::set_panic_hook(with_mouse);

        self.initialized = true;
    }

    /// Deinitializes the terminal back into its normal state.
    ///
    /// Note that this does not do anything until [`flush`] is used.
    pub fn deinitialize(&mut self) {
        if !self.initialized {
            panic!("terminal is not initialized");
        }

        self.exit_alternate_dimension();
        self.disable_raw_mode();
        self.show_cursor();

        if self.with_mouse {
            self.disable_mouse_capture();
        }

        self.initialized = false;
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x > 0 && point.x < self.size.width && point.y < self.size.height && point.y > 0
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
