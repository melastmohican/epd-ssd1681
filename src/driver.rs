use crate::cmd::Cmd;
use crate::flag::Flag;
use crate::interface::{DisplayError, DisplayInterface};
use crate::{HEIGHT, WIDTH};
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::spi::SpiDevice;

/// A configured display with a hardware interface.
pub struct Ssd1681<SPI, CS, BUSY, DC, RST> {
    interface: DisplayInterface<SPI, CS, BUSY, DC, RST>,
}

impl<SPI, CS, BUSY, DC, RST> Ssd1681<SPI, CS, BUSY, DC, RST>
where
    SPI: SpiDevice,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    /// Create and initialize the display driver
    pub fn new(
        spi: SPI,
        cs: CS,
        busy: BUSY,
        dc: DC,
        rst: RST,
        delay: &mut impl DelayNs,
    ) -> Result<Self, DisplayError>
    where
        Self: Sized,
    {
        let interface = DisplayInterface::new(spi, cs, busy, dc, rst);
        let mut ssd1681 = Ssd1681 { interface };
        ssd1681.init(delay)?;
        Ok(ssd1681)
    }

    /// Initialise the controller
    pub fn init(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        self.interface.reset(delay);
        self.interface.cmd(Cmd::SW_RESET)?;
        self.interface.wait_until_idle(delay);

        self.interface
            .cmd_with_data(Cmd::DRIVER_CONTROL, &[(HEIGHT - 1) as u8, 0x00, 0x00])?;

        self.interface
            .cmd_with_data(Cmd::DATA_MODE, &[Flag::DATA_ENTRY_INCRY_INCRX])?;

        self.use_full_frame()?;

        self.interface.cmd_with_data(
            Cmd::WRITE_BORDER,
            &[Flag::BORDER_WAVEFORM_FOLLOW_LUT | Flag::BORDER_WAVEFORM_LUT1],
        )?;

        self.interface
            .cmd_with_data(Cmd::TEMP_CONTROL, &[Flag::INTERNAL_TEMP_SENSOR])?;

        self.interface.wait_until_idle(delay);
        Ok(())
    }

    /// Update the whole BW buffer on the display driver
    pub fn update_bw_frame(&mut self, buffer: &[u8]) -> Result<(), DisplayError> {
        self.use_full_frame()?;
        self.interface.cmd_with_data(Cmd::WRITE_BWRAM, &buffer)
    }

    /// Update the whole Red buffer on the display driver
    pub fn update_red_frame(&mut self, buffer: &[u8]) -> Result<(), DisplayError> {
        self.use_full_frame()?;
        self.interface.cmd_with_data(Cmd::WRITE_REDRAM, &buffer)
    }

    /// Start an update of the whole display
    pub fn display_frame(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        self.interface
            .cmd_with_data(Cmd::DISP_CTRL2, &[Flag::DISPLAY_MODE_1])?;
        self.interface.cmd(Cmd::MASTER_ACTIVATE)?;

        self.interface.wait_until_idle(delay);

        Ok(())
    }

    /// Make the whole black and white frame on the display driver white
    pub fn clear_bw_frame(&mut self) -> Result<(), DisplayError> {
        self.use_full_frame()?;

        // TODO: allow non-white background color
        let color = 0xFF;

        self.interface.cmd(Cmd::WRITE_BWRAM)?;
        self.interface
            .data_x_times(color, u32::from(WIDTH) / 8 * u32::from(HEIGHT))?;
        Ok(())
    }

    /// Make the whole red frame on the display driver white
    pub fn clear_red_frame(&mut self) -> Result<(), DisplayError> {
        self.use_full_frame()?;

        // TODO: allow non-white background color
        let color = 0x00;

        self.interface.cmd(Cmd::WRITE_REDRAM)?;
        self.interface
            .data_x_times(color, u32::from(WIDTH) / 8 * u32::from(HEIGHT))?;
        Ok(())
    }

    fn use_full_frame(&mut self) -> Result<(), DisplayError> {
        // choose full frame/ram
        self.set_ram_area(0, 0, u32::from(WIDTH) - 1, u32::from(HEIGHT) - 1)?;
        // start from the beginning
        self.set_ram_counter(0, 0)
    }

    fn set_ram_area(
        &mut self,
        start_x: u32,
        start_y: u32,
        end_x: u32,
        end_y: u32,
    ) -> Result<(), DisplayError> {
        assert!(start_x < end_x);
        assert!(start_y < end_y);

        self.interface.cmd_with_data(
            Cmd::SET_RAMXPOS,
            &[(start_x >> 3) as u8, (end_x >> 3) as u8],
        )?;
        self.interface.cmd_with_data(
            Cmd::SET_RAMYPOS,
            &[
                start_y as u8,
                (start_y >> 8) as u8,
                end_y as u8,
                (end_y >> 8) as u8,
            ],
        )?;
        Ok(())
    }

    fn set_ram_counter(&mut self, x: u32, y: u32) -> Result<(), DisplayError> {
        // x is positioned in bytes, so the last 3 bits which show the position inside a byte in the ram
        // aren't relevant
        self.interface
            .cmd_with_data(Cmd::SET_RAMXCOUNT, &[(x >> 3) as u8])?;
        // 2 Databytes: A[7:0] & 0..A[8]
        self.interface
            .cmd_with_data(Cmd::SET_RAMYCOUNT, &[y as u8, (y >> 8) as u8])?;
        Ok(())
    }

    // pub fn wake_up<DELAY: DelayMs<u8>>(
    //     &mut self,
    //     spi: &mut SPI,
    //     delay: &mut DELAY,
    // ) -> Result<(), SPI::Error> {
    //     todo!()
    // }
}
