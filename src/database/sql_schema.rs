// src/schema.rs

// This file defines the entire structure of the SQLite database.
// When you want to add a new setting, this is the main place you'll make changes.

pub const SCHEMA_VERSION: u32 = 5;

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
    Table {
        name: "safety_config",
        columns: &[
            Column { name: "max_grill_temp", data_type: "INTEGER", default_value: "550" },
            Column { name: "igniter_temp_window", data_type: "INTEGER", default_value: "15" },
            Column { name: "min_startup_temp_delta", data_type: "INTEGER", default_value: "10" },
            Column { name: "max_auger_on_time_no_ignition", data_type: "INTEGER", default_value: "300" },
        ],
    },
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
	Table {
		name: "mqtt_config",
		columns: &[
			Column { name: "mqtt_broker", data_type: "TEXT", default_value: "'mqtt://localhost'" },
			Column { name: "mqtt_port", data_type: "INTEGER", default_value: "1883" },
			Column { name: "mqtt_username", data_type: "TEXT", default_value: "''" },
			Column { name: "mqtt_password", data_type: "TEXT", default_value: "''" },
			Column { name: "mqtt_topic_prefix", data_type: "TEXT", default_value: "'pidpro'" },
		],
	}
    // Future tables like 'history' would be added here.
];

// (table, old_column, new_column)
pub const RENAMES: &[(&str, &str, &str)] = &[
    // Example: ("operational_settings", "p_mode", "p_mode_value"),
];
