//src main.rs
use serde_json;

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

struct Controller {
    config: PIDConfig,
    cycle: PIDCycle,
    gains: PIDGains,
}

impl Controller {
    fn new(config: PIDConfig, cycle: PIDCycle, gains: PIDGains) -> Self {
        Controller { config, cycle, gains }
    }
    fn calculate_gains(&self) -> PIDGains {
        let kp = self.config.pb;
        let ki = kp / self.config.ti;
        let kd = kp * self.config.td;
        PIDGains { kp, ki, kd }
    }
    fn update(&mut self) -> i32 {
        // Update output
        let u = 0;
        return u; 
    }
}

fn main () {
    println!("Hello, world!");
}