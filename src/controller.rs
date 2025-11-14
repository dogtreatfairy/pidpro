use serde_json::{self, json, Value};
use std::fs;

struct PIDConfig{
    pb: i32,
    ti: i32,
    td: i32,
}

struct PIDCycle {
    cycle_time: i32,
    u_min: i32,
    u_max: i32,
}

struct PIDGains {
    kp: i32,
    ki: i32,
    kd: i32,
}

pub struct Controller {
    pub active_controller: String,
    config: PIDConfig,
    cycle: PIDCycle,
    gains: PIDGains,
}

impl Controller {
    pub fn load() -> Controller {
        let config_data = std::fs::read_to_string("config.json")
            .expect("Failed to read config.json");
        let json: serde_json::Value = serde_json::from_str(&config_data)
            .expect("Failed to parse JSON");
        
        let active_controller = json["config"]["active_controller"]
            .as_str()
            .expect("Failed to parse active_controller")
            .to_string();
        
        let controller_config = &json["config"]["controllers"][&active_controller];
        
        let config = PIDConfig {
            pb: controller_config["pb"].as_i64().unwrap_or(0) as i32,
            ti: controller_config["ti"].as_i64().unwrap_or(0) as i32,
            td: controller_config["td"].as_i64().unwrap_or(0) as i32,
        };
        
        let cycle = PIDCycle {
            cycle_time: controller_config["cycle_time"].as_i64().unwrap_or(0) as i32,
            u_min: controller_config["u_min"].as_i64().unwrap_or(0) as i32,
            u_max: controller_config["u_max"].as_i64().unwrap_or(0) as i32,
        };
        
        let gains = Controller::calculate_gains(&config);

        Controller { active_controller, config, cycle, gains }
    }
    pub fn set(active: String){
        
        // Read the config file
        let  config_data = std::fs::read_to_string("config.json")
            .expect("Failed to read config.json");
        let mut json: serde_json::Value = serde_json::from_str(&config_data)
            .expect("Failed to parse JSON");
        
        // Update the active_controller in JSON
        json["config"]["active_controller"] = serde_json::json!(active);
        
        // Write back to file
        std::fs::write("config.json", json.to_string())
            .expect("Failed to write config.json");

        Controller::load();
    }
    fn calculate_gains(config: &PIDConfig) -> PIDGains {
        let kp = config.pb;
        let ki = if config.ti != 0 { kp / config.ti } else { 0 };
        let kd = kp * config.td;
        PIDGains { kp, ki, kd }
    }
    pub fn update(&mut self) -> i32 {
        // Update output
        let u = 0;
        return u; 
    }
    pub fn print() {
        // Load the current controller from config and print its values
        let c = Controller::load();
        println!("Active Controller: {}", c.active_controller);
        println!("PB: {}", c.config.pb);
        println!("Ti: {}", c.config.ti);
        println!("Td: {}", c.config.td);
        println!("Cycle Time: {}", c.cycle.cycle_time);
        println!("U Min: {}", c.cycle.u_min);
        println!("U Max: {}", c.cycle.u_max);
    }
}