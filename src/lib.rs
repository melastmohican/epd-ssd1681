#![no_std]

mod cmd;
mod color;
mod driver;
mod flag;
mod graphics;
mod interface;

/// Maximum display height this driver supports
pub const HEIGHT: u8 = 200;

/// Maximum display width this driver supports
pub const WIDTH: u8 = 200;
