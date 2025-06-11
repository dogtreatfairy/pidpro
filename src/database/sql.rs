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
            Value::Integer(i) => SettingValue::Integer(i),
            Value::Text(s) => SettingValue::Text(s),
            _ => SettingValue::Text("Unsupported Type".to_string()),
        }
    }
}

pub struct SqlDbManager {
    conn: Connection,
}

impl SqlDbManager {
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
            Self::cleanup_controller_tables(&mut conn, full_schema_slice)?;
        }
        Ok(SqlDbManager { conn })
    }

    pub fn _get_all_settings(&self) -> Result<HashMap<String, SettingValue>> {
        let mut settings_map = HashMap::new();
        for table in SCHEMA {
            for column in table.columns {
                let value = self.get_setting(column.name)?;
                settings_map.insert(column.name.to_string(), value);
            }
        }
        Ok(settings_map)
    }

    pub fn get_setting(&self, key: &str) -> Result<SettingValue> {
        let (table_name, _) = self.find_table_and_column(key)?;
        self.conn.query_row(
            &format!("SELECT {} FROM {} WHERE id = 1", key, table_name),
            [],
            |row| row.get::<_, Value>(0).map(SettingValue::from),
        )
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
    }

    	fn migrate_schema(conn: &mut Connection, from_version: u32, to_version: u32) -> Result<()> {
        use super::sql_schema::RENAMES;
        println!("Migrating schema from version {} to {}", from_version, to_version);
        // For each table in the schema
        for table in SCHEMA {
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
        for (table, old_col, new_col) in RENAMES {
            let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
            let db_columns: Vec<String> = stmt
                .query_map([], |row| row.get(1))?
                .filter_map(Result::ok)
                .collect();
            if db_columns.contains(&old_col.to_string()) && !db_columns.contains(&new_col.to_string()) {
                // Find new column type/default from schema
                if let Some(table_schema) = SCHEMA.iter().find(|t| t.name == *table) {
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

    fn migrate_schema_with_schema(conn: &mut Connection, _from_version: u32, _to_version: u32, schema: &[&super::sql_schema::Table]) -> Result<()> {
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
    }

    fn find_table_and_column(&self, key: &str) -> Result<(&'static str, &'static Column)> {
        for table in SCHEMA {
            if let Some(column) = table.columns.iter().find(|c| c.name == key) {
                return Ok((table.name, column));
            }
        }
        Err(rusqlite::Error::QueryReturnedNoRows)
    }
    
    // *** FIX APPLIED HERE: Function now takes a mutable reference ***
    fn initialize_database(conn: &mut Connection) -> Result<()> {
        let transaction = conn.transaction()?;
        for table in SCHEMA {
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
    }

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
    }

    pub fn get_all_settings_grouped(&self) -> Result<Vec<(&'static str, Vec<(String, SettingValue)>)>> {
        let mut grouped = Vec::new();
        for table in SCHEMA {
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
