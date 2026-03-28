/// Raw accelerometer reading from the SC7A20H (12-bit, ±2 G scale by default).
/// At ±2 G with 12-bit resolution, 1 LSB ≈ 1 mg.
#[derive(Clone, Copy, Debug, Default)]
pub struct AccelReading {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

impl AccelReading {
    /// Squared vector magnitude — avoids a square-root and stays in integer arithmetic.
    #[inline]
    pub fn magnitude_sq(self) -> i64 {
        (self.x as i64).pow(2) + (self.y as i64).pow(2) + (self.z as i64).pow(2)
    }
}

/// Detects motion by comparing the squared magnitude of consecutive accelerometer
/// readings.  No floating-point required, so it compiles for `no_std` targets too.
pub struct MotionDetector {
    prev_magnitude_sq: i64,
    /// Squared threshold — set via `MotionDetector::new(threshold)`.
    threshold_sq: i64,
}

impl MotionDetector {
    /// `threshold` is expressed in the same raw units as the individual axis values.
    /// For the SC7A20H in ±2 G / 12-bit mode (1 LSB ≈ 1 mg), a threshold of 100
    /// corresponds to roughly 0.1 g of acceleration change.
    pub fn new(threshold: i16) -> Self {
        Self {
            prev_magnitude_sq: 0,
            threshold_sq: (threshold as i64).pow(2),
        }
    }

    /// Feed the latest reading.  Returns `true` when motion is detected.
    pub fn update(&mut self, reading: AccelReading) -> bool {
        let mag_sq = reading.magnitude_sq();
        let delta_sq = (mag_sq - self.prev_magnitude_sq).unsigned_abs();
        self.prev_magnitude_sq = mag_sq;
        (delta_sq as i64) > self.threshold_sq
    }

    /// Reset the detector's baseline (useful after a known disturbance).
    pub fn reset(&mut self) {
        self.prev_magnitude_sq = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn still_reading_no_motion() {
        let mut det = MotionDetector::new(50);
        let flat = AccelReading { x: 0, y: 0, z: 1000 };
        det.update(flat); // prime the baseline
        assert!(!det.update(flat));
    }

    #[test]
    fn large_delta_triggers_motion() {
        let mut det = MotionDetector::new(50);
        det.update(AccelReading { x: 0, y: 0, z: 1000 });
        assert!(det.update(AccelReading { x: 500, y: 500, z: 1000 }));
    }
}
