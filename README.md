# PIDPro

## Build Instructions

This project supports multiple hardware platforms (Raspberry Pi, ESP32, Orange Pi) with automatic board selection based on build features.

### Building for a Specific Board

Use Cargo features to select the target board at build time:

- **Raspberry Pi (pfpwm):**
  ```sh
  cargo build --release --features rpi
  ```
- **ESP32:**
  ```sh
  cargo build --release --features esp32
  ```
- **Orange Pi:**
  ```sh
  cargo build --release --features orangepi
  ```

### How Board Selection Works
- On first run, if no board is set in the database, the application will automatically select a default board based on the enabled Cargo feature:
  - `--features rpi` → `pfpwm`
  - `--features esp32` → `esp32`
  - `--features orangepi` → `orangepi`
- You can override the board at runtime using the `switch` command in the CLI.
- The selected board is stored in the `settings.db` database.

### Cross-Compilation
- For Raspberry Pi, you may need to install the ARMv7 toolchain:
  ```sh
  rustup target add armv7-unknown-linux-gnueabihf
  cargo build --release --target=armv7-unknown-linux-gnueabihf --features rpi
  ```
- For ESP32, use the appropriate Rust cross-compilation setup for ESP-IDF.

See the source code and comments for more details on board abstraction and hardware compatibility.
