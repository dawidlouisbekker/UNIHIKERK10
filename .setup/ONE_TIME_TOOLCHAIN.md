
# 1. Install espup and the Xtensa Rust fork
```bash
cargo install espup
espup install          # adds the "esp" channel to rustup
```

# 2. Install the flashing tool
```bash
cargo install espflash
```

# 3. Source the generated environment file (add to your shell profile)
```bash
. $HOME/export-esp.sh  # Linux/macOS
```
