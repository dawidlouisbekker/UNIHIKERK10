#!/usr/bin/env python3
"""
MCP server for UNIHIKER K10 schematic queries.
PDF source: .claude/.docs/UnihikerK10Schematic.pdf
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any

from mcp.server.fastmcp import FastMCP

# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------

_HERE = Path(__file__).parent
PDF_PATH = _HERE.parent / ".docs" / "UnihikerK10Schematic.pdf"

# ---------------------------------------------------------------------------
# PDF extraction
# ---------------------------------------------------------------------------


def _extract_text(pdf_path: Path) -> str:
    """Extract all text from PDF pages. Uses pdfplumber, falls back to pypdf."""
    try:
        import pdfplumber

        pages: list[str] = []
        with pdfplumber.open(str(pdf_path)) as pdf:
            for i, page in enumerate(pdf.pages):
                try:
                    text = page.extract_text() or ""
                except Exception as e:
                    print(
                        f"[schematic_server] WARNING: page {i + 1} extraction failed: {e}",
                        file=sys.stderr,
                    )
                    text = ""
                pages.append(f"=== PAGE {i + 1} ===\n{text}")
        return "\n".join(pages)
    except ImportError:
        pass

    try:
        from pypdf import PdfReader

        reader = PdfReader(str(pdf_path))
        return "\n".join(
            f"=== PAGE {i + 1} ===\n{page.extract_text()}"
            for i, page in enumerate(reader.pages)
        )
    except ImportError:
        return "[PDF extraction failed: install pdfplumber or pypdf]"


def _build_chips_from_known_data() -> dict[str, dict[str, Any]]:
    """
    Returns the base chip dict populated from schematic analysis and hw.rs constants.

    GPIO values are sourced from kernel/src/hw.rs and the component datasheets.
    """
    return {
        "U4": {
            "ref": "U4",
            "part": "ESP32-S3-WROOM-1",
            "description": "Main MCU (ESP32-S3N16R8, 16 MB Flash, 8 MB PSRAM)",
            "interface": "N/A",
            "i2c_address": None,
            "gpio_pins": {
                "I2C_SDA": 47,
                "I2C_SCL": 45,
                "LCD_MOSI": 41,
                "LCD_SCLK": 40,
                "LCD_CS": 39,
                "LCD_DC": 38,
                "LCD_RST": 37,
                "LCD_BL": 36,
            },
            "notes": (
                "I2C bus: SDA=GPIO47, SCL=GPIO45. "
                "SPI LCD: MOSI=41, SCLK=40, CS=39, DC=38, RST=37, BL=36. "
                "UART: TXD0=GPIO43, RXD0=GPIO44."
            ),
        },
        "U1": {
            "ref": "U1",
            "part": "AHT20",
            "description": "Temperature and Humidity Sensor",
            "interface": "I2C",
            "i2c_address": "0x38",
            "gpio_pins": {"SDA": 47, "SCL": 45},
            "notes": "Fixed I2C address 0x38. No address configuration pins.",
        },
        "U2": {
            "ref": "U2",
            "part": "LTR-303ALS-01",
            "description": "Ambient Light Sensor",
            "interface": "I2C",
            "i2c_address": "0x29",
            "gpio_pins": {"SDA": 47, "SCL": 45},
            "notes": "Fixed I2C address 0x29. INT pin connected to XL9535 expander.",
        },
        "U3": {
            "ref": "U3",
            "part": "SC7A20H",
            "description": "3-Axis Accelerometer (SILAN, LIS3DH-compatible register map)",
            "interface": "I2C",
            "i2c_address": "0x19",
            "gpio_pins": {"SDA": 47, "SCL": 45, "INT1": None, "INT2": None},
            "notes": (
                "SA0 pin pulled HIGH on K10 board → address 0x19. "
                "WHO_AM_I register 0x0F returns 0x11. "
                "LIS3DH-compatible: CTRL_REG1=0x20, CTRL_REG4=0x23, OUT_X_L=0x28."
            ),
        },
        "U5": {
            "ref": "U5",
            "part": "XL9535QF24",
            "description": "16-bit I2C GPIO Expander",
            "interface": "I2C",
            "i2c_address": "0x20",
            "gpio_pins": {"SDA": 47, "SCL": 45},
            "notes": (
                "Provides 16 additional GPIO pins (P00-P07, P10-P17). "
                "Address 0x20 — A0/A1/A2 all grounded. "
                "Controls: RGB LEDs, user LED, KeyA, KeyB, camera power."
            ),
        },
        "U8": {
            "ref": "U8",
            "part": "ES7243E",
            "description": "Audio ADC / Codec",
            "interface": "I2C+I2S",
            "i2c_address": "0x15",
            "gpio_pins": {"SDA": 47, "SCL": 45},
            "notes": (
                "I2C address 0x15 (AD2 pulled up). "
                "I2S signals: I2S_BLCK, I2S_LRCK, I2S_MCLK, I2S_SDI/SDO. "
                "Analog inputs: AINLP, AINLN, AINRP, AINRN (microphone)."
            ),
        },
        "U11": {
            "ref": "U11",
            "part": "NS4168",
            "description": "Mono Class-D Audio Amplifier",
            "interface": "I2S",
            "i2c_address": None,
            "gpio_pins": {},
            "notes": (
                "I2S audio input (BCLK, LRCLK, SDATA). "
                "Speaker outputs: SPK+, SPK-. "
                "Output coupling capacitors C25, C26 (22µF/6.3V). "
                "No I2C control interface."
            ),
        },
        "M1": {
            "ref": "M1",
            "part": "GC2145",
            "description": "2MP Camera Module",
            "interface": "DVP/SCCB",
            "i2c_address": None,
            "gpio_pins": {},
            "notes": (
                "DVP parallel 8-bit interface: Camera_D2–Camera_D9, "
                "Camera_VSYNC, Camera_HREF, Camera_PCLK, Camera_XCLK. "
                "SCCB control (I2C-compatible): SCL=P19, SDA=P20. "
                "Reset: Camera_RST. Power: DVDD(1V2), DOVDD(1V8), AVDD(2V8)."
            ),
        },
        "M2": {
            "ref": "M2",
            "part": "SD Card",
            "description": "MicroSD Card Slot",
            "interface": "SPI",
            "i2c_address": None,
            "gpio_pins": {
                "CS": None,
                "MOSI": None,
                "SCLK": None,
                "MISO": None,
            },
            "notes": "SPI mode (CS3, MOSI3, SCLK3, MISO3). Card detect pin (CD).",
        },
        "U15": {
            "ref": "U15",
            "part": "GT30L24A3W",
            "description": "Serial Font ROM (GB2312 + ASCII bitmaps)",
            "interface": "SPI",
            "i2c_address": None,
            "gpio_pins": {
                "CS": None,
                "SDI": None,
                "SDO": None,
                "SCLK": None,
            },
            "notes": (
                "Shares SPI3 bus with SD card (CS3, MOSI3, MISO3, SCLK3). "
                "Chip select controlled via Q1 (MMBT3904T transistor). "
                "Used for Chinese character font rendering."
            ),
        },
        "LCD": {
            "ref": "LCD",
            "part": "ILI9341",
            "description": "2.8-inch TFT LCD Controller (320×240, RGB565)",
            "interface": "SPI",
            "i2c_address": None,
            "gpio_pins": {
                "MOSI": 41,
                "SCLK": 40,
                "CS": 39,
                "DC": 38,
                "RST": 37,
                "BL": 36,
            },
            "notes": (
                "Dedicated SPI bus (not shared). "
                "BL=GPIO36 (backlight on/off or PWM). "
                "Connector J3."
            ),
        },
    }


def _enrich_from_text(
    chips: dict[str, dict[str, Any]], full_text: str
) -> dict[str, dict[str, Any]]:
    """Scan raw PDF text for GPIO numbers and I2C addresses not yet in the base data."""
    gpio_re = re.compile(r"GPIO\s*(\d+)", re.IGNORECASE)
    i2c_re = re.compile(r"0x[0-9A-Fa-f]{2}", re.IGNORECASE)

    for chip in chips.values():
        # Search for lines mentioning the chip's part name or ref
        needle = chip["part"].lower()
        ref = chip["ref"].lower()
        relevant_lines: list[str] = []
        for line in full_text.splitlines():
            ll = line.lower()
            if needle in ll or ref in ll:
                relevant_lines.append(line)

        # Extract any GPIO numbers mentioned near this chip
        found_gpios = set()
        for line in relevant_lines:
            for m in gpio_re.finditer(line):
                found_gpios.add(int(m.group(1)))

        # Add newly discovered GPIOs to notes (avoid polluting gpio_pins dict)
        if found_gpios:
            existing = set(chip["gpio_pins"].values()) - {None}
            new_gpios = found_gpios - existing
            if new_gpios:
                extra = f" PDF-mentioned GPIOs: {sorted(new_gpios)}."
                chip["notes"] = chip.get("notes", "") + extra

    return chips


def parse_pdf(pdf_path: Path = PDF_PATH) -> dict[str, Any]:
    """
    Build the full schematic data dict.
    Uses PDF text to enrich the hardcoded base data.
    """
    if pdf_path.exists():
        full_text = _extract_text(pdf_path)
    else:
        print(
            f"[schematic_server] WARNING: PDF not found at {pdf_path}",
            file=sys.stderr,
        )
        full_text = ""

    chips = _build_chips_from_known_data()
    if full_text:
        chips = _enrich_from_text(chips, full_text)

    i2c_devices = [c for c in chips.values() if c.get("i2c_address")]
    spi_devices = [c for c in chips.values() if "SPI" in c.get("interface", "")]

    return {
        "chips": chips,
        "i2c_devices": i2c_devices,
        "spi_devices": spi_devices,
        "full_text": full_text,
    }


# ---------------------------------------------------------------------------
# Module-level cache — populated once on import
# ---------------------------------------------------------------------------

SCHEMATIC_DATA: dict[str, Any] = parse_pdf()

# ---------------------------------------------------------------------------
# MCP server
# ---------------------------------------------------------------------------

mcp = FastMCP(
    name="unihiker-k10-schematic",
    instructions=(
        "Query UNIHIKER K10 hardware schematic: chips, GPIO pins, "
        "I2C/SPI devices, and raw PDF text. "
        "Data is sourced from UnihikerK10Schematic.pdf and kernel/src/hw.rs."
    ),
)


@mcp.tool()
def list_chips() -> list[dict[str, str]]:
    """List all ICs and components in the schematic with their reference designators and part numbers."""
    return [
        {
            "ref": c["ref"],
            "part": c["part"],
            "description": c["description"],
            "interface": c["interface"],
        }
        for c in SCHEMATIC_DATA["chips"].values()
    ]


@mcp.tool()
def get_chip_info(chip_id: str) -> dict[str, Any]:
    """
    Get detailed information about a specific chip.

    Args:
        chip_id: Reference designator (e.g. "U3") or part name/keyword (e.g. "SC7A20H", "accelerometer")
    """
    chips = SCHEMATIC_DATA["chips"]
    upper = chip_id.upper()
    if upper in chips:
        return chips[upper]

    needle = chip_id.lower()
    for chip in chips.values():
        if (
            needle in chip["part"].lower()
            or needle in chip["description"].lower()
            or needle in chip["ref"].lower()
        ):
            return chip

    return {
        "error": f"Chip '{chip_id}' not found.",
        "available_refs": list(chips.keys()),
    }


@mcp.tool()
def get_gpio_pins(device_name: str) -> dict[str, Any]:
    """
    Get GPIO pin assignments for a device.

    Args:
        device_name: Device ref (e.g. "LCD", "U3") or part name (e.g. "ILI9341", "SC7A20H")
    """
    info = get_chip_info(device_name)
    if "error" in info:
        return info
    return {
        "ref": info["ref"],
        "part": info["part"],
        "interface": info["interface"],
        "gpio_pins": info["gpio_pins"],
        "notes": info.get("notes", ""),
    }


@mcp.tool()
def get_i2c_devices() -> list[dict[str, Any]]:
    """List all I2C devices with their addresses and SDA/SCL GPIO assignments."""
    result = []
    for c in SCHEMATIC_DATA["chips"].values():
        if not c.get("i2c_address"):
            continue
        pins = c["gpio_pins"]
        result.append(
            {
                "ref": c["ref"],
                "part": c["part"],
                "description": c["description"],
                "i2c_address": c["i2c_address"],
                "sda_gpio": pins.get("SDA") or pins.get("I2C_SDA"),
                "scl_gpio": pins.get("SCL") or pins.get("I2C_SCL"),
            }
        )
    return result


@mcp.tool()
def get_spi_devices() -> list[dict[str, Any]]:
    """List all SPI devices with their GPIO pin assignments."""
    return [
        {
            "ref": c["ref"],
            "part": c["part"],
            "description": c["description"],
            "gpio_pins": c["gpio_pins"],
            "notes": c.get("notes", ""),
        }
        for c in SCHEMATIC_DATA["chips"].values()
        if "SPI" in c.get("interface", "")
    ]


@mcp.tool()
def search_schematic(query: str) -> dict[str, Any]:
    """
    Full-text search across all schematic data including raw PDF text.

    Args:
        query: Search term (case-insensitive). E.g. "GPIO47", "0x38", "I2C", "accelerometer"
    """
    needle = query.lower()

    matched_chips: list[dict[str, str]] = []
    for chip in SCHEMATIC_DATA["chips"].values():
        blob = json.dumps(chip).lower()
        if needle in blob:
            matched_chips.append({"ref": chip["ref"], "part": chip["part"]})

    pdf_matches: list[dict[str, Any]] = []
    full_text = SCHEMATIC_DATA.get("full_text", "")
    lines = full_text.splitlines()
    for i, line in enumerate(lines):
        if needle in line.lower():
            context_lines = lines[max(0, i - 1) : i + 2]
            pdf_matches.append(
                {
                    "line": i + 1,
                    "text": line.strip(),
                    "context": " | ".join(l.strip() for l in context_lines),
                }
            )

    return {
        "query": query,
        "matched_chips": matched_chips,
        "pdf_text_matches": pdf_matches,
        "total_pdf_matches": len(pdf_matches),
    }


@mcp.tool()
def reparse_pdf() -> dict[str, Any]:
    """
    Re-parse the PDF file and refresh the in-memory schematic cache.
    Use this if the PDF has been updated or the cache seems stale.
    """
    global SCHEMATIC_DATA
    try:
        SCHEMATIC_DATA = parse_pdf()
        return {
            "status": "ok",
            "chips_found": len(SCHEMATIC_DATA["chips"]),
            "i2c_devices": len(SCHEMATIC_DATA["i2c_devices"]),
            "spi_devices": len(SCHEMATIC_DATA["spi_devices"]),
            "pdf_text_length": len(SCHEMATIC_DATA["full_text"]),
            "pdf_path": str(PDF_PATH),
        }
    except Exception as exc:
        return {"status": "error", "message": str(exc)}


@mcp.resource("schematic://full")
def get_full_schematic() -> str:
    """Full structured chip data as JSON plus the raw extracted PDF text."""
    structured = json.dumps(
        {k: v for k, v in SCHEMATIC_DATA.items() if k != "full_text"},
        indent=2,
    )
    return (
        f"=== STRUCTURED SCHEMATIC DATA ===\n{structured}\n\n"
        f"=== RAW PDF TEXT ===\n{SCHEMATIC_DATA.get('full_text', '')}"
    )


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    mcp.run(transport="stdio")