// ---------------------------------------------------------------------------
// Hardware constants – UNIHIKER K10 (ESP32-S3N16R8)
// Verify against the schematic:
// https://www.unihiker.com/wiki/K10/HardwareReference/hardwarereference_stepschematic/
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// I2C bus (internal sensors) – page 1 & 10 of schematic
// ---------------------------------------------------------------------------

/// Internal I2C SDA – P20 = GPIO47.
pub const PIN_I2C_SDA: i32 = 47;
/// Internal I2C SCL – P19 = GPIO45.
pub const PIN_I2C_SCL: i32 = 45;

/// I2C transaction timeout (ms).
pub const I2C_TIMEOUT_MS: u32 = 50;

// ---------------------------------------------------------------------------
// ILI9341 SPI LCD
// ---------------------------------------------------------------------------

pub const PIN_LCD_MOSI: i32 = 41;
pub const PIN_LCD_SCLK: i32 = 40;
pub const PIN_LCD_CS:   i32 = 39;
pub const PIN_LCD_DC:   i32 = 38;
pub const PIN_LCD_RST:  i32 = 37;
pub const PIN_LCD_BL:   i32 = 36;

// ---------------------------------------------------------------------------
// SC7A20H triaxial accelerometer (SILAN) – LIS3DH-compatible register map
// ---------------------------------------------------------------------------

/// I2C address: SA0 pulled high on the K10 board.
pub const SC7A20H_ADDR: u8 = 0x19;

pub const REG_WHO_AM_I: u8 = 0x0F;
pub const REG_CTRL1:    u8 = 0x20;
pub const REG_CTRL4:    u8 = 0x23;
/// Base address of the six output data registers (OUT_X_L … OUT_Z_H).
/// OR with 0x80 to enable the auto-increment burst read.
pub const REG_OUT_X_L:  u8 = 0x28;

/// CTRL_REG1: ODR=100 Hz, normal power, all axes enabled.
/// Bits: ODR[3:0]=0101, LPen=0, Zen=1, Yen=1, Xen=1  → 0x57
pub const CTRL1_100HZ_ALL_AXES: u8 = 0x57;

/// CTRL_REG4: BDU=1 (block-data update), FS=±2 G, HR=1 (12-bit).
/// Bits: BDU=1, BLE=0, FS[1:0]=00, HR=1, ST[1:0]=00, SIM=0  → 0x88
pub const CTRL4_BDU_HR_2G: u8 = 0x88;

/// Expected response to a WHO_AM_I query.
pub const WHO_AM_I_EXPECTED: u8 = 0x11;
