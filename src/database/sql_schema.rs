// src/schema.rs

// This file defines the entire structure of the SQLite database.
// When you want to add a new setting, this is the main place you'll make changes.

pub const SCHEMA_VERSION: u32 = 6;

pub struct Column {
    pub name: &'static str,
    pub data_type: &'static str,
    pub default_value: &'static str,
}

pub struct Table {
    pub name: &'static str,
    pub columns: &'static [Column],
}

// This is the single source of truth for the database schema.
// To add a new setting, add a new `Column` to the appropriate table.
// To add a new table, add a new `Table` to this array.

pub const SCHEMA: &[Table] = &[
	// CONFIG -- Items that change infrequently, like safety limits and operational parameters.
	Table {
		name: "device_config",
		columns: &[
			Column { name: "device_name", data_type: "TEXT", default_value: "'PIDPro'" }, // Name of the device.
			Column { name: "device_id", data_type: "TEXT", default_value: "'default-id'" }, // Unique identifier for the device.
			Column { name: "version", data_type: "INTEGER", default_value: "1" }, // Version of the configuration.
			Column { name: "board_type", data_type: "TEXT", default_value: "'pfpwm'" }, // Type of board used (e.g., ESP32 (esp32), Raspberry Pi Zero 2 W (p02w)).
		],
	},
	Table {
        name: "safety_config",
        columns: &[
            Column { name: "max_grill_temp", data_type: "INTEGER", default_value: "550" }, // Max grill temperature in Fahrenheit.
            Column { name: "flameout_window", data_type: "INTEGER", default_value: "15" }, // If temp is outside this window, consider grill flamed out.
			Column { name: "flameout_auger_timeout", data_type: "INTEGER", default_value: "300" }, // How long to wait before error and shutdown if not temp rise
            Column { name: "min_startup_temp_delta", data_type: "INTEGER", default_value: "20" }, // Minimum temp rise to consider startup sequence finished.
        ],
    },
	Table {
		name: "mqtt_config",
		columns: &[
			Column { name: "enabled", data_type: "BOOLEAN", default_value: "0" }, //0 for false, 1 for true
			Column { name: "mqtt_broker", data_type: "TEXT", default_value: "'mqtt://localhost'" },
			Column { name: "mqtt_port", data_type: "INTEGER", default_value: "1883" },
			Column { name: "mqtt_username", data_type: "TEXT", default_value: "''" },
			Column { name: "mqtt_password", data_type: "TEXT", default_value: "''" },
			Column { name: "mqtt_topic_prefix", data_type: "TEXT", default_value: "'pidpro'" },
		],
	},
	
	// SETTINGS -- Items that change more frequently.

    Table {
        name: "operational_settings",
        columns: &[
            Column { name: "p_mode", data_type: "INTEGER", default_value: "4" },
			Column { name: "pid_pb", data_type: "INTEGER", default_value: "65" },
			Column { name: "pid_ti", data_type: "INTEGER", default_value: "180" },
			Column { name: "pid_td", data_type: "INTEGER", default_value: "45" },
			Column { name: "primary_setpoint", data_type: "INTEGER", default_value: "225" },
        ],
    },

    // Future tables like 'history' would be added here.
];


// Update Mapper
// (table, old_column, new_column)
pub const RENAMES: &[(&str, &str, &str)] = &[
    // Example: ("operational_settings", "p_mode", "p_mode_value"),
];
