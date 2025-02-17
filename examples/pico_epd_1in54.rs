//! Simple no-std "Hello World" example for the Raspberry Pi Pico microcontroller board
//! with Dalian Good Display Tri-Color e-ink display 1.54 inch e-ink small display screen, [GDEM0154Z90](https://www.good-display.com/product/436.html)
//! using DESPI-C02 adapter https://buyepaper.com/products/development-kit-connection-adapter-board-for-eaper-display-demo-kit
//!
//! Connections:
//!
//! | Pico | DESPI-C02   |
//! |--------|-------|
//! | GP18   | SCK   |
//! | GP19   | MOSI  |
//! | GP17   | CS    |
//! | GP13   | BUSY  |
//! | GP12   | DC    |
//! | GP11   | RESET |
//!
//! To run this example clone this repository and run:
//! `cargo run --example epd_154

#![no_std]
#![no_main]
#[cfg(target_os = "none")]
use defmt::{info, println};
#[cfg(target_os = "none")]
use defmt_rtt as _;
use embedded_graphics::geometry::Point;
use embedded_graphics::mono_font::ascii::FONT_6X9;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::prelude::Primitive;
use embedded_graphics::primitives::{Circle, PrimitiveStyle, Rectangle};
use embedded_graphics::text::Text;
use embedded_graphics_core::geometry::Size;
use embedded_graphics_core::Drawable;
#[cfg(target_os = "none")]
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::StatefulOutputPin;
use embedded_hal_bus::spi::ExclusiveDevice;
use epd_ssd1681::color::TriColor::{Black, Red, White};
use epd_ssd1681::driver::Ssd1681;
use epd_ssd1681::graphics::{Display, DisplayRotation};
#[cfg(target_os = "none")]
use panic_probe as _;
#[cfg(target_os = "none")]
use rp_pico as bsp;
#[cfg(target_os = "none")]
use rp_pico::hal::clocks::init_clocks_and_plls;
#[cfg(target_os = "none")]
use rp_pico::hal::fugit::RateExtU32;
#[cfg(target_os = "none")]
use rp_pico::hal::gpio::FunctionSpi;
#[cfg(target_os = "none")]
use rp_pico::hal::{spi, Clock, Sio, Watchdog};
#[cfg(target_os = "none")]
use rp_pico::{entry, pac};

#[cfg(target_os = "none")]
#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = DelayCompat(cortex_m::delay::Delay::new(
        core.SYST,
        clocks.system_clock.freq().to_Hz(),
    ));

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut led_pin = pins.led.into_push_pull_output();

    // These are implicitly used by the spi driver if they are in the correct mode
    let sck = pins.gpio18.into_function::<FunctionSpi>(); // SCK
    let mosi = pins.gpio19.into_function::<FunctionSpi>(); // SCL TX
    let miso = pins.gpio16.into_function::<FunctionSpi>(); // SDA RX
    let cs0 = pins.gpio21.into_push_pull_output();

    let cs = pins.gpio17.into_push_pull_output();
    let dc = pins.gpio12.into_push_pull_output();
    let rst = pins.gpio11.into_push_pull_output();
    let busy = pins.gpio13.into_pull_down_input();

    // Create an SPI driver instance for the SPI0 device
    let spi = spi::Spi::<_, _, _, 8>::new(pac.SPI0, (mosi, miso, sck)).init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        16_000_000u32.Hz(),
        embedded_hal::spi::MODE_0,
    );

    let mut spi_device = ExclusiveDevice::new_no_delay(spi, cs0).unwrap();

    // Initialize display controller
    println!("Initialize display controller");
    let mut ssd1681 = Ssd1681::new(&mut spi_device, cs, busy, dc, rst, &mut delay).unwrap();

    // Clear frames on the display driver
    println!("Clear bw frame to display");
    ssd1681.clear_bw_frame();
    println!("Clear red frame to display");
    ssd1681.clear_red_frame();
    println!("Update display");
    ssd1681.display_frame(&mut delay);

    // Create buffer for black and white
    let mut display = Display::new();
    display.clear(White);

    let style = MonoTextStyleBuilder::new()
        .font(&FONT_6X9)
        .text_color(Black)
        .background_color(White)
        .build();

    Text::new(
        "This is a\nmultiline\nHello World!",
        Point::new(15, 15),
        style,
    )
    .draw(&mut display);

    display.set_rotation(DisplayRotation::Rotate0);
    Rectangle::new(Point::new(50, 50), Size::new(50, 50))
        .into_styled(PrimitiveStyle::with_fill(Red))
        .draw(&mut display)
        .unwrap();

    Circle::new(Point::new(100, 100), 20)
        .into_styled(PrimitiveStyle::with_fill(Red))
        .draw(&mut display)
        .unwrap();

    println!("Send bw frame to display");
    ssd1681.update_bw_frame(display.black_data());
    println!("Send red frame to display");
    ssd1681.update_red_frame(display.red_data());

    println!("Update display");
    ssd1681.display_frame(&mut delay);

    println!("Done");

    loop {
        let _ = led_pin.toggle();
        delay.delay_ms(500);
    }
}

/// Wrapper around `Delay` to implement the embedded-hal 1.0 delay.
///
/// This can be removed when a new version of the `cortex_m` crate is released.
#[cfg(target_os = "none")]
struct DelayCompat(cortex_m::delay::Delay);

#[cfg(target_os = "none")]
impl embedded_hal::delay::DelayNs for DelayCompat {
    fn delay_ns(&mut self, mut ns: u32) {
        while ns > 1000 {
            self.0.delay_us(1);
            ns = ns.saturating_sub(1000);
        }
    }

    fn delay_us(&mut self, us: u32) {
        self.0.delay_us(us);
    }
}
