use crate::color::TriColor;
use crate::{HEIGHT, WIDTH};
use embedded_graphics::framebuffer::{buffer_size, Framebuffer};
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::geometry::{OriginDimensions, Size};
use embedded_graphics_core::pixelcolor::raw::RawU1;
use embedded_graphics_core::pixelcolor::{raw, BinaryColor};
use embedded_graphics_core::Pixel;

/// Rotation of the display.
#[derive(Clone, Copy, Debug, Default)]
#[repr(u8)]
pub enum DisplayRotation {
    /// No rotation, normal display
    #[default]
    Rotate0,
    /// Rotate by 90 degress clockwise
    Rotate90,
    /// Rotate by 180 degress clockwise
    Rotate180,
    /// Rotate 270 degress clockwise, recommend
    Rotate270,
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

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}
impl Display {
    pub fn new() -> Self {
        Display {
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
    pub fn clear(&mut self, color: TriColor) {
        let (black, red) = match color {
            TriColor::White => (BinaryColor::On, BinaryColor::Off),
            TriColor::Black => (BinaryColor::Off, BinaryColor::Off),
            TriColor::Red => (BinaryColor::On, BinaryColor::On),
        };

        let _ = self.black_fbuf.clear(black);
        let _ = self.red_fbuf.clear(red);
    }

    #[allow(dead_code)]
    fn set_pixel(&mut self, x: u32, y: u32, color: TriColor) {
        let (index, bit) = find_position(x, y, WIDTH as u32, HEIGHT as u32, self.rotation);
        let index = index as usize;

        match color {
            TriColor::Black => {
                self.black_fbuf.data_mut()[index] &= !bit;
                self.red_fbuf.data_mut()[index] &= !bit;
            }
            TriColor::White => {
                self.black_fbuf.data_mut()[index] |= bit;
                self.red_fbuf.data_mut()[index] &= !bit;
            }
            TriColor::Red => {
                self.black_fbuf.data_mut()[index] |= bit;
                self.red_fbuf.data_mut()[index] |= bit;
            }
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

    pub fn black_data(&self) -> &[u8; buffer_size::<BinaryColor>(WIDTH as usize, HEIGHT as usize)] {
        self.black_fbuf.data()
    }
    pub fn red_data(&self) -> &[u8; buffer_size::<BinaryColor>(WIDTH as usize, HEIGHT as usize)] {
        self.red_fbuf.data()
    }
}
fn find_rotation(x: u32, y: u32, width: u32, height: u32, rotation: DisplayRotation) -> (u32, u32) {
    let nx;
    let ny;
    match rotation {
        DisplayRotation::Rotate0 => {
            nx = x;
            ny = y;
        }
        DisplayRotation::Rotate90 => {
            nx = width - 1 - y;
            ny = x;
        }
        DisplayRotation::Rotate180 => {
            nx = width - 1 - x;
            ny = height - 1 - y;
        }
        DisplayRotation::Rotate270 => {
            nx = y;
            ny = height - 1 - x;
        }
    }
    (nx, ny)
}

fn find_position(x: u32, y: u32, width: u32, height: u32, rotation: DisplayRotation) -> (u32, u8) {
    let (nx, ny) = find_rotation(x, y, width, height, rotation);
    (nx / 8 + ((width + 7) / 8) * ny, 0x80 >> (nx % 8))
}

impl DrawTarget for Display {
    type Color = TriColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels.into_iter() {
            match color {
                TriColor::White => {
                    self.black_fbuf.draw_iter([Pixel(point, BinaryColor::On)])?;
                    self.red_fbuf.draw_iter([Pixel(point, BinaryColor::Off)])?;
                }
                TriColor::Black => {
                    self.black_fbuf
                        .draw_iter([Pixel(point, BinaryColor::Off)])?;
                    self.red_fbuf.draw_iter([Pixel(point, BinaryColor::Off)])?;
                }
                TriColor::Red => {
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
