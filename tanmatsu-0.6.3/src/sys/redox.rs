//! (Unfinished) terminal implementation for the Redox operating system.

// TODO: if the Redox backend is to be implemented, the methods should probably be defined in a trait

// Also see `terminal` crate as a reference

use crate::{
    event::{Event, Key, MouseButton, MouseEventKind},
    util::{Color, Point, Size},
    Terminal,
};
use std::io::{self, Write};
use std::time::Duration;
use termion::{event, input::TermRead, is_tty, raw::IntoRawMode, screen};

impl<'a> Terminal<'a> {
    pub fn enter_alternate_dimension(&mut self) {
        write!(self.stdout, "{}", screen::ToAlternateScreen);
    }
    pub fn exit_alternate_dimension(&mut self) {
        write!(self.stdout, "{}", screen::ToMainScreen);
    }

    pub fn set_title(&mut self, title: &str) {
        write!(self.stdout, "\u{1B}]0;{}\u{7}", title);
    }

    pub fn enable_raw_mode(&mut self) {
        self.stdout.into_raw_mode();
    }

    /// Reads an event. It also sets the new size if the terminal has been resized, hence a mutable borrow of `self` is required.
    pub fn read_event(&mut self) -> Option<Event> {
        if let Some(Ok(termion_event)) = self.stdin.events().next() {
            match termion_event {
                // event::Event::Mouse(event) => {
                //     fn translate_button(button: event::MouseButton) -> MouseButton {
                //         match button {
                //             event::MouseButton::Left => MouseButton::Left,
                //             event::MouseButton::Middle => MouseButton::Middle,
                //             event::MouseButton::Right => MouseButton::Right,
                //         }
                //     }

                //     // match event {
                //     //     event::MouseEvent::Press(button)
                //     // }

                //     // let kind = match event.kind {
                //     //     event::MouseEventKind::Moved => MouseEventKind::Move,
                //     //     event::MouseEventKind::Drag(button) => {
                //     //         MouseEventKind::Drag(translate_button(button))
                //     //     }
                //     //     event::MouseEventKind::Down(button) => {
                //     //         MouseEventKind::Press(translate_button(button))
                //     //     }
                //     //     event::MouseEventKind::Up(button) => {
                //     //         MouseEventKind::Release(translate_button(button))
                //     //     }
                //     //     event::MouseEventKind::ScrollUp => MouseEventKind::ScrollUp,
                //     //     event::MouseEventKind::ScrollDown => MouseEventKind::ScrollDown,
                //     // };

                //     let point = Point {
                //         x: event.column,
                //         y: event.row,
                //     };

                //     Event::Mouse { kind, point }
                // }
                event::Event::Key(key) => Event::Key(match key {
                    event::Key::Char(char) => Key::Char(char),
                    event::Key::Up => Key::Up,
                    event::Key::Down => Key::Down,
                    event::Key::Left => Key::Left,
                    event::Key::Right => Key::Right,
                    event::Key::Char('\t') => Key::Tab,
                    event::Key::Char('\n') => Key::Enter,
                    event::Key::F(number) => Key::F(number),
                    event::Key::Backspace => Key::Backspace,
                    event::Key::Esc => Key::Esc,
                    _ => return None,
                }),
                event::Event(width, height) => {
                    self.size = Size { width, height };
                    Event::Resize
                }
            }
            Some(event)
        } else {
            None
        }
    }

    pub fn is_tty(stdout: &io::StdoutLock) -> bool {
        is_tty(&stdout)
    }
}
