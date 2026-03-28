mod dev;
mod hw;

use crate::dev::screen::Screen;

use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{AnyIOPin, PinDriver},
    peripherals::Peripherals,
    spi::{config as spi_config, SpiDeviceDriver},
    units::FromValueType,
};
use esp_idf_sys as _; // pulls in the ESP-IDF link patches
use log::info;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    // Sets up ESP-IDF logging for the `log` crate.
    dev::init();

    info!("=== UNIHIKER K10 ===");

    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;

    // -----------------------------------------------------------------------
    // ILI9341 SPI display
    // GPIO36=BL  GPIO37=RST  GPIO38=DC  GPIO39=CS  GPIO40=SCLK  GPIO41=MOSI
    // -----------------------------------------------------------------------
    let spi = SpiDeviceDriver::new_single(
        peripherals.spi2,
        pins.gpio40,                    // SCLK
        pins.gpio41,                    // MOSI
        None::<AnyIOPin<'_>>,           // no MISO needed
        Some(pins.gpio39),              // CS
        &spi_config::DriverConfig::new(),
        &spi_config::Config::new().baudrate(10u32.MHz().into()),
    )?;
    let dc  = PinDriver::output(pins.gpio38)?;
    let rst = PinDriver::output(pins.gpio37)?;
    let bl  = PinDriver::output(pins.gpio36)?;

    let mut screen = Screen::new(spi, dc, rst, bl)?;
    info!("ILI9341 display initialised");

    screen.println("UNIHIKER K10")?;
    screen.println("Screen OK")?;

    // I2C / accelerometer disabled: esp-idf-hal 0.46 uses the old i2c_driver_install
    // API which crashes on ESP-IDF v5.2+ due to a bug in interrupt allocation.
    // Re-enable once esp-idf-hal is updated to use the new i2c_master driver.
    screen.println("Accel: disabled")?;
    screen.println("(I2C driver bug)")?;

    info!("Running – screen confirmed OK");
    loop {
        // Interupt handler.
        FreeRtos::delay_ms(5000);
    }
}
