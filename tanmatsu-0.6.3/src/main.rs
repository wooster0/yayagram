//! This file exists for testing purposes.

use std::io;

fn main() {
    let stdout = io::stdout();
    let mut terminal = tanmatsu::Terminal::new(stdout.lock()).unwrap();

    terminal.initialize(None, false);

    terminal.flush();

    std::thread::park();
}
