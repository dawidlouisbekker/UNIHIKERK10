# USB Access from WSL2 (for espflash)

WSL2 does not have USB access by default. You need to forward the USB port from Windows to WSL using **usbipd-win**.

---

## 1. Install usbipd-win (Windows, run once)

Download and install the latest release from:
https://github.com/dorssel/usbipd-win/releases

Or with winget in an **elevated** PowerShell:
```powershell
winget install usbipd
```

---

## 2. Install usbip tools in WSL (run once)

```bash
sudo apt install linux-tools-generic hwdata
sudo update-alternatives --install /usr/local/bin/usbip usbip /usr/lib/linux-tools/*-generic/usbip 20
```

---

## 3. Forward the USB device (every session)

Plug in your ESP32 board, then in an **elevated** PowerShell (Run as Administrator):

### List connected USB devices
```powershell
usbipd list
```

Find your ESP32 — it will appear as something like `USB-SERIAL CH340`, `CP210x`, or `USB JTAG/serial debug unit`. Note its `BUSID` (e.g. `2-3`).

### Bind the device (first time only, elevated PowerShell)
```powershell
usbipd bind --busid 2-3
```

### Attach to WSL
```powershell
usbipd attach --wsl --busid 2-3
```

---

## 4. Verify the device is visible in WSL

```bash
lsusb
ls /dev/ttyUSB* /dev/ttyACM*
```

You should see a device like `/dev/ttyUSB0` or `/dev/ttyACM0`.

---

## 5. Grant your user permission to access the port (run once)

```bash
sudo usermod -aG dialout $USER
```

Then log out and back in (or `newgrp dialout`) for the group change to take effect.

---

## 6. Flash the chip

```bash
. $HOME/export-esp.sh
cargo run -p kernel --release
```

espflash will auto-detect the port, or you can specify it explicitly:
```bash
espflash flash --monitor --port /dev/ttyUSB0 target/xtensa-esp32s3-espidf/release/kernel
```

---

## Detach when done (optional)

In PowerShell:
```powershell
usbipd detach --busid 2-3
```

---

## Troubleshooting

| Problem | Fix |
|---|---|
| `usbipd attach` fails | Run PowerShell as Administrator |
| `/dev/ttyUSB0` not visible | Check `dmesg \| tail` in WSL after attaching |
| Permission denied on port | Add user to `dialout` group (step 5) |
| Device disconnects after WSL restart | Re-run `usbipd attach --wsl --busid <id>` |
| Wrong busid after replug | Run `usbipd list` again — busid may change |
