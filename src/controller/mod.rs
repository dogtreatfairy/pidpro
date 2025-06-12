use std::{thread, time::Duration};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use crate::boards::{BoardPinMap, HardwareBridge, HalPinState};
use crossterm::{execute, terminal::{Clear, ClearType}, cursor::MoveTo};
use std::io::{stdout, Write};

pub mod pmode;
pub mod pid;

pub enum ControllerType {
    PMode(pmode::PModeController),
    PID(pid::PidController),
    // Add more controllers here
}

impl ControllerType {
    /// Returns u (fraction of cycle auger should be ON)
    pub fn u(&self) -> f32 {
        match self {
            ControllerType::PMode(ctrl) => ctrl.u(),
            ControllerType::PID(ctrl) => ctrl.u(),
        }
    }
}

use crate::database::sql_schema::Table;
// Controller settings schema trait
pub trait ControllerSchema {
    fn controller_tables() -> &'static [Table];
}

fn print_relay_status<M: BoardPinMap, H: crate::boards::PlatformHal>(bridge: &mut HardwareBridge<M, H>, relay_states: &[(&str, bool)]) {
    let mut stdout = stdout();
    // Clear the terminal and move cursor to top-left
    let _ = execute!(stdout, Clear(ClearType::All), MoveTo(0, 0));
    writeln!(stdout, "==== Relay Status ({}) ====", bridge.get_board_name()).unwrap();
    for &(relay, is_on) in relay_states {
        if let Some(pin) = bridge.pin_map.pin_for_function(relay) {
            let state = if is_on { "On (High)" } else { "Off (Low)" };
            writeln!(stdout, "{:<12} - GPIO{:>2} - {}", relay, pin.gpio_pin, state).unwrap();
        } else {
            writeln!(stdout, "{:<12} - (not mapped)", relay).unwrap();
        }
    }
    writeln!(stdout, "======================").unwrap();
    stdout.flush().unwrap();
}

pub fn run_controller_with_flag<M: BoardPinMap, H: crate::boards::PlatformHal>(
    bridge: &mut HardwareBridge<M, H>,
    running: Arc<AtomicBool>,
    controller: &ControllerType,
    cycle_length: u64,
) {
    let fan_on = true;
    let mut ign_on = true;
    let mut auger_on = false;

    // Turn Fan ON
    let _ = bridge.set_logical_pin_state("RELAY_FAN", HalPinState::High);
    // Turn Igniter ON
    let _ = bridge.set_logical_pin_state("RELAY_IGN", HalPinState::High);
    for _ in 0..5 {
        if !running.load(Ordering::SeqCst) { return; }
        print_relay_status(bridge, &[
            ("RELAY_FAN", fan_on),
            ("RELAY_IGN", ign_on),
            ("RELAY_AUG", auger_on),
        ]);
        thread::sleep(Duration::from_secs(1));
    }
    let _ = bridge.set_logical_pin_state("RELAY_IGN", HalPinState::Low);
    ign_on = false;
    for _ in 0..2 {
        if !running.load(Ordering::SeqCst) { return; }
        print_relay_status(bridge, &[
            ("RELAY_FAN", fan_on),
            ("RELAY_IGN", ign_on),
            ("RELAY_AUG", auger_on),
        ]);
        thread::sleep(Duration::from_secs(1));
    }

    // Main auger control loop
    while running.load(Ordering::SeqCst) {
        let u = controller.u().clamp(0.0, 1.0);
        let on_time = if let ControllerType::PMode(ctrl) = controller {
            ctrl.get_on_off_times(cycle_length).0
        } else {
            (cycle_length as f32 * u) as u64
        };
        let off_time = if let ControllerType::PMode(ctrl) = controller {
            ctrl.get_on_off_times(cycle_length).1
        } else {
            cycle_length - on_time
        };
        // Auger ON
        let _ = bridge.set_logical_pin_state("RELAY_AUG", HalPinState::High);
        auger_on = true;
        for _ in 0..on_time {
            if !running.load(Ordering::SeqCst) { return; }
            print_relay_status(bridge, &[
                ("RELAY_FAN", fan_on),
                ("RELAY_IGN", ign_on),
                ("RELAY_AUG", auger_on),
            ]);
            thread::sleep(Duration::from_secs(1));
        }
        // Auger OFF
        let _ = bridge.set_logical_pin_state("RELAY_AUG", HalPinState::Low);
        auger_on = false;
        for _ in 0..off_time {
            if !running.load(Ordering::SeqCst) { return; }
            print_relay_status(bridge, &[
                ("RELAY_FAN", fan_on),
                ("RELAY_IGN", ign_on),
                ("RELAY_AUG", auger_on),
            ]);
            thread::sleep(Duration::from_secs(1));
        }
    }
}
