use regex::Regex;
use serde_json::Value;
use std::process::Command;
use std::str;

pub fn get_aio_metrics() -> String {
    lazy_static! {
        static ref LIQUID_TEMP_PATTERN: Regex =
            Regex::new(r"Liquid temperature\s+([0-9.]+)\s+°C$").unwrap();
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
    match Command::new("sensors").arg("-j").output() {
        Ok(output) => match str::from_utf8(&output.stdout) {
            Ok(out) => {
                let v: Value = serde_json::from_str(out).expect("sensors -j did not return json");
                let tctl = v["k10temp-pci-00c3"]["Tctl"]["temp1_input"]
                    .as_f64()
                    .unwrap_or(0.0);
                let tdie = v["k10temp-pci-00c3"]["Tdie"]["temp2_input"]
                    .as_f64()
                    .unwrap_or(0.0);

                let core_voltage = v["asuswmisensors-isa-0000"]["CPU Core Voltage"]["in0_input"]
                    .as_f64()
                    .unwrap_or(0.0);
                let cpu_temp = v["asuswmisensors-isa-0000"]["CPU Temperature"]["temp1_input"]
                    .as_f64()
                    .unwrap_or(0.0);
                let mb_temp = v["asuswmisensors-isa-0000"]["Motherboard Temperature"]
                    ["temp2_input"]
                    .as_f64()
                    .unwrap_or(0.0);
                let chipset_temp = v["asuswmisensors-isa-0000"]["Chipset Temperature"]
                    ["temp3_input"]
                    .as_f64()
                    .unwrap_or(0.0);
                let cpu_fan = v["asuswmisensors-isa-0000"]["CPU Fan"]["fan1_input"]
                    .as_f64()
                    .unwrap_or(0.0);
                let chassis_fan = v["asuswmisensors-isa-0000"]["Chassis Fan 2"]["fan3_input"]
                    .as_f64()
                    .unwrap_or(0.0);

                let mut result = String::new();
                result.push_str(&format!("\nlm_sensors_tctl {}", tctl));
                result.push_str(&format!("\nlm_sensors_tdie {}", tdie));
                result.push_str(&format!("\nlm_sensors_core_voltage {}", core_voltage));
                result.push_str(&format!("\nlm_sensors_cpu_temp {}", cpu_temp));
                result.push_str(&format!("\nlm_sensors_mb_temp {}", mb_temp));
                result.push_str(&format!("\nlm_sensors_chipset_temp {}", chipset_temp));
                result.push_str(&format!("\nlm_sensors_cpu_fan {}", cpu_fan));
                result.push_str(&format!("\nlm_sensors_chassis_fan {}\n", chassis_fan));
                return result;
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
}
