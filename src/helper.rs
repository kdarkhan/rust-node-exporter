use regex::Regex;
use std::process::Command;
use std::str;

pub fn get_aio_metrics() -> String {
    lazy_static! {
        static ref LIQUID_TEMP_PATTERN: Regex = Regex::new(r"Liquid temperature\s+([0-9.]+)\s+°C$").unwrap();
        static ref FAN_SPEED_PATTERN: Regex = Regex::new(r"Fan speed\s+([0-9]+)\s+rpm$").unwrap();
        static ref PUMP_SPEED_PATTERN: Regex = Regex::new(r"Pump speed\s+([0-9]+)\s+rpm$").unwrap();
    }

    let mut temp = String::new();
    let mut fan_speed = String::new();
    let mut pump_speed = String::new();

    match Command::new("liquidctl").arg("status").output() {
        Ok(output) => match str::from_utf8(&output.stdout) {
            Ok(out) => {
                for line in out.lines() {
                    if let Some(m) = LIQUID_TEMP_PATTERN.captures(line) {
                        temp = m[1].to_string();
                    } else if let Some(m) = FAN_SPEED_PATTERN.captures(line) {
                        fan_speed = m[1].to_string();
                    } else if let Some(m) = PUMP_SPEED_PATTERN.captures(line) {
                        pump_speed = m[1].to_string();
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

pub fn get_lm_sensor_metric() -> String {

    lazy_static! {
        static ref TCTL_PATTERN: Regex = Regex::new(r"Tctl:\s+([0-9.+-]+)°C\s*$").unwrap();
        static ref TDIE_PATTERN: Regex = Regex::new(r"Tdie:\s+([0-9.+-]+)°C\s*$").unwrap();
        static ref CORE_VOLTAGE_PATTERN: Regex = Regex::new(r"CPU Core Voltage:\s+([0-9.]+)\s*V\s*$").unwrap();
        static ref CPU_TEMP_PATTERN: Regex = Regex::new(r"CPU Temperature:\s+([0-9.+-]+)°C\s*$").unwrap();
        static ref MB_TEMP_PATTERN: Regex = Regex::new(r"Motherboard Temperature:\s+([0-9.+-]+)°C\s*$").unwrap();
        static ref CHIPSET_TEMP_PATTERN: Regex = Regex::new(r"Chipset Temperature:\s+([0-9.+-]+)°C\s*$").unwrap();
        static ref CPU_FAN_PATTERN: Regex = Regex::new(r"CPU Fan:\s+([0-9]+) RPM\s*$").unwrap();
        static ref CHASSIS_FAN_PATTERN: Regex = Regex::new(r"Chassis Fan 2:\s+([0-9]+) RPM\s*$").unwrap();
    }

    let mut tctl = String::new();
    let mut tdie = String::new();
    let mut core_voltage = String::new();
    let mut cpu_temp = String::new();
    let mut mb_temp = String::new();
    let mut chipset_temp = String::new();
    let mut cpu_fan = String::new();
    let mut chassis_fan = String::new();

    match Command::new("sensors").output() {
        Ok(output) => match str::from_utf8(&output.stdout) {
            Ok(out) => {
                for line in out.lines() {
                    if let Some(m) = TCTL_PATTERN.captures(line) {
                        tctl = m[1].to_string();
                    } else if let Some(m) = TDIE_PATTERN.captures(line) {
                        tdie = m[1].to_string();
                    } else if let Some(m) = CORE_VOLTAGE_PATTERN.captures(line) {
                        core_voltage = m[1].to_string();
                    }  else if let Some(m) = CPU_TEMP_PATTERN.captures(line) {
                        cpu_temp = m[1].to_string();
                    } else if let Some(m) = MB_TEMP_PATTERN.captures(line) {
                        mb_temp = m[1].to_string();
                    } else if let Some(m) = CHIPSET_TEMP_PATTERN.captures(line) {
                        chipset_temp = m[1].to_string();
                    } else if let Some(m) = CPU_FAN_PATTERN.captures(line) {
                        cpu_fan = m[1].to_string();
                    } else if let Some(m) = CHASSIS_FAN_PATTERN.captures(line) {
                        chassis_fan = m[1].to_string();
                    }
                }
            }
            Err(e) => {
                eprintln!("error parsing stdout {}", e);
                panic!()
            }
        },
        Err(e) => {
            eprintln!("error running sensors {}", e);
            panic!()
        }
    }

    let mut result = String::new();
    result.push_str(&format!("\nlm_sensors_tctl {}", tctl));
    result.push_str(&format!("\nlm_sensors_tdie {}", tdie));
    result.push_str(&format!("\nlm_sensors_core_voltage {}", core_voltage));
    result.push_str(&format!("\nlm_sensors_cpu_temp {}", cpu_temp));
    result.push_str(&format!("\nlm_sensors_mb_temp {}", mb_temp));
    result.push_str(&format!("\nlm_sensors_chipset_temp {}", chipset_temp));
    result.push_str(&format!("\nlm_sensors_cpu_fan {}", cpu_fan));
    result.push_str(&format!("\nlm_sensors_chassis_fan {}\n", chassis_fan));
    result
}
