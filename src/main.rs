// src/main.rs

// This line tells Rust to look for a `database` directory with a `mod.rs` file.
mod database;

// Import the manager from its new, nested location.
use database::sql::SqlManager;

use rusqlite::Result;
use std::io::{self, Write};


fn main() -> Result<()> {
    let db_path = "settings.db";
    // Create an instance of the renamed manager.
    let sql_manager = SqlManager::new(db_path)?;

    println!("Dynamic Settings Manager. Type 'help' for commands.");

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
                println!("  get <key>          - Get a setting's value.");
                println!("  set <key> <value>    - Set a setting's value.");
                println!("  list               - List all settings and their current values.");
                println!("  exit               - Exit the application.");
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
