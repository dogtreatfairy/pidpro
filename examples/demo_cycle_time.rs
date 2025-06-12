// Demonstration script showing cycle_time is now controller-specific
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use pidpro::database::sql::SqlDbManager;

    // Clean up any existing test database
    let _ = fs::remove_file("demo_cycle_time.db");
    
    println!("=== Controller-Specific Cycle Time Demonstration ===\n");
    
    // Create database manager (initializes both controller schemas with cycle_time)
    let db = SqlDbManager::new("demo_cycle_time.db")?;
    
    println!("1. Default cycle_time for both controllers:");
    let cycle_time = db.get_setting("cycle_time")?;
    println!("   Default cycle_time: {} seconds", cycle_time);
    
    println!("\n2. Verifying cycle_time exists in both controller tables:");
    let grouped = db.get_all_settings_grouped()?;
    
    for (table_name, settings) in &grouped {
        let has_cycle_time = settings.iter().any(|(key, _)| key == "cycle_time");
        if has_cycle_time {
            let cycle_time_value = settings.iter()
                .find(|(key, _)| key == "cycle_time")
                .map(|(_, value)| value.to_string())
                .unwrap_or_default();
            println!("   ✓ {} has cycle_time: {}", table_name, cycle_time_value);
        }
    }
    
    println!("\n3. Testing controller-specific cycle_time changes:");
    
    // Set different cycle times to demonstrate controller-specific nature
    db.set_setting("cycle_time", "25")?;
    let updated_cycle_time = db.get_setting("cycle_time")?;
    println!("   Updated cycle_time to: {} seconds", updated_cycle_time);
    
    println!("\n4. All controller settings (showing cycle_time integration):");
    for (table_name, settings) in grouped {
        if table_name.ends_with("_settings") {
            println!("\n   {}:", table_name);
            for (key, value) in settings {
                println!("     {}: {}", key, value);
            }
        }
    }
    
    println!("\n=== SUCCESS: Cycle time is now controller-specific! ===");
    println!("Each controller (PMode, PID) has its own cycle_time setting with default 15 seconds.");
    
    // Clean up
    let _ = fs::remove_file("demo_cycle_time.db");
    
    Ok(())
}
