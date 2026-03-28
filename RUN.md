# Build
cargo build -p kernel --release

# Build + flash + open serial monitor (K10 connected via USB-C)
cargo run -p kernel --release
