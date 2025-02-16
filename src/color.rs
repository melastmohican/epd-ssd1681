use embedded_graphics_core::pixelcolor::{BinaryColor, PixelColor};

/// Color with 3 states.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum EpdColor {
    Black,
    White,
    Red,
}

impl PixelColor for EpdColor {
    type Raw = ();
}

impl From<u8> for EpdColor {
    fn from(value: u8) -> Self {
        match value {
            0 => EpdColor::Black,
            1 => EpdColor::White,
            2 => EpdColor::Red,
            _ => panic!("invalid color value"),
        }
    }
}

impl From<BinaryColor> for EpdColor {
    fn from(b: BinaryColor) -> EpdColor {
        match b {
            BinaryColor::On => Self::Black,
            BinaryColor::Off => Self::White,
        }
    }
}