use anyhow::bail;
use esp_idf_hal::{
    delay::FreeRtos,
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    units::FromValueType,
};
use esp_idf_sys as _; // pulls in the ESP-IDF link patches
use log::{error, info, warn};
use shared::{AccelReading, MotionDetector};

// ---------------------------------------------------------------------------
// Pin assignments – verify these against the UNIHIKER K10 schematic PDF:
// https://www.unihiker.com/wiki/K10/HardwareReference/hardwarereference_stepschematic/
// ---------------------------------------------------------------------------

/// Internal I2C bus shared by SC7A20H, AHT20, and LTR303ALS.
const PIN_I2C_SDA: i32 = 8;
const PIN_I2C_SCL: i32 = 9;

// ---------------------------------------------------------------------------
// SC7A20H triaxial accelerometer (SILAN) – register map compatible with LIS3DH
// ---------------------------------------------------------------------------

const SC7A20H_ADDR: u8 = 0x19; // SA0 pin pulled high on K10

const REG_WHO_AM_I: u8 = 0x0F;
const REG_CTRL1: u8 = 0x20;
const REG_CTRL4: u8 = 0x23;
const REG_OUT_X_L: u8 = 0x28;

/// CTRL_REG1: ODR=100 Hz, normal power, all axes enabled.
/// Bits: ODR[3:0]=0101, LPen=0, Zen=1, Yen=1, Xen=1  → 0x57
const CTRL1_100HZ_ALL_AXES: u8 = 0x57;

/// CTRL_REG4: BDU=1 (block-data update), FS=±2 G, HR=1 (12-bit).
/// Bits: BDU=1, BLE=0, FS[1:0]=00, HR=1, ST[1:0]=00, SIM=0  → 0x88
const CTRL4_BDU_HR_2G: u8 = 0x88;

const WHO_AM_I_EXPECTED: u8 = 0x11;

/// I2C timeout for all transactions (ms).
const I2C_TIMEOUT_MS: u32 = 50;

// ---------------------------------------------------------------------------
// Motion-detection tuning
// ---------------------------------------------------------------------------

/// Motion threshold in raw LSB units.
/// At ±2 G / 12-bit, 1 LSB ≈ 1 mg.  100 LSB ≈ 0.1 g – a gentle tap.
const MOTION_THRESHOLD: i16 = 100;

// ---------------------------------------------------------------------------
// SC7A20H driver
// ---------------------------------------------------------------------------

struct Sc7a20h<'d> {
    i2c: I2cDriver<'d>,
    addr: u8,
}

impl<'d> Sc7a20h<'d> {
    fn new(i2c: I2cDriver<'d>, addr: u8) -> Self {
        Self { i2c, addr }
    }

    fn write_reg(&mut self, reg: u8, val: u8) -> anyhow::Result<()> {
        self.i2c
            .write(self.addr, &[reg, val], I2C_TIMEOUT_MS)
            .map_err(|e| anyhow::anyhow!("I2C write to 0x{:02X} reg 0x{:02X}: {:?}", self.addr, reg, e))
    }

    fn write_read(&mut self, reg: u8, buf: &mut [u8]) -> anyhow::Result<()> {
        self.i2c
            .write_read(self.addr, &[reg], buf, I2C_TIMEOUT_MS)
            .map_err(|e| anyhow::anyhow!("I2C write_read 0x{:02X} reg 0x{:02X}: {:?}", self.addr, reg, e))
    }

    fn init(&mut self) -> anyhow::Result<()> {
        let mut id = [0u8; 1];
        self.write_read(REG_WHO_AM_I, &mut id)?;
        if id[0] != WHO_AM_I_EXPECTED {
            bail!(
                "SC7A20H WHO_AM_I mismatch: got 0x{:02X}, expected 0x{:02X}. \
                 Check I2C wiring (SDA=GPIO{}, SCL=GPIO{}) and I2C address (0x{:02X}).",
                id[0], WHO_AM_I_EXPECTED, PIN_I2C_SDA, PIN_I2C_SCL, self.addr
            );
        }
        self.write_reg(REG_CTRL1, CTRL1_100HZ_ALL_AXES)?;
        self.write_reg(REG_CTRL4, CTRL4_BDU_HR_2G)?;
        info!("SC7A20H initialised (WHO_AM_I=0x{:02X})", id[0]);
        Ok(())
    }

    /// Read a single 12-bit (high-resolution) sample from all three axes.
    fn read(&mut self) -> anyhow::Result<AccelReading> {
        // Setting MSB of the sub-address enables auto-increment over 6 consecutive
        // output registers (OUT_X_L … OUT_Z_H).
        let mut buf = [0u8; 6];
        self.write_read(REG_OUT_X_L | 0x80, &mut buf)?;

        // Each axis is a 16-bit little-endian value, left-justified at 12 bits.
        // Right-shift by 4 to obtain the signed 12-bit sample.
        let x = i16::from_le_bytes([buf[0], buf[1]]) >> 4;
        let y = i16::from_le_bytes([buf[2], buf[3]]) >> 4;
        let z = i16::from_le_bytes([buf[4], buf[5]]) >> 4;

        Ok(AccelReading { x, y, z })
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> anyhow::Result<()> {
    // Required by esp-idf-sys to apply Rust ↔ C shims.
    esp_idf_sys::link_patches();

    // Route `log` macros to ESP-IDF's logging subsystem (visible over USB-CDC).
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("=== UNIHIKER K10 – Accelerometer Motion Detection ===");
    info!("Chip: ESP32-S3N16R8  |  Sensor: SC7A20H (±2 G, 12-bit, 100 Hz)");

    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;

    // I2C @ 400 kHz (fast-mode) on the internal bus used by on-board sensors.
    let i2c_config = I2cConfig::new().baudrate(400_u32.kHz().into());
    let i2c = I2cDriver::new(
        peripherals.i2c0,
        pins.gpio8, // SDA – verify with schematic
        pins.gpio9, // SCL – verify with schematic
        &i2c_config,
    )?;

    let mut accel = Sc7a20h::new(i2c, SC7A20H_ADDR);

    // Retry init briefly to allow the sensor to power-up after boot.
    let mut init_ok = false;
    for attempt in 1..=5 {
        match accel.init() {
            Ok(()) => {
                init_ok = true;
                break;
            }
            Err(e) => {
                warn!("SC7A20H init attempt {}/5 failed: {}", attempt, e);
                FreeRtos::delay_ms(200);
            }
        }
    }
    if !init_ok {
        error!(
            "Could not initialise SC7A20H after 5 attempts.\n\
             → Check SDA/SCL pin constants (currently GPIO{}/GPIO{}) against the schematic.\n\
             → Download schematic PDF from: \
               https://www.unihiker.com/wiki/K10/HardwareReference/hardwarereference_stepschematic/",
            PIN_I2C_SDA, PIN_I2C_SCL
        );
        // Keep running so the log message stays visible over the monitor.
        loop {
            FreeRtos::delay_ms(5000);
        }
    }

    let mut detector = MotionDetector::new(MOTION_THRESHOLD);
    let mut motion_count: u32 = 0;

    info!("Listening for motion  (threshold = {} LSB ≈ {} mg)…", MOTION_THRESHOLD, MOTION_THRESHOLD);

    loop {
        match accel.read() {
            Ok(sample) => {
                if detector.update(sample) {
                    motion_count += 1;
                    info!(
                        "MOTION #{:04}  x={:5}  y={:5}  z={:5}",
                        motion_count, sample.x, sample.y, sample.z
                    );
                } else {
                    // Uncomment for verbose output during debugging:
                    // info!("Still        x={:5}  y={:5}  z={:5}", sample.x, sample.y, sample.z);
                }
            }
            Err(e) => {
                error!("Accelerometer read error: {}", e);
            }
        }

        // Sample period matches the sensor's 100 Hz ODR.
        FreeRtos::delay_ms(10);
    }
}
