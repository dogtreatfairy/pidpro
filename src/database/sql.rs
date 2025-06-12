// src/database/sql.rs

// Use `super::` to refer to a sibling module
use super::sql_schema::{SCHEMA, Column, SCHEMA_VERSION};

use rusqlite::{Connection, Result, params, types::Value};
use std::collections::HashMap;
use std::path::Path;
use std::fmt;
use crate::controller::{ControllerSchema, pmode::PModeController, pid::PidController};


#[derive(Debug)]
pub enum SettingValue {
    Integer(i64),
    Text(String),
	Boolean(bool),
}

impl fmt::Display for SettingValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SettingValue::Integer(val) => write!(f, "{}", val),
            SettingValue::Text(val) => write!(f, "{}", val),
			SettingValue::Boolean(val) => write!(f, "{}", if *val { "true" } else { "false" }),	
        }
    }
}

impl From<Value> for SettingValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Integer(i) => {
                // Check if this might be a boolean stored as integer
                if i == 0 || i == 1 {
                    // We could try to detect booleans here, but for now treat as integer
                    SettingValue::Integer(i)
                } else {
                    SettingValue::Integer(i)
                }
            },
            Value::Real(r) => SettingValue::Text(r.to_string()), // Convert REAL to text for now
            Value::Text(s) => SettingValue::Text(s),
            Value::Blob(_) => SettingValue::Text("Binary Data".to_string()),
            Value::Null => SettingValue::Text("NULL".to_string()),
        }
    }
}

pub struct SqlDbManager {
    conn: Connection,
    full_schema: Vec<&'static super::sql_schema::Table>,
}

impl SqlDbManager {
    /// Create a new SqlDbManager instance, initializing the database and schema if needed.
    ///
    /// This function will:
    /// - Check if the database file exists at the given path
    /// - Open a connection to the database
    /// - Initialize the schema: create tables and set initial values
    /// - Migrate the schema if the current version is older than the defined SCHEMA_VERSION
    /// - Clean up any controller tables that are no longer present in the schema
    /// 
    /// # Errors
    /// Returns an error if the database cannot be opened, or if schema initialization or migration fails.
    pub fn new(db_path: &str) -> Result<Self> {
        let db_exists = Path::new(db_path).exists();
        let mut conn = Connection::open(db_path)?;        // Gather all controller tables
        let mut controller_tables: Vec<&'static super::sql_schema::Table> = Vec::new();
        controller_tables.extend(PModeController::controller_tables().iter());
        controller_tables.extend(PidController::controller_tables().iter());
        // Compose full schema: static + controller tables (as slices of references)
        // Convert SCHEMA (&[Table]) to &[&Table] for type compatibility
        let mut full_schema: Vec<&'static super::sql_schema::Table> = SCHEMA.iter().collect();
        full_schema.extend(controller_tables);
        let full_schema_slice: &[&super::sql_schema::Table] = &full_schema;
        if !db_exists {
            Self::initialize_database_with_schema(&mut conn, full_schema_slice)?;
            Self::set_schema_version(&mut conn, SCHEMA_VERSION)?;
        } else {
            let current_version = Self::get_schema_version(&mut conn)?;
            if SCHEMA_VERSION > current_version {
                Self::migrate_schema_with_schema(&mut conn, current_version, SCHEMA_VERSION, full_schema_slice)?;
                Self::set_schema_version(&mut conn, SCHEMA_VERSION)?;
            }
            // Remove tables for controllers that are no longer present
            Self::cleanup_controller_tables(&mut conn, full_schema_slice)?;        }
        Ok(SqlDbManager { 
            conn,
            full_schema,
        })
    }    pub fn _get_all_settings(&self) -> Result<HashMap<String, SettingValue>> {
        let mut settings_map = HashMap::new();
        for table in &self.full_schema {
            for column in table.columns {
                let value = self.get_setting(column.name)?;
                settings_map.insert(column.name.to_string(), value);
            }
        }
        Ok(settings_map)
    }    pub fn get_setting(&self, key: &str) -> Result<SettingValue> {
        let (table_name, column) = self.find_table_and_column(key)?;
        let raw_value = self.conn.query_row(
            &format!("SELECT {} FROM {} WHERE id = 1", key, table_name),
            [],
            |row| row.get::<_, Value>(0),
        )?;
        
        // Convert based on expected column type
        match column.data_type {
            "BOOLEAN" => {
                match raw_value {
                    Value::Integer(i) => Ok(SettingValue::Boolean(i != 0)),
                    _ => Ok(SettingValue::Boolean(false)),
                }
            },
            _ => Ok(SettingValue::from(raw_value)),
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let (table_name, column) = self.find_table_and_column(key)?;
        match column.data_type {
            "INTEGER" => {
                let int_val = value.parse::<i32>().map_err(|_| rusqlite::Error::InvalidParameterName("Expected integer value".to_string()))?;
                self.conn.execute(
                    &format!("UPDATE {} SET {} = ?1 WHERE id = 1", table_name, key),
                    params![int_val],
                )?;
            }
            "REAL" => {
                let float_val = value.parse::<f64>().map_err(|_| rusqlite::Error::InvalidParameterName("Expected real/float value".to_string()))?;
                self.conn.execute(
                    &format!("UPDATE {} SET {} = ?1 WHERE id = 1", table_name, key),
                    params![float_val],
                )?;
            }
            "TEXT" => {
                self.conn.execute(
                    &format!("UPDATE {} SET {} = ?1 WHERE id = 1", table_name, key),
                    params![value],
                )?;
            }
            "BOOLEAN" => {
                let bool_val = match value.to_lowercase().as_str() {
                    "true" | "1" => 1,
                    "false" | "0" => 0,
                    _ => return Err(rusqlite::Error::InvalidParameterName("Expected boolean value (true/false/1/0)".to_string())),
                };
                self.conn.execute(
                    &format!("UPDATE {} SET {} = ?1 WHERE id = 1", table_name, key),
                    params![bool_val],
                )?;
            }
            _ => {
                return Err(rusqlite::Error::InvalidParameterName(format!("Unsupported data type: {}", column.data_type)));
            }
        }
        Ok(())
    }
    
    fn get_schema_version(conn: &mut Connection) -> Result<u32> {
        let version: Result<u32> = conn.query_row(
            "SELECT version FROM meta WHERE id = 1",
            [],
            |row| row.get(0),
        );
        match version {
            Ok(v) => Ok(v),
            Err(_) => Ok(0), // If meta table/version doesn't exist, treat as version 0
        }
    }

    fn set_schema_version(conn: &mut Connection, version: u32) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS meta (id INTEGER PRIMARY KEY, version INTEGER)",
            [],
        )?;
        let updated = conn.execute(
            "UPDATE meta SET version = ?1 WHERE id = 1",
            [version],
        )?;
        if updated == 0 {
            conn.execute("INSERT INTO meta (id, version) VALUES (1, ?1)", [version])?;
        }
        Ok(())
    }    fn migrate_schema_with_schema(conn: &mut Connection, _from_version: u32, _to_version: u32, schema: &[&super::sql_schema::Table]) -> Result<()> {
        // (copy logic from migrate_schema, but use provided schema)
        // For each table in the schema
        for table in schema {
            // Get columns in the actual database
            let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table.name))?;
            let db_columns: Vec<(String, String)> = stmt
                .query_map([], |row| Ok((row.get(1)?, row.get::<_, String>(2)?)))?
                .filter_map(Result::ok)
                .collect();
            // Add missing columns and check type changes
            for column in table.columns {
                match db_columns.iter().find(|(c, _)| c == column.name) {
                    Some((_, db_type)) => {
                        // If type does not match, update value if needed
                        let db_type_upper = db_type.to_uppercase();
                        let schema_type_upper = column.data_type.to_uppercase();
                        if db_type_upper != schema_type_upper {
                            // Check current value
                            let value: Option<String> = conn.query_row(
                                &format!("SELECT {} FROM {} WHERE id = 1", column.name, table.name),
                                [],
                                |row| row.get(0),
                            ).ok();
                            let needs_update = match (schema_type_upper.as_str(), value.as_deref()) {
                                ("INTEGER", Some(v)) => v.parse::<i32>().is_err(),
                                ("REAL", Some(v)) => v.parse::<f64>().is_err(),
                                ("BOOLEAN", Some(v)) => !(v == "0" || v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("false")),
                                ("TEXT", _) => false, // Any value is valid for TEXT
                                _ => false,
                            };
                            if needs_update {
                                let sql = format!("UPDATE {} SET {} = {} WHERE id = 1", table.name, column.name, column.default_value);
                                println!("[MIGRATION] Type changed for {}. Setting default: {}", column.name, sql);
                                let _ = conn.execute(&sql, []);
                            }
                        }
                    }
                    None => {
                        // Add missing column
                        let sql = format!("ALTER TABLE {} ADD COLUMN {} {} DEFAULT {}", table.name, column.name, column.data_type, column.default_value);
                        println!("[MIGRATION] {}", sql);
                        let _ = conn.execute(&sql, []);
                    }
                }
            }
        }
        // Handle renames
        for (table, old_col, new_col) in super::sql_schema::RENAMES {
            let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
            let db_columns: Vec<String> = stmt
                .query_map([], |row| row.get(1))?
                .filter_map(Result::ok)
                .collect();
            if db_columns.contains(&old_col.to_string()) && !db_columns.contains(&new_col.to_string()) {
                // Find new column type/default from schema
                if let Some(table_schema) = schema.iter().find(|t| t.name == *table) {
                    if let Some(new_col_schema) = table_schema.columns.iter().find(|c| c.name == *new_col) {
                        let sql = format!("ALTER TABLE {} ADD COLUMN {} {} DEFAULT {}", table, new_col, new_col_schema.data_type, new_col_schema.default_value);
                        println!("[MIGRATION] {}", sql);
                        let _ = conn.execute(&sql, []);
                        let sql = format!("UPDATE {} SET {} = {}", table, new_col, old_col);
                        println!("[MIGRATION] {}", sql);
                        let _ = conn.execute(&sql, []);
                        // Note: SQLite cannot drop columns directly; document manual cleanup if needed
                    }
                }
            }
        }
        Ok(())
    }

    fn cleanup_controller_tables(conn: &mut Connection, schema: &[&super::sql_schema::Table]) -> Result<()> {
        // Get all tables in DB
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table'")?;
        let db_tables: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(Result::ok)
            .collect();
        let schema_tables: Vec<&str> = schema.iter().map(|t| t.name).collect();
        for db_table in db_tables {
            if db_table.ends_with("_settings") && !schema_tables.contains(&db_table.as_str()) {
                let sql = format!("DROP TABLE IF EXISTS {}", db_table);
                let _ = conn.execute(&sql, []);
            }
        }
        Ok(())
    }    fn find_table_and_column(&self, key: &str) -> Result<(&'static str, &'static Column)> {
        for table in &self.full_schema {
            if let Some(column) = table.columns.iter().find(|c| c.name == key) {
                return Ok((table.name, column));
            }
        }
        Err(rusqlite::Error::QueryReturnedNoRows)    }
    
    fn initialize_database_with_schema(conn: &mut Connection, schema: &[&super::sql_schema::Table]) -> Result<()> {
        let transaction = conn.transaction()?;
        for table in schema {
            let columns_sql: String = table.columns
                .iter()
                .map(|c| format!("{} {} DEFAULT {}", c.name, c.data_type, c.default_value))
                .collect::<Vec<String>>()
                .join(", ");
            let create_sql = format!("CREATE TABLE {} (id INTEGER PRIMARY KEY, {});", table.name, columns_sql);
            let insert_sql = format!("INSERT INTO {} (id) VALUES (1);", table.name);
            transaction.execute(&create_sql, [])?;
            transaction.execute(&insert_sql, [])?;
        }
        transaction.execute(
            "CREATE TABLE IF NOT EXISTS meta (id INTEGER PRIMARY KEY, version INTEGER)",
            [],
        )?;
        transaction.execute(
            "INSERT INTO meta (id, version) VALUES (1, ?1)",
            [SCHEMA_VERSION],
        )?;
        transaction.commit()
    }    pub fn get_all_settings_grouped(&self) -> Result<Vec<(&'static str, Vec<(String, SettingValue)>)>> {
        let mut grouped = Vec::new();
        for table in &self.full_schema {
            let mut table_settings = Vec::new();
            for column in table.columns {
                let value = self.get_setting(column.name)?;
                table_settings.push((column.name.to_string(), value));
            }
            grouped.push((table.name, table_settings));
        }
        Ok(grouped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;    #[test]
    fn test_controller_schema_access() {
        // Remove any existing test database
        let _ = fs::remove_file("test_controller_access.db");
        
        // Create new SqlDbManager - should initialize both static and controller tables
        let db = SqlDbManager::new("test_controller_access.db").expect("Failed to create database");
        
        // Test static schema access
        let device_name = db.get_setting("device_name").expect("Failed to get device_name");
        println!("Device name: {}", device_name);
        
        // Test PMode controller settings access
        let p_mode = db.get_setting("p_mode").expect("Failed to get p_mode from controller schema");
        println!("P-mode: {}", p_mode);
          // Test PID controller settings access  
        let pb = db.get_setting("Pb").expect("Failed to get Pb from controller schema");
        println!("Pb: {}", pb);
        
        // Test grouped settings includes controller tables
        let grouped = db.get_all_settings_grouped().expect("Failed to get grouped settings");
        let table_names: Vec<&str> = grouped.iter().map(|(name, _)| *name).collect();
        
        println!("Tables found: {:?}", table_names);
        
        assert!(table_names.contains(&"pmode_settings"), "pmode_settings table should be included");
        assert!(table_names.contains(&"pid_settings"), "pid_settings table should be included");
        assert!(table_names.contains(&"device_config"), "device_config table should be included");
        
        // Clean up
        let _ = fs::remove_file("test_controller_access.db");
        
        println!("✓ All controller schema tests passed!");
    }
    
    #[test]
    fn test_controller_cycle_time() {
        // Remove any existing test database
        let _ = fs::remove_file("test_cycle_time.db");
        
        // Create new SqlDbManager
        let db = SqlDbManager::new("test_cycle_time.db").expect("Failed to create database");
        
        // Test that both controllers have cycle_time defined with default value of 15
        let pmode_cycle_time = db.get_setting("cycle_time").expect("Failed to get cycle_time");
        println!("Default cycle_time: {}", pmode_cycle_time);
        assert_eq!(pmode_cycle_time.to_string(), "15");
        
        // Test setting custom cycle time
        db.set_setting("cycle_time", "20").expect("Failed to set cycle_time");
        let updated_cycle_time = db.get_setting("cycle_time").expect("Failed to get updated cycle_time");
        assert_eq!(updated_cycle_time.to_string(), "20");
        
        // Verify that cycle_time appears in grouped settings
        let grouped = db.get_all_settings_grouped().expect("Failed to get grouped settings");
        
        // Find tables that contain cycle_time
        let tables_with_cycle_time: Vec<&str> = grouped.iter()
            .filter(|(_, settings)| settings.iter().any(|(key, _)| key == "cycle_time"))
            .map(|(table_name, _)| *table_name)
            .collect();
        
        println!("Tables with cycle_time: {:?}", tables_with_cycle_time);
        
        // Should be in both pmode_settings and pid_settings
        assert!(tables_with_cycle_time.contains(&"pmode_settings"), "pmode_settings should have cycle_time");
        assert!(tables_with_cycle_time.contains(&"pid_settings"), "pid_settings should have cycle_time");
        
        // Clean up
        let _ = fs::remove_file("test_cycle_time.db");
        
        println!("✓ Controller cycle_time tests passed!");
    }
    
    #[test]
    fn test_pid_parameters() {
        // Remove any existing test database
        let _ = fs::remove_file("test_pid_params.db");
        
        // Create new SqlDbManager
        let db = SqlDbManager::new("test_pid_params.db").expect("Failed to create database");
        
        // Test PID parameters are accessible with correct names
        let pb = db.get_setting("Pb").expect("Failed to get Pb");
        let ti = db.get_setting("Ti").expect("Failed to get Ti"); 
        let td = db.get_setting("Td").expect("Failed to get Td");
        
        println!("PID Parameters - Pb: {}, Ti: {}, Td: {}", pb, ti, td);
        
        // Verify default values
        assert_eq!(pb.to_string(), "65");
        assert_eq!(ti.to_string(), "180");
        assert_eq!(td.to_string(), "45");
        
        // Test setting new PID values
        db.set_setting("Pb", "70.0").expect("Failed to set Pb");
        db.set_setting("Ti", "200.0").expect("Failed to set Ti");
        db.set_setting("Td", "50.0").expect("Failed to set Td");
        
        // Verify updates worked
        let updated_pb = db.get_setting("Pb").expect("Failed to get updated Pb");
        let updated_ti = db.get_setting("Ti").expect("Failed to get updated Ti");
        let updated_td = db.get_setting("Td").expect("Failed to get updated Td");
        
        println!("Updated PID Parameters - Pb: {}, Ti: {}, Td: {}", updated_pb, updated_ti, updated_td);
        
        // Clean up
        let _ = fs::remove_file("test_pid_params.db");
        
        println!("✓ PID parameter tests passed!");
    }
    
    #[test]
    fn test_setpoint_not_in_database() {
        // Remove any existing test database
        let _ = fs::remove_file("test_no_setpoint.db");
        
        // Create new SqlDbManager
        let db = SqlDbManager::new("test_no_setpoint.db").expect("Failed to create database");
        
        // Verify setpoint is NOT in the database (should return error)
        let setpoint_result = db.get_setting("setpoint");
        assert!(setpoint_result.is_err(), "setpoint should not be in database schema");
        
        // Verify PID parameters ARE available
        let pb = db.get_setting("Pb").expect("Pb should be available");
        let ti = db.get_setting("Ti").expect("Ti should be available");
        let td = db.get_setting("Td").expect("Td should be available");
        let cycle_time = db.get_setting("cycle_time").expect("cycle_time should be available");
        
        println!("Available PID settings - Pb: {}, Ti: {}, Td: {}, cycle_time: {}", pb, ti, td, cycle_time);
        
        // Clean up
        let _ = fs::remove_file("test_no_setpoint.db");
        
        println!("✓ Verified setpoint is correctly moved to Redis (not in database)!");
    }
}
