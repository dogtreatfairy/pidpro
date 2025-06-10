// src/main.rs

mod database;
mod boards;
mod controller;

use database::sql::SqlDbManager;
use boards::{
    BoardPinMap, 
    HardwareBridge, 
    HalPinState,
    BoardError,
    PlatformHal,
    DummyHal,
    PinMap, // Use the generic PinMap struct
};
use boards::pfpwm::{PIN_MAP as PFPWM_PIN_MAP, BOARD_NAME as PFPWM_BOARD_NAME};
use boards::esp32::{PIN_MAP as ESP32_PIN_MAP, BOARD_NAME as ESP32_BOARD_NAME};
use boards::orangepi::{PIN_MAP as OPI_ZERO_PIN_MAP, BOARD_NAME as OPI_ZERO_BOARD_NAME};

// Conditional HAL and specific PinMap imports
#[cfg(feature = "rpi")]
use boards::rpi_hal::RppalHal;

#[cfg(feature = "esp32")]
use boards::esp32_hal::EspIdfHal;

#[cfg(feature = "orangepi")]
use boards::orangepi_hal::SysfsOrangePiHal;
// OrangePi will use PiFirePwmPinMap as a placeholder for now

use rusqlite::Result;
use std::io::{self, Write};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use controller::run_controller_with_flag;


fn get_default_board_type() -> &'static str {
    #[cfg(feature = "rpi")]
    { return "pfpwm"; }
    #[cfg(feature = "esp32")]
    { return "esp32"; }
    #[cfg(feature = "orangepi")]
    { return "orangepi"; }
    // fallback default
    "pfpwm"
}

fn main() -> Result<()> {
    let db_path = "settings.db";
    let sql_manager = SqlDbManager::new(db_path)?;

    // Use the default board type if not set in the database
    if sql_manager.get_setting("board_type").is_err() {
        let default_board = get_default_board_type();
        let _ = sql_manager.set_setting("board_type", default_board);
        println!("[INFO] No board_type set. Defaulting to: {}", default_board);
    }

    println!("Dynamic Settings Manager. Type 'help' for commands.");

    // Controller thread management
    let running = Arc::new(AtomicBool::new(false));
    let mut controller_handle = None;

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Error reading input.");
            continue;
        }

        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        let command = parts.get(0).unwrap_or(&"");

        match *command {
            "start" => {
                if running.load(Ordering::SeqCst) {
                    println!("Controller is already running.");
                } else {
                    running.store(true, Ordering::SeqCst);
                    let running_flag = running.clone();
                    // Clone board selection logic from fanon/fanoff
                    let current_board_type = match sql_manager.get_setting("board_type") {
                        Ok(val) => val.to_string(),
                        Err(_) => "pfpwm".to_string(),
                    };
                    controller_handle = Some(thread::spawn(move || {
                        match current_board_type.as_str() {
                            "pfpwm" => {
                                let board_map = PinMap { pins: PFPWM_PIN_MAP, board_name: PFPWM_BOARD_NAME };
                                #[cfg(feature = "rpi")]
                                {
                                    if let Ok(hal) = RppalHal::new() {
                                        let mut bridge = HardwareBridge::new(board_map, hal);
                                        run_controller_with_flag(&mut bridge, running_flag);
                                    }
                                }
                                #[cfg(not(feature = "rpi"))]
                                {
                                    let dummy_hal = DummyHal;
                                    let mut bridge = HardwareBridge::new(board_map, dummy_hal);
                                    run_controller_with_flag(&mut bridge, running_flag);
                                }
                            }
                            "esp32" => {
                                let board_map = PinMap { pins: ESP32_PIN_MAP, board_name: ESP32_BOARD_NAME };
                                #[cfg(feature = "esp32")]
                                {
                                    if let Ok(hal) = EspIdfHal::new() {
                                        let mut bridge = HardwareBridge::new(board_map, hal);
                                        run_controller_with_flag(&mut bridge, running_flag);
                                    }
                                }
                                #[cfg(not(feature = "esp32"))]
                                {
                                    let dummy_hal = DummyHal;
                                    let mut bridge = HardwareBridge::new(board_map, dummy_hal);
                                    run_controller_with_flag(&mut bridge, running_flag);
                                }
                            }
                            "orangepi" => {
                                let board_map = PinMap { pins: OPI_ZERO_PIN_MAP, board_name: OPI_ZERO_BOARD_NAME };
                                #[cfg(feature = "orangepi")]
                                {
                                    if let Ok(hal) = SysfsOrangePiHal::new() {
                                        let mut bridge = HardwareBridge::new(board_map, hal);
                                        run_controller_with_flag(&mut bridge, running_flag);
                                    }
                                }
                                #[cfg(not(feature = "orangepi"))]
                                {
                                    let dummy_hal = DummyHal;
                                    let mut bridge = HardwareBridge::new(board_map, dummy_hal);
                                    run_controller_with_flag(&mut bridge, running_flag);
                                }
                            }
                            _ => println!("[ERROR] Unknown board_type '{}' configured. Cannot start controller.", current_board_type),
                        }
                    }));
                    println!("Controller started.");
                }
            }
            "stop" => {
                if running.load(Ordering::SeqCst) {
                    running.store(false, Ordering::SeqCst);
                    if let Some(handle) = controller_handle.take() {
                        let _ = handle.join();
                    }
                    println!("Controller stopped.");
                } else {
                    println!("Controller is not running.");
                }
            }
            "get" => {
                if let Some(key) = parts.get(1) {
                    match sql_manager.get_setting(key) {
                        Ok(value) => println!("  {} = {}", key, value),
                        Err(_) => println!("Error: Unknown setting key '{}'", key),
                    }
                } else {
                    println!("Usage: get <setting_key>");
                }
            }
            "set" => {
                if let (Some(key), Some(val_str)) = (parts.get(1), parts.get(2)) {
                    match sql_manager.set_setting(key, val_str) {
                        Ok(_) => println!("Successfully set '{}' to {}", key, val_str),
                        Err(e) => println!("Error: {}", e),
                    }
                } else {
                    println!("Usage: set <setting_key> <value>");
                }
            }
            "list" => {
                match sql_manager.get_all_settings_grouped() {
                    Ok(grouped) => {
                        for (table_name, settings) in grouped {
                            println!("--- {} ---", table_name.replace('_', " ").to_uppercase());
                            for (key, value) in settings {
                                println!("  {}: {}", key, value);
                            }
                        }
                    }
                    Err(e) => println!("Error fetching settings: {}", e),
                }
            }
            "help" => {
                println!("Available commands:");
                println!("  start              - Start the controller loop.");
                println!("  stop               - Stop the controller loop.");
                println!("  get <key>          - Get a setting's value.");
                println!("  set <key> <value>  - Set a setting's value.");
                println!("  list               - List all settings and their current values.");
                println!("  fanon              - Turn the fan relay ON.");
                println!("  fanoff             - Turn the fan relay OFF.");
                println!("  switch             - Switch the active board type.");
                println!("  exit               - Exit the application.");
            }
            "fanon" | "fanoff" => {
                // Fix: get_setting returns SettingValue, not String
                let current_board_type = match sql_manager.get_setting("board_type") {
                    Ok(val) => val.to_string(),
                    Err(_) => "pfpwm".to_string(),
                };
                let target_state = if *command == "fanon" { HalPinState::High } else { HalPinState::Low };
                println!("[INFO] Command: {}, Board Type: {}, Target State: {:?}", command, current_board_type, target_state);

                match current_board_type.as_str() {
                    "pfpwm" => {
                        let board_map = PinMap { pins: PFPWM_PIN_MAP, board_name: PFPWM_BOARD_NAME };
                        #[cfg(feature = "rpi")]
                        {
                            println!("[INFO] Attempting to use RppalHal for pfpwm (Raspberry Pi).");
                            match RppalHal::new() {
                                Ok(hal) => {
                                    let mut bridge = HardwareBridge::new(board_map, hal);
                                    handle_fan_command(&mut bridge, target_state, "RELAY_FAN");
                                }
                                Err(e) => {
                                    println!("[ERROR] Failed to initialize RppalHal: {:?}. Falling back to DummyHal.", e);
                                    let dummy_hal = DummyHal;
                                    let mut bridge = HardwareBridge::new(board_map, dummy_hal);
                                    handle_fan_command(&mut bridge, target_state, "RELAY_FAN");
                                }
                            }
                        }
                        #[cfg(not(feature = "rpi"))]
                        {
                            println!("[INFO] RPi feature not enabled. Using DummyHal for pfpwm board.");
                            let dummy_hal = DummyHal;
                            let mut bridge = HardwareBridge::new(board_map, dummy_hal);
                            handle_fan_command(&mut bridge, target_state, "RELAY_FAN");
                        }
                    }
                    "esp32" => {
                        let board_map = PinMap { pins: ESP32_PIN_MAP, board_name: ESP32_BOARD_NAME };
                        #[cfg(feature = "esp32")]
                        {
                            println!("[INFO] Attempting to use EspIdfHal for esp32.");
                            match EspIdfHal::new() {
                                Ok(hal) => {
                                    let mut bridge = HardwareBridge::new(board_map, hal);
                                    handle_fan_command(&mut bridge, target_state, "RELAY_FAN");
                                }
                                Err(e) => {
                                    println!("[ERROR] Failed to initialize EspIdfHal: {:?}. Falling back to DummyHal.", e);
                                    let dummy_hal = DummyHal;
                                    let mut bridge = HardwareBridge::new(board_map, dummy_hal);
                                    handle_fan_command(&mut bridge, target_state, "RELAY_FAN");
                                }
                            }
                        }
                        #[cfg(not(feature = "esp32"))]
                        {
                            println!("[INFO] ESP32 feature not enabled. Using DummyHal for esp32 board.");
                            let dummy_hal = DummyHal;
                            let mut bridge = HardwareBridge::new(board_map, dummy_hal);
                            handle_fan_command(&mut bridge, target_state, "RELAY_FAN");
                        }
                    }
                    "orangepi" => {
                        let board_map = PinMap { pins: OPI_ZERO_PIN_MAP, board_name: OPI_ZERO_BOARD_NAME };
                        #[cfg(feature = "orangepi")]
                        {
                            println!("[INFO] Attempting to use SysfsOrangePiHal for orangepi.");
                            match SysfsOrangePiHal::new() {
                                Ok(hal) => {
                                    let mut bridge = HardwareBridge::new(board_map, hal);
                                    handle_fan_command(&mut bridge, target_state, "RELAY_FAN");
                                }
                                Err(e) => {
                                    println!("[ERROR] Failed to initialize SysfsOrangePiHal: {:?}. Falling back to DummyHal.", e);
                                    let dummy_hal = DummyHal;
                                    let mut bridge = HardwareBridge::new(board_map, dummy_hal);
                                    handle_fan_command(&mut bridge, target_state, "RELAY_FAN");
                                }
                            }
                        }
                        #[cfg(not(feature = "orangepi"))]
                        {
                            println!("[INFO] OrangePi feature not enabled. Using DummyHal for orangepi board.");
                            let dummy_hal = DummyHal;
                            let mut bridge = HardwareBridge::new(board_map, dummy_hal);
                            handle_fan_command(&mut bridge, target_state, "RELAY_FAN");
                        }
                    }
                    _ => println!("[ERROR] Unknown board_type '{}' configured. Cannot control fan.", current_board_type),
                }
            }
            "switch" => {
                switch(&sql_manager);
            }
            "exit" => {
                println!("Exiting.");
                break;
            }
            "" => {}
            _ => {
                println!("Unknown command. Type 'help' for a list of commands.");
            }
        }
    }

    Ok(())
}

// Helper function to handle fan commands
fn handle_fan_command<M: BoardPinMap, H: PlatformHal>(
    bridge: &mut HardwareBridge<M, H>,
    state: HalPinState,
    function_name: &str,
) {
    println!(
        "[INFO] Attempting to set {} to {:?} on board: {}",
        function_name,
        state,
        bridge.pin_map.get_board_name()
    );
    match bridge.pin_map.pin_for_function(function_name) {
        Some(pin_info) => {
            println!(
                "[INFO] Logical pin '{}' maps to GPIO: {}. Physical pin: {:?}. Pin type: {:?}",
                function_name,
                pin_info.gpio_pin,
                pin_info.physical_pin,
                pin_info.pin_type
            );
            match bridge.set_logical_pin_state(function_name, state) {
                Ok(_) => println!(
                    "[SUCCESS] Successfully set {} to {:?}.",
                    function_name,
                    state
                ),
                Err(e) => match e {
                    BoardError::PinNotFound(name) => println!("[ERROR] Pin function '{}' (detail: {}) not found on this board.", function_name, name),
                    BoardError::InvalidPinType(desc) => println!("[ERROR] Invalid pin type for '{}'. {}", function_name, desc),
                    BoardError::HalError(hal_err) => println!("[ERROR] Generic HAL error for '{}': {}", function_name, hal_err),
                    _ => println!("[ERROR] Unhandled BoardError for '{}': {:?}", function_name, e),
                },
            }
        }
        None => {
            println!(
                "[ERROR] Pin function '{}' is not defined for board type '{}'.",
                function_name,
                bridge.pin_map.get_board_name()
            );
        }
    }
}

// New switch function
fn switch(sql_manager: &SqlDbManager) {
    // List of (board_key, board_name)
    let boards = vec![
        ("esp32", boards::esp32::BOARD_NAME),
        ("orangepi", boards::orangepi::BOARD_NAME),
        ("pfpwm", boards::pfpwm::BOARD_NAME),
    ];
    println!("Available boards:");
    for (i, (key, name)) in boards.iter().enumerate() {
        println!("  {}. {} - {}", i + 1, key, name);
    }
    print!("Select a board by number (1-{}): ", boards.len());
    io::stdout().flush().unwrap();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        println!("Error reading input.");
        return;
    }
    let selection = input.trim().parse::<usize>();
    match selection {
        Ok(num) if num >= 1 && num <= boards.len() => {
            let (selected_key, selected_name) = boards[num - 1];
            match sql_manager.set_setting("board_type", selected_key) {
                Ok(_) => println!("Board switched to: {} - {}", selected_key, selected_name),
                Err(e) => println!("Error updating board_type: {}", e),
            }
        }
        _ => println!("Invalid selection."),
    }
}
