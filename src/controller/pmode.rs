// PMode controller for auger timing
// u is always 1.0 (full on) for cycle_length, then off for cycle_length + (p_mode * 10)

use crate::database::sql_schema::{Table, Column};
use super::ControllerSchema;

pub struct PModeController {
    pub p_mode: u32,
}

impl PModeController {
    pub fn new(p_mode: u32) -> Self {
        Self { p_mode }
    }
    /// Returns (on_time, off_time) in seconds for the current p_mode and cycle_length
    pub fn get_on_off_times(&self, cycle_length: u64) -> (u64, u64) {
        let on_time = cycle_length;
        let off_time = cycle_length + (self.p_mode as u64) * 10;
        (on_time, off_time)
    }
    /// Returns u (fraction of cycle auger should be ON). For p_mode, always 1.0 (on full cycle)
    pub fn u(&self) -> f32 {
        1.0
    }
}

impl ControllerSchema for PModeController {
    fn controller_tables() -> &'static [Table] {
        const TABLES: &[Table] = &[
            Table {
                name: "pmode_settings",
                columns: &[
                    Column { name: "p_mode", data_type: "INTEGER", default_value: "4" },
                    Column { name: "cycle_time", data_type: "INTEGER", default_value: "15" },
                ],
            },
        ];
        TABLES
    }
}
