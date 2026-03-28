# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Bare-metal Rust kernel for the **UNIHIKER K10** board (ESP32-S3 N16R8, Xtensa LX7 dual-core). Targets the `xtensa-esp32s3-espidf` platform using the ESP-IDF framework via Rust bindings (`esp-idf-hal`, `esp-idf-svc`, `esp-idf-sys` with ESP-IDF v5.3.1).

## Build & Flash Commands

```bash
# Build only
cargo build -p kernel --release

# Build + flash + serial monitor (device connected via USB-C)
cargo run -p kernel --release

# Run shared library tests (host, not on device)
cargo test -p shared
```

The build uses Rust nightly with `build-std = ["std", "panic_abort"]`. Output goes to `./out/` (configured in `.cargo/config.toml`). The runner is `espflash flash --monitor`.

## Architecture

**Workspace layout:**
- `kernel/` — firmware binary that runs on the K10. Entry point: `kernel/src/main.rs`
- `shared/` — platform-independent library (`no_std`-compatible). Contains data types (`AccelReading`) and algorithms (`MotionDetector`) shared between kernel and potential host-side tooling

**Kernel internals:**
- `kernel/src/hw.rs` — all hardware constants (GPIO pin numbers, I2C addresses, register maps). Single source of truth for pin assignments; reference this before using any GPIO
- `kernel/src/dev/` — device drivers. Each device gets its own module:
  - `screen.rs` — ILI9341 SPI LCD driver (240x320, via `mipidsi` crate). Provides a rolling-log `println()` interface
  - `accelerometer.rs` — SC7A20H I2C accelerometer driver (currently disabled due to esp-idf-hal I2C driver incompatibility with ESP-IDF v5.2+)
- `kernel/src/dev/mod.rs` — `dev::init()` sets up ESP-IDF logging

## Hardware Reference

The K10 schematic PDF is at `.claude/.docs/UnihikerK10Schematic.pdf`. Key on-board peripherals not yet driven: AHT20 (temp/humidity), LTR303ALS (ambient light), GC2145 (camera), WS2812 RGB LEDs x3, MEMS microphones x2, speaker, buttons (A/B/RST/BOOT), SD card slot.

## Known Issues

- **I2C is disabled**: `esp-idf-hal` 0.46 uses the legacy `i2c_driver_install` API which crashes on ESP-IDF v5.2+ due to an interrupt allocation bug. The accelerometer driver exists but is not wired up in `main.rs`. Re-enable once the HAL updates to the new `i2c_master` driver.
