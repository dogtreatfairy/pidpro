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

## Database Schema Architecture

PIDPro uses a SQLite database with a dynamic schema system that combines static configuration tables with controller-specific tables. This architecture allows for flexible controller implementations while maintaining a consistent database interface.

### Schema Components

The database schema consists of two main parts:

1. **Static Schema** (`sql_schema.rs`): Core configuration tables that are always present
2. **Controller Schemas**: Dynamic tables provided by individual controller implementations

### Static Schema Tables

Defined in `src/database/sql_schema.rs`:

- **`device_config`**: Basic device information (name, ID, board type, controller type)
- **`safety_config`**: Safety parameters (max temperatures, timeouts, thresholds)
- **`mqtt_config`**: MQTT broker settings and connectivity options
- **`operational_settings`**: General operational parameters

### Controller-Specific Tables

Controllers implement the `ControllerSchema` trait to provide their own database tables:

- **PMode Controller** (`pmode_settings`): Simple on/off timing parameters
  - `p_mode`: Timing multiplier for auger off-time
  - `cycle_time`: Duration in seconds for one complete cycle (default: 15)
  
- **PID Controller** (`pid_settings`): PID algorithm parameters
  - `Pb`: Proportional band (gain parameter)
  - `Ti`: Integral time constant
  - `Td`: Derivative time constant
  - `cycle_time`: Duration in seconds for one complete cycle (default: 15)
  
  Note: Setpoint is stored in Redis database for frequent user adjustments

### SqlDbManager Integration

The `SqlDbManager` automatically combines static and controller schemas:

```rust
// Creating the database manager loads all schemas
let db = SqlDbManager::new("settings.db")?;

// Access static settings
let device_name = db.get_setting("device_name")?;

// Access controller-specific settings
let p_mode = db.get_setting("p_mode")?;     // From pmode_settings table
let pb = db.get_setting("Pb")?;             // From pid_settings table
let cycle_time = db.get_setting("cycle_time")?; // Controller-specific cycle time

// Set controller values
db.set_setting("p_mode", "6")?;
db.set_setting("Pb", "70.0")?;
db.set_setting("cycle_time", "20")?;        // Set custom cycle time

// List all settings across all tables
let grouped = db.get_all_settings_grouped()?;
```

### Database Initialization and Migration

- **New Database**: All tables (static + controller) are created with default values
- **Existing Database**: Schema migrations handle adding new columns and tables
- **Controller Changes**: Unused controller tables are automatically cleaned up
- **Version Management**: Schema versions track database evolution

### Adding New Controllers

To add a new controller with database settings:

1. Create controller struct implementing `ControllerSchema`
2. Define controller tables with columns and defaults
3. Register controller in `SqlDbManager::new()`
4. Settings become automatically accessible through the database manager

Example:
```rust
impl ControllerSchema for MyController {
    fn controller_tables() -> &'static [Table] {
        const TABLES: &[Table] = &[
            Table {
                name: "my_controller_settings",
                columns: &[
                    Column { name: "parameter1", data_type: "INTEGER", default_value: "100" },
                    Column { name: "parameter2", data_type: "REAL", default_value: "1.5" },
                ],
            },
        ];
        TABLES
    }
}
```

This architecture ensures that all controller settings are accessible through a unified interface while maintaining separation of concerns between different controller implementations.

### Controller-Specific Cycle Time

Each controller defines its own cycle time, allowing for different timing characteristics:

- **Default cycle time**: 15 seconds for all controllers
- **Customizable per controller**: PMode and PID controllers can have different cycle times
- **Database managed**: Cycle time is stored in each controller's settings table
- **Runtime configurable**: Can be changed via CLI or API without recompilation

The cycle time determines the base timing unit for controller operations:
- **PMode Controller**: Uses cycle time for auger on-time, with variable off-time based on p_mode
- **PID Controller**: Uses cycle time as the base period for PID calculations and output adjustments

**Note**: PID setpoint is stored in Redis database for frequent user changes, not in the SQLite configuration database.

Example cycle time usage:
```rust
// Each controller has its own cycle_time setting
// PMode: auger_on_time = cycle_time, auger_off_time = cycle_time + (p_mode * 10)
// PID: output_duty_cycle applied over cycle_time period
```

## Command Line Interface

PIDPro provides an interactive CLI for managing settings and controlling the system:

### Available Commands

- **`list`**: Display all settings grouped by table (includes both static and controller settings)
- **`get <key>`**: Retrieve the value of a specific setting
- **`set <key> <value>`**: Update a setting value
- **`start`**: Begin the controller loop with current settings
- **`stop`**: Stop the controller loop
- **`controller`**: Switch between available controller types (pmode, pid)
- **`switch`**: Change the active board type
- **`fanon`/`fanoff`**: Manual fan control
- **`help`**: Show available commands
- **`exit`**: Quit the application

### Example Usage

```sh
# List all current settings
> list

# Get specific controller setting
> get p_mode
> get Pb
> get cycle_time

# Update controller parameters
> set p_mode 6
> set Pb 70.0
> set Ti 200.0
> set cycle_time 20

# Switch controller type
> controller
> start

# Manual fan control
> fanon
> fanoff
```

The CLI automatically works with all settings across both static configuration tables and controller-specific tables, providing a unified interface for system management.

See the source code and comments for more details on board abstraction and hardware compatibility.
