#![no_std]

pub mod cmd;
pub mod color;
pub mod driver;
pub mod flag;
pub mod graphics;
pub mod interface;

/// Maximum display height this driver supports
pub const HEIGHT: u8 = 200;

/// Maximum display width this driver supports
pub const WIDTH: u8 = 200;
