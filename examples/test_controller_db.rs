// Quick test to demonstrate SqlDbManager now works with controller schemas
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Import the modules we need
    use pidpro::database::sql::SqlDbManager;

    // Remove existing test database if present
    let _ = fs::remove_file("test_controller_settings.db");
    
    println!("=== Testing SqlDbManager with Controller Schemas ===\n");
    
    // Create new SqlDbManager (should initialize both static and controller tables)
    let db = SqlDbManager::new("test_controller_settings.db")?;
    
    println!("1. Testing access to STATIC schema settings:");
    
    // Test static schema access
    let device_name = db.get_setting("device_name")?;
    println!("   ✓ device_name: {}", device_name);
    
    let max_temp = db.get_setting("max_grill_temp")?;
    println!("   ✓ max_grill_temp: {}", max_temp);
    
    println!("\n2. Testing access to CONTROLLER schema settings:");
    
    // Test PMode controller settings 
    let p_mode = db.get_setting("p_mode")?;
    println!("   ✓ p_mode (from pmode_settings table): {}", p_mode);
    
    // Test PID controller settings
    let kp = db.get_setting("kp")?;
    println!("   ✓ kp (from pid_settings table): {}", kp);
    
    let setpoint = db.get_setting("setpoint")?;
    println!("   ✓ setpoint (from pid_settings table): {}", setpoint);
    
    println!("\n3. Testing setting updates for controller settings:");
    
    // Update controller settings
    db.set_setting("p_mode", "8")?;
    let updated_p_mode = db.get_setting("p_mode")?;
    println!("   ✓ Updated p_mode to: {}", updated_p_mode);
    
    db.set_setting("kp", "3.5")?;
    let updated_kp = db.get_setting("kp")?;
    println!("   ✓ Updated kp to: {}", updated_kp);
    
    println!("\n4. Testing grouped settings (should show ALL tables including controller tables):");
    
    let grouped = db.get_all_settings_grouped()?;
    for (table_name, settings) in grouped {
        println!("\n   Table: {}", table_name);
        for (key, value) in settings {
            println!("     {}: {}", key, value);
        }
    }
    
    println!("\n=== SUCCESS: All controller and static settings are accessible! ===");
    
    // Clean up
    let _ = fs::remove_file("test_controller_settings.db");
    
    Ok(())
}
