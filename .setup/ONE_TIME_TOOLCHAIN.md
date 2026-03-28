
# 1. Install espup and the Xtensa Rust fork
```bash
cargo +stable install espup
espup install          # adds the "esp" channel to rustup
```

# 2. Install the flashing tool
```bash
cargo install espflash
```

# 3. Install ldproxy
```bash
rustup
```

# 4. Install espflash
```bash
cargo install espflash --version "3.3.0" #cargo install espflash --version 4.0.1
```

# 5. Source the generated environment file (add to your shell profile)
```bash
. $HOME/export-esp.sh  # Linux/macOS
```
