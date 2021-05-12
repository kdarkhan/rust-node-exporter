use regex::Regex;
use std::process::Command;
use std::str;

pub fn get_response_string() -> String {
    let temp_pattern = Regex::new(r"Liquid temperature\s+([0-9.]+)\s+°C$").unwrap();
    let fan_speed_pattern = Regex::new(r"Fan speed\s+([0-9.]+)\s+rpm$").unwrap();
    let pump_speed_pattern = Regex::new(r"Pump speed\s+([0-9.]+)\s+rpm$").unwrap();

    let mut temp = String::from("NA");
    let mut fan_speed = String::from("NA");
    let mut pump_speed = String::from("NA");

    match Command::new("liquidctl").arg("status").output() {
        Ok(output) => match str::from_utf8(&output.stdout) {
            Ok(out) => {
                for line in out.lines() {
                    if let Some(cap) = temp_pattern.captures(line) {
                        temp = cap[1].to_string();
                    } else if let Some(cap) = fan_speed_pattern.captures(line) {
                        fan_speed = cap[1].to_string();
                    } else if let Some(cap) = pump_speed_pattern.captures(line) {
                        pump_speed = cap[1].to_string();
                    }
                }
            }
            Err(e) => {
                eprintln!("error parsing stdout {}", e)
            }
        },
        Err(e) => {
            eprintln!("error running liquidctl {}", e)
        }
    }

    format!(
        "aio_liquid_temp {}\naio_fan_speed {}\naio_pump_speed {}",
        temp, fan_speed, pump_speed
    )
}
