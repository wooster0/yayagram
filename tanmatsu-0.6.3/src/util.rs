use std::{convert::TryFrom, fmt};

#[derive(Clone, Copy, PartialEq, Eq, Default, Hash, Debug)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl std::ops::Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Hash, Debug)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Size {
    pub fn product(&self) -> u32 {
        self.width as u32 * self.height as u32
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Color {
    // 4-bit colors
    DarkRed,
    DarkGreen,
    DarkYellow,
    DarkBlue,
    DarkMagenta,
    DarkCyan,
    /// A terminal's default background color.
    Black,
    /// A terminal's default foreground color.
    Gray,
    DarkGray,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    // 8-bit colors
    Byte(u8),
    // 24-bit colors
    Rgb {
        r: u8,
        g: u8,
        b: u8,
    },
}

impl Default for Color {
    fn default() -> Self {
        Color::Black
    }
}

impl Color {
    pub const GRAYSCALE_COLOR_COUNT: u8 = 24;
    pub const FOUR_BIT_COLOR_COUNT: u8 = 8 * 2;

    pub fn invert(&self) -> Self {
        use Color::*;

        match self {
            Byte(mut byte) => {
                if byte >= u8::MAX - Color::GRAYSCALE_COLOR_COUNT {
                    byte = u8::MAX - byte;
                    byte += u8::MAX - Color::GRAYSCALE_COLOR_COUNT - 1;
                } else {
                    byte = u8::MAX - byte - Color::FOUR_BIT_COLOR_COUNT + 1;
                }
                Byte(byte)
            }
            Rgb { r, g, b } => Rgb {
                r: u8::MAX - r,
                g: u8::MAX - g,
                b: u8::MAX - b,
            },
            Black | DarkGray => White,
            _ => Black,
        }
    }

    /// Tries to parse the input into an RGB color.
    /// It can parse the following RGB notations:
    ///
    /// - [X] 8-bit,       e.g. (255, 0, 0)
    /// - [X] Hexadecimal, e.g. #FF0000
    /// - [ ] Float,       e.g. (1.0, 0.0, 0.0)
    /// - [ ] Percentage,  e.g. (100%, 0%, 0%)
    ///
    /// See <https://en.wikipedia.org/wiki/RGB_color_model> for more information.
    ///
    pub fn from_rgb(string: &str) -> Option<Color> {
        let mut r: Option<u8> = None;
        let mut g: Option<u8> = None;
        let mut b: Option<u8> = None;

        let mut component = &mut r;

        let mut digits_in_a_row = 0;
        let mut index = 0;
        for char in string.chars() {
            match char {
                '0'..='9' => {
                    if let Some(byte) = char.to_digit(10) {
                        *component = if let Some(component) = *component {
                            Some(
                                u8::try_from(component as usize * 10 + byte as usize)
                                    .unwrap_or(u8::MAX),
                            )
                        } else {
                            Some(byte as u8)
                        };
                    };
                    digits_in_a_row += 1;
                }
                _ if char.is_ascii_hexdigit() => {
                    if let Some(color) = Self::from_hex(&string[index..]) {
                        return Some(color);
                    }
                    digits_in_a_row += 1;
                }
                _ => {
                    component = match (r, g, b) {
                        (None, None, None) => &mut r,
                        (Some(_), None, None) => &mut g,
                        (Some(_), Some(_), None) => &mut b,
                        (Some(r), Some(g), Some(b)) => return Some(Color::Rgb { r, g, b }),
                        _ => unreachable!(),
                    };
                    digits_in_a_row = 0;
                }
            }

            index += 1;

            if digits_in_a_row == 6 && index >= digits_in_a_row {
                if let Some(color) = Self::from_hex(&string[index - digits_in_a_row..]) {
                    return Some(color);
                }
            }
        }

        match (r, g, b) {
            (Some(r), None, None) => Some(Color::Rgb { r, g: 0, b: 0 }),
            (Some(r), Some(g), None) => Some(Color::Rgb { r, g, b: 0 }),
            (Some(r), Some(g), Some(b)) => Some(Color::Rgb { r, g, b }),
            _ => None,
        }
    }

    pub fn from_hex(string: &str) -> Option<Color> {
        if let (Some(r), Some(g), Some(b)) = (string.get(..2), string.get(2..4), string.get(4..6)) {
            let r = u8::from_str_radix(r, 16);
            let g = u8::from_str_radix(g, 16);
            let b = u8::from_str_radix(b, 16);
            if let (Ok(r), Ok(g), Ok(b)) = (r, g, b) {
                Some(Color::Rgb { r, g, b })
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgb(r: u8, g: u8, b: u8) -> Option<Color> {
        Some(Color::Rgb { r, g, b })
    }

    #[test]
    fn test_parse_rgb_color() {
        fn parse(string: &str) -> Option<Color> {
            Color::from_rgb(string)
        }

        assert_eq!(parse("255, 255, 255"), rgb(255, 255, 255));
        assert_eq!(parse("200,255,255"), rgb(200, 255, 255));
        assert_eq!(parse("-200,-255,-255"), rgb(200, 255, 255));
        assert_eq!(parse("(255,200,255)"), rgb(255, 200, 255));
        assert_eq!(parse("www255,255,200www"), rgb(255, 255, 200));
        assert_eq!(parse("    www100www,www0www,www100www"), rgb(100, 0, 100));
        assert_eq!(parse("www100www,www20www,,,"), rgb(100, 20, 0));
        assert_eq!(parse("   123"), rgb(123, 0, 0));
        assert_eq!(parse("99999,99999,99999"), rgb(255, 255, 255));
        assert_eq!(parse("FF0000,00FF00,0000FF"), rgb(255, 0, 0));
        assert_eq!(parse("00FF00"), rgb(0, 255, 0));
        assert_eq!(parse("    00FF00"), rgb(0, 255, 0));
        assert_eq!(parse("-50,-50,-50-00FF00"), rgb(50, 50, 50));
        assert_eq!(parse("256"), rgb(255, 0, 0));
        assert_eq!(parse("99999"), rgb(255, 0, 0));
        assert_eq!(parse("rgb(123,255,100)"), rgb(123, 255, 100));
        assert_eq!(parse("123,255,100"), rgb(123, 255, 100));
        // assert_eq!(parse("255,255,255555555"), rgb(255, 255, 255));
        // assert_eq!(parse("255,255,255efefef"), rgb(255, 255, 255));
    }

    #[test]
    fn test_parse_hex() {
        fn parse(string: &str) -> Option<Color> {
            Color::from_hex(string)
        }

        assert_eq!(parse("dea584"), rgb(222, 165, 132));
        assert_eq!(parse("ff0000"), rgb(255, 0, 0));
    }
}
