use std::collections::VecDeque;

use anyhow::anyhow;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
};
use esp_idf_hal::{
    gpio::Output,
    gpio::PinDriver,
    spi::{SpiDeviceDriver, SpiDriver},
};
use mipidsi::{interface::SpiInterface, models::ILI9341Rgb565, Builder};

// Pin assignments live in crate::hw (PIN_LCD_MOSI, PIN_LCD_SCLK, etc.).

// ---------------------------------------------------------------------------
// Rolling log constants
// ---------------------------------------------------------------------------

const MAX_LINES: usize = 6;

/// Line height in pixels (FONT_6X10 is 10px tall + 4px gap).
const LINE_HEIGHT: i32 = 14;

/// X offset for text.
const TEXT_X: i32 = 4;

/// Y baseline of the first line.
const TEXT_Y_START: i32 = 12;

// ---------------------------------------------------------------------------
// Screen
// ---------------------------------------------------------------------------

// In esp-idf-hal 0.46 PinDriver<'d, MODE> has only one generic parameter –
// the mode (Output, InputOutput, …).  The pin identity is erased at runtime.
type Display<'d> = mipidsi::Display<
    SpiInterface<'static, SpiDeviceDriver<'d, SpiDriver<'d>>, PinDriver<'d, Output>>,
    ILI9341Rgb565,
    PinDriver<'d, Output>,
>;

pub struct Screen<'d> {
    display: Display<'d>,
    _bl: PinDriver<'d, Output>,
    logs: VecDeque<String>,
}

impl<'d> Screen<'d> {
    /// Initialise the ILI9341 display and turn on the backlight.
    ///
    /// Construct the drivers in main before calling this:
    ///
    /// ```ignore
    /// use esp_idf_hal::{
    ///     gpio::PinDriver,
    ///     spi::{SpiDriver, SpiDriverConfig, SpiDeviceDriver, SpiConfig},
    ///     units::FromValueType,
    /// };
    ///
    /// let spi_driver = SpiDriver::new(
    ///     peripherals.spi2,
    ///     pins.gpio40,            // SCLK
    ///     pins.gpio41,            // MOSI
    ///     None::<esp_idf_hal::gpio::AnyIOPin>,  // no MISO
    ///     &SpiDriverConfig::new(),
    /// )?;
    /// let spi = SpiDeviceDriver::new(
    ///     spi_driver,
    ///     Some(pins.gpio39),      // CS
    ///     &SpiConfig::new().baudrate(40.MHz().into()),
    /// )?;
    /// let dc  = PinDriver::output(pins.gpio38)?;
    /// let rst = PinDriver::output(pins.gpio37)?;
    /// let bl  = PinDriver::output(pins.gpio36)?;
    /// let screen = Screen::new(spi, dc, rst, bl)?;
    /// ```
    pub fn new(
        spi: SpiDeviceDriver<'d, SpiDriver<'d>>,
        dc: PinDriver<'d, Output>,
        rst: PinDriver<'d, Output>,
        mut bl: PinDriver<'d, Output>,
    ) -> anyhow::Result<Self> {
        // Allocate a 512-byte batch buffer and leak it to obtain a 'static
        // reference.  This is intentional: the display is created once and
        // lives for the duration of the program.
        let buf: &'static mut [u8] = vec![0u8; 512].leak();
        let di = SpiInterface::new(spi, dc, buf);

        let display = Builder::new(ILI9341Rgb565, di)
            .reset_pin(rst)
            .init(&mut esp_idf_hal::delay::Ets)
            .map_err(|e| anyhow!("ILI9341 init error: {:?}", e))?;

        bl.set_high().map_err(|e| anyhow!("BL pin error: {:?}", e))?;

        let mut screen = Self {
            display,
            _bl: bl,
            logs: VecDeque::with_capacity(MAX_LINES + 1),
        };

        screen
            .display
            .clear(Rgb565::BLACK)
            .map_err(|e| anyhow!("Display clear error: {:?}", e))?;

        Ok(screen)
    }

    /// Append a log line and redraw.  Lines older than MAX_LINES are dropped.
    pub fn println(&mut self, text: impl Into<String>) -> anyhow::Result<()> {
        self.logs.push_back(text.into());
        if self.logs.len() > MAX_LINES {
            self.logs.pop_front();
        }
        self.render()
    }

    // -----------------------------------------------------------------------
    // Private
    // -----------------------------------------------------------------------

    fn render(&mut self) -> anyhow::Result<()> {
        self.display
            .clear(Rgb565::BLACK)
            .map_err(|e| anyhow!("Display clear error: {:?}", e))?;

        let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

        for (i, line) in self.logs.iter().enumerate() {
            let y = TEXT_Y_START + i as i32 * LINE_HEIGHT;
            Text::new(line, Point::new(TEXT_X, y), style)
                .draw(&mut self.display)
                .map_err(|e| anyhow!("Draw text error: {:?}", e))?;
        }

        Ok(())
    }
}
