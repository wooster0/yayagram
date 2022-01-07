//! Terminal implementation for all non-Redox operating systems.

use crate::{
    event::{Event, Key, MouseButton, MouseEvent, MouseEventKind},
    util::{Color, Point, Size},
    Terminal,
};
use crossterm::{cursor, event, style, terminal, tty::IsTty, QueueableCommand};
use std::{io, time::Duration};

// TODO: return result instead of unwrapping?

// > When I first did this, it was noticeably slower than the termion version(roughly 5-10 fps).
// > This is because calling into the Console API that often (once per character) is going to pull down performance.
// > Luckily, I could work around this by just checking if we were already using the color I wanted to render.
// > If we were, I didn't set the color again.

impl<'a> Terminal<'a> {
    pub fn enter_alternate_dimension(&mut self) {
        self.stdout.queue(terminal::EnterAlternateScreen).unwrap();
    }
    pub fn exit_alternate_dimension(&mut self) {
        self.stdout.queue(terminal::LeaveAlternateScreen).unwrap();
    }

    pub fn set_title(&mut self, title: &str) {
        self.stdout.queue(terminal::SetTitle(title)).unwrap();
    }

    pub fn enable_raw_mode(&self) {
        terminal::enable_raw_mode().unwrap();
    }
    pub fn disable_raw_mode(&self) {
        terminal::disable_raw_mode().unwrap();
    }

    // TODO: use custom escape sequence to be more specific about what mouse events exactly to take
    pub fn enable_mouse_capture(&mut self) {
        self.stdout.queue(event::EnableMouseCapture).unwrap();
        self.with_mouse = true;
    }
    pub fn disable_mouse_capture(&mut self) {
        self.stdout.queue(event::DisableMouseCapture).unwrap();
        self.with_mouse = false;
    }

    pub fn show_cursor(&mut self) {
        self.stdout.queue(cursor::Show).unwrap();
    }
    pub fn hide_cursor(&mut self) {
        self.stdout.queue(cursor::Hide).unwrap();
    }

    /// Reads an event. It also sets the new size if the terminal has been resized, hence a mutable borrow of `self` is required.
    pub fn read_event(&mut self) -> Option<Event> {
        if let Ok(crossterm_event) = event::read() {
            let event = match crossterm_event {
                event::Event::Mouse(event) => {
                    fn translate_button(button: event::MouseButton) -> MouseButton {
                        match button {
                            event::MouseButton::Left => MouseButton::Left,
                            event::MouseButton::Middle => MouseButton::Middle,
                            event::MouseButton::Right => MouseButton::Right,
                        }
                    }

                    let kind = match event.kind {
                        event::MouseEventKind::Moved => MouseEventKind::Move,
                        event::MouseEventKind::Drag(button) => {
                            MouseEventKind::Drag(translate_button(button))
                        }
                        event::MouseEventKind::Down(button) => {
                            MouseEventKind::Press(translate_button(button))
                        }
                        event::MouseEventKind::Up(button) => {
                            MouseEventKind::Release(translate_button(button))
                        }
                        event::MouseEventKind::ScrollUp => MouseEventKind::ScrollUp,
                        event::MouseEventKind::ScrollDown => MouseEventKind::ScrollDown,
                    };

                    let point = Point {
                        x: event.column,
                        y: event.row,
                    };

                    Event::Mouse(MouseEvent { kind, point })
                }
                event::Event::Key(event::KeyEvent { code, modifiers: _ }) => {
                    let key = match code {
                        event::KeyCode::Char(char) => Key::Char(char),
                        event::KeyCode::Up => Key::Up,
                        event::KeyCode::Down => Key::Down,
                        event::KeyCode::Left => Key::Left,
                        event::KeyCode::Right => Key::Right,
                        event::KeyCode::Tab => Key::Tab,
                        event::KeyCode::Enter => Key::Enter,
                        event::KeyCode::F(number) => Key::F(number),
                        event::KeyCode::Backspace => Key::Backspace,
                        event::KeyCode::Esc => Key::Esc,
                        _ => return None,
                    };

                    // let modifier = if modifiers == event::KeyModifiers::ALT {
                    //     Some(KeyModifier::Alt)
                    // } else if modifiers == event::KeyModifiers::CONTROL {
                    //     Some(KeyModifier::Control)
                    // } else if modifiers == event::KeyModifiers::SHIFT {
                    //     Some(KeyModifier::Shift)
                    // } else {
                    //     None
                    // };

                    Event::Key(key)
                }
                event::Event::Resize(width, height) => {
                    self.size = Size { width, height };
                    Event::Resize
                }
            };
            Some(event)
        } else {
            None
        }
    }

    pub fn poll_event(&mut self, timeout: Duration) -> Option<Event> {
        if let Ok(true) = crossterm::event::poll(timeout) {
            self.read_event()
        } else {
            None
        }
    }

    /// Sets the cursor to the top left corner.
    #[cfg(not(target_os = "windows"))]
    pub fn reset_cursor(&mut self) {
        self.write("\u{1b}[;H");
    }

    /// Sets the cursor to the top left corner.
    #[cfg(target_os = "windows")]
    pub fn reset_cursor(&mut self) {
        self.set_cursor(Point::default());
    }

    /// Sets the cursor to `point`.
    ///
    /// If possible, try to use the `move_cursor_{}_by` and `move_cursor_{}` methods instead for single operations.
    pub fn set_cursor(&mut self, point: Point) {
        self.stdout.queue(cursor::MoveTo(point.x, point.y)).unwrap();
    }

    /// Sets the cursor X-coordinate to `x`.
    pub fn set_cursor_x(&mut self, x: u16) {
        self.stdout.queue(cursor::MoveToColumn(x)).unwrap();
    }

    /// Sets the cursor Y-coordinate to `y`.
    pub fn set_cursor_y(&mut self, y: u16) {
        self.stdout.queue(cursor::MoveToRow(y)).unwrap();
    }

    pub fn move_cursor_up_by(&mut self, cells: u16) {
        self.stdout.queue(cursor::MoveUp(cells)).unwrap();
    }
    pub fn move_cursor_down_by(&mut self, cells: u16) {
        self.stdout.queue(cursor::MoveDown(cells)).unwrap();
    }
    pub fn move_cursor_left_by(&mut self, cells: u16) {
        self.stdout.queue(cursor::MoveLeft(cells)).unwrap();
    }
    pub fn move_cursor_right_by(&mut self, cells: u16) {
        self.stdout.queue(cursor::MoveRight(cells)).unwrap();
    }

    #[cfg(not(target_os = "windows"))]
    pub fn move_cursor_up(&mut self) {
        self.write("\u{1b}[A");
    }
    #[cfg(not(target_os = "windows"))]
    pub fn move_cursor_down(&mut self) {
        self.write("\u{1b}[B");
    }
    #[cfg(not(target_os = "windows"))]
    pub fn move_cursor_left(&mut self) {
        self.write("\u{1b}[D");
    }
    #[cfg(not(target_os = "windows"))]
    pub fn move_cursor_right(&mut self) {
        self.write("\u{1b}[C");
    }

    #[cfg(not(target_os = "windows"))]
    pub fn next_line(&mut self) {
        self.write("\u{1b}[E");
    }
    #[cfg(not(target_os = "windows"))]
    pub fn previous_line(&mut self) {
        self.write("\u{1b}[F");
    }

    #[cfg(target_os = "windows")]
    pub fn move_cursor_up(&mut self) {
        self.move_cursor_up_by(1);
    }
    #[cfg(target_os = "windows")]
    pub fn move_cursor_down(&mut self) {
        self.move_cursor_down_by(1);
    }
    #[cfg(target_os = "windows")]
    pub fn move_cursor_left(&mut self) {
        self.move_cursor_left_by(1);
    }
    #[cfg(target_os = "windows")]
    pub fn move_cursor_right(&mut self) {
        self.move_cursor_right_by(1);
    }

    #[cfg(target_os = "windows")]
    pub fn next_line(&mut self) {
        self.stdout.queue(cursor::MoveToNextLine(1)).unwrap();
    }
    #[cfg(target_os = "windows")]
    pub fn previous_line(&mut self) {
        self.stdout.queue(cursor::MoveToPreviousLine(1)).unwrap();
    }

    pub fn save_cursor_point(&mut self) {
        self.stdout.queue(cursor::SavePosition).unwrap();
    }
    pub fn restore_cursor_point(&mut self) {
        self.stdout.queue(cursor::RestorePosition).unwrap();
    }

    pub fn set_foreground_color(&mut self, color: Color) {
        self.stdout
            .queue(style::SetForegroundColor(Self::convert_color(color)))
            .unwrap();
    }
    pub fn set_background_color(&mut self, color: Color) {
        self.stdout
            .queue(style::SetBackgroundColor(Self::convert_color(color)))
            .unwrap();
    }

    //
    // TODO for the following methods: Do they work on Windows?
    //

    // Reference: https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands
    // NOTE: clipboard functionality can be added: https://github.com/alacritty/alacritty/blob/3e867a056018c507d79396cb5c5b4b8309c609c2/alacritty_terminal/src/ansi.rs#L440

    /// Changes the terminal's foreground text color to `hex_color`.
    ///
    /// `hex_color` must be a hexadecimal color such as `"FF0000"`.
    pub fn change_foreground_color(&mut self, hex_color: &str) {
        self.write(&format!("\u{1b}]10;#{}\u{7}", hex_color));
    }
    pub fn reset_foreground_color(&mut self) {
        self.write("\u{1b}]110\u{7}");
    }

    /// Changes the terminal's background text color to `hex_color`.
    ///
    /// `hex_color` must be a hexadecimal color such as `FF0000`.
    pub fn change_background_color(&mut self, hex_color: &str) {
        self.write(&format!("\u{1b}]11;#{}\u{7}", hex_color));
    }
    pub fn reset_background_color(&mut self) {
        self.write("\u{1b}]111\u{7}");
    }

    /// Changes the terminal's cursor color to `hex_color`.
    ///
    /// `hex_color` must be a hexadecimal color such as `FF0000`.
    pub fn change_cursor_color(&mut self, hex_color: &str) {
        self.write(&format!("\u{1b}]12;#{}\u{7}", hex_color));
    }
    pub fn reset_cursor_color(&mut self) {
        self.write("\u{1b}]112\u{7}");
    }

    pub fn enable_italic(&mut self) {
        self.write(&format!("{}", style::Attribute::Italic));
    }
    pub fn disable_italic(&mut self) {
        self.write(&format!("{}", style::Attribute::NoItalic));
    }

    pub fn reset_colors(&mut self) {
        self.stdout.queue(style::ResetColor).unwrap();
    }

    pub fn clear(&mut self) {
        self.stdout
            .queue(terminal::Clear(terminal::ClearType::All))
            .unwrap();
    }
    pub fn clear_from_cursor_to_end(&mut self) {
        self.stdout
            .queue(terminal::Clear(terminal::ClearType::FromCursorUp))
            .unwrap();
    }

    fn convert_color(color: Color) -> style::Color {
        match color {
            Color::Black => style::Color::Black,
            Color::DarkGray => style::Color::DarkGrey,
            Color::Red => style::Color::Red,
            Color::DarkRed => style::Color::DarkRed,
            Color::Green => style::Color::Green,
            Color::DarkGreen => style::Color::DarkGreen,
            Color::Yellow => style::Color::Yellow,
            Color::DarkYellow => style::Color::DarkYellow,
            Color::Blue => style::Color::Blue,
            Color::DarkBlue => style::Color::DarkBlue,
            Color::Magenta => style::Color::Magenta,
            Color::DarkMagenta => style::Color::DarkMagenta,
            Color::Cyan => style::Color::Cyan,
            Color::DarkCyan => style::Color::DarkCyan,
            Color::White => style::Color::White,
            Color::Gray => style::Color::Grey,
            Color::Rgb { r, g, b } => style::Color::Rgb { r, g, b },
            Color::Byte(rgb) => style::Color::AnsiValue(rgb),
        }
    }

    pub(crate) fn size() -> Size {
        let size = terminal::size().unwrap();
        Size {
            width: size.0,
            height: size.1,
        }
    }

    pub(crate) fn is_tty(stdout: &io::StdoutLock) -> bool {
        stdout.is_tty()
    }
}
