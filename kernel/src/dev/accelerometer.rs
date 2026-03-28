use anyhow::bail;
use esp_idf_hal::{
    delay::FreeRtos,
    i2c::{I2cConfig, I2cDriver},
    units::FromValueType,
};
use esp_idf_sys as _; // pulls in the ESP-IDF link patches
use log::{error, info, warn};
use shared::{AccelReading, MotionDetector};



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