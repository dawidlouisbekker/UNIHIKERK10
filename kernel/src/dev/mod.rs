pub mod accelerometer;
pub mod screen;


pub fn init() {
  // Route `log` macros to ESP-IDF's logging subsystem (visible over USB-CDC).
  esp_idf_svc::log::EspLogger::initialize_default();
  
}