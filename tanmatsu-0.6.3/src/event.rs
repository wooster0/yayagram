//! Terminal events defined specific to usage.

use crate::util::Point;

#[derive(Clone, Copy, Debug)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

#[derive(Clone, Copy, Debug)]
pub enum MouseEventKind {
    ScrollUp,
    ScrollDown,
    Move,
    Drag(MouseButton),
    Press(MouseButton),
    Release(MouseButton),
}

#[derive(Clone, Copy, Debug)]
pub enum Key {
    Char(char),
    // Alt(char),
    //  Ctrl(char),
    Up,
    Down,
    Left,
    Right,
    Tab,
    Enter,
    F(u8),
    Backspace,
    Esc,
}

// #[derive(Debug)]
// pub struct KeyEvent {
//     pub key: Key,
//     pub modifier: Option<KeyModifier>,
// }

// #[derive(Debug)]
// pub enum KeyModifier {
//     Shift,
//     Control,
//     Alt,
// }

#[derive(Clone, Copy, Debug)]
pub struct MouseEvent {
    pub kind: MouseEventKind,
    pub point: Point,
    // TODO: modifier: Option<KeyModifier> (or bitflags for multipl events)
}

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Key(Key),
    Mouse(MouseEvent),
    /// No `Size` included. Call [`crate::Terminal::size`] instead.
    Resize,
}
