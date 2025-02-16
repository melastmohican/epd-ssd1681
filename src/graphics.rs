use crate::color::EpdColor;
use crate::{HEIGHT, WIDTH};
use embedded_graphics::framebuffer::{buffer_size, Framebuffer};
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::geometry::{OriginDimensions, Size};
use embedded_graphics_core::pixelcolor::raw::RawU1;
use embedded_graphics_core::pixelcolor::{raw, BinaryColor};
use embedded_graphics_core::Pixel;

/// Rotation of the display.
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum DisplayRotation {
    /// No rotation, normal display
    Rotate0,
    /// Rotate by 90 degress clockwise
    Rotate90,
    /// Rotate by 180 degress clockwise
    Rotate180,
    /// Rotate 270 degress clockwise, recommend
    Rotate270,
}

impl Default for DisplayRotation {
    fn default() -> Self {
        DisplayRotation::Rotate0
    }
}
pub struct Display {
    black_fbuf: Framebuffer<
        BinaryColor,
        RawU1,
        raw::LittleEndian,
        { WIDTH as usize },
        { HEIGHT as usize },
        { buffer_size::<BinaryColor>(WIDTH as usize, HEIGHT as usize) },
    >,
    red_fbuf: Framebuffer<
        BinaryColor,
        RawU1,
        raw::LittleEndian,
        { WIDTH as usize },
        { HEIGHT as usize },
        { buffer_size::<BinaryColor>(WIDTH as usize, HEIGHT as usize) },
    >,
    rotation: DisplayRotation,
    is_inverted: bool,
}

impl Display {
    pub fn new() -> Self {
        Self {
            black_fbuf: Framebuffer::<
                BinaryColor,
                _,
                _,
                { WIDTH as usize },
                { HEIGHT as usize },
                { buffer_size::<BinaryColor>(WIDTH as usize, HEIGHT as usize) },
            >::new(),
            red_fbuf: Framebuffer::<
                BinaryColor,
                _,
                _,
                { WIDTH as usize },
                { HEIGHT as usize },
                { buffer_size::<BinaryColor>(WIDTH as usize, HEIGHT as usize) },
            >::new(),
            rotation: DisplayRotation::default(),
            is_inverted: true,
        }
    }

    /// Clear the buffers, filling them a single color.
    pub fn clear(&mut self, color: EpdColor) {
        let (black, red) = match color {
            EpdColor::White => (BinaryColor::On, BinaryColor::Off),
            EpdColor::Black => (BinaryColor::Off, BinaryColor::Off),
            EpdColor::Red => (BinaryColor::On, BinaryColor::On),
        };

        let _ = self.black_fbuf.clear(black);
        let _ = self.red_fbuf.clear(red);
    }

    fn set_pixel(&mut self, x: u32, y: u32, color: EpdColor) {
        let (index, bit) = self.from_rotation(x, y, WIDTH as u32, HEIGHT as u32);
        let index = index as usize;

        match color {
            EpdColor::Black => {
                self.black_fbuf.data_mut()[index] &= !bit;
                self.red_fbuf.data_mut()[index] &= !bit;
            }
            EpdColor::White => {
                self.black_fbuf.data_mut()[index] |= bit;
                self.red_fbuf.data_mut()[index] &= !bit;
            }
            EpdColor::Red => {
                self.black_fbuf.data_mut()[index] |= bit;
                self.red_fbuf.data_mut()[index] |= bit;
            }
        }
    }

    fn from_rotation(&mut self, x: u32, y: u32, width: u32, height: u32) -> (u32, u8) {
        match self.rotation {
            DisplayRotation::Rotate0 => (x / 8 + (width / 8) * y, 0x80 >> (x % 8)),
            DisplayRotation::Rotate90 => ((width - 1 - y) / 8 + (width / 8) * x, 0x01 << (y % 8)),
            DisplayRotation::Rotate180 => (
                ((width / 8) * height - 1) - (x / 8 + (width / 8) * y),
                0x01 << (x % 8),
            ),
            DisplayRotation::Rotate270 => (y / 8 + (height - 1 - x) * (width / 8), 0x80 >> (y % 8)),
        }
    }

    pub fn set_rotation(&mut self, rotation: DisplayRotation) {
        self.rotation = rotation;
    }

    pub fn rotation(&self) -> DisplayRotation {
        self.rotation
    }

    pub fn is_inverted(&self) -> bool {
        self.is_inverted
    }

    pub fn black_data(
        &self,
    ) -> &[u8; { buffer_size::<BinaryColor>(WIDTH as usize, HEIGHT as usize) }] {
        self.black_fbuf.data()
    }
    pub fn red_data(
        &self,
    ) -> &[u8; { buffer_size::<BinaryColor>(WIDTH as usize, HEIGHT as usize) }] {
        self.red_fbuf.data()
    }
}

impl DrawTarget for Display {
    type Color = EpdColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels.into_iter() {
            match color {
                EpdColor::White => {
                    self.black_fbuf.draw_iter([Pixel(point, BinaryColor::On)])?;
                    self.red_fbuf.draw_iter([Pixel(point, BinaryColor::Off)])?;
                }
                EpdColor::Black => {
                    self.black_fbuf
                        .draw_iter([Pixel(point, BinaryColor::Off)])?;
                    self.red_fbuf.draw_iter([Pixel(point, BinaryColor::Off)])?;
                }
                EpdColor::Red => {
                    self.black_fbuf.draw_iter([Pixel(point, BinaryColor::On)])?;
                    self.red_fbuf.draw_iter([Pixel(point, BinaryColor::On)])?;
                }
            }
        }
        Ok(())
    }
}

impl OriginDimensions for Display {
    fn size(&self) -> Size {
        //if display is rotated 90 deg or 270 then swap height and width
        match self.rotation() {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
                Size::new(WIDTH.into(), HEIGHT.into())
            }
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => {
                Size::new(HEIGHT.into(), WIDTH.into())
            }
        }
    }
}
