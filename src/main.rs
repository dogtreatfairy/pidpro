//src main.rs
mod controller;
use controller::Controller;

fn main () {

    Controller::set("pid_sec".to_string());
    Controller::print();
    println!("-----CHANGING CONTROLLER-----");
    Controller::set("pid_std".to_string());
    Controller::print();
}