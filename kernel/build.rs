fn main() {
    // Propagates ESP-IDF environment variables (include paths, linker flags, etc.)
    // into the Cargo build graph so that esp-idf-sys can find the compiled C SDK.
    embuild::espidf::sysenv::output();
}
