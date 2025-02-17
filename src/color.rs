use embedded_graphics_core::pixelcolor::{BinaryColor, PixelColor};

/// Color with 3 states.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum TriColor {
    Black,
    White,
    Red,
}

impl PixelColor for TriColor {
    type Raw = ();
}

impl From<u8> for TriColor {
    fn from(value: u8) -> Self {
        match value {
            0 => TriColor::Black,
            1 => TriColor::White,
            2 => TriColor::Red,
            _ => panic!("invalid color value"),
        }
    }
}

impl From<BinaryColor> for TriColor {
    fn from(b: BinaryColor) -> TriColor {
        match b {
            BinaryColor::On => Self::Black,
            BinaryColor::Off => Self::White,
        }
    }
}

impl From<TriColor> for u8 {
    fn from(c: TriColor) -> u8 {
        match c {
            TriColor::White => 0xFF,
            TriColor::Black | TriColor::Red => 0x00,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_u8() {
        assert_eq!(TriColor::Black, TriColor::from(0u8));
        assert_eq!(TriColor::White, TriColor::from(1u8));
        assert_eq!(TriColor::Red, TriColor::from(2u8));
    }
}
