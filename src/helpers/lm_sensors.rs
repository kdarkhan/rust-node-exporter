use serde_json::Value;
use std::process::Command;
use std::str;

pub fn get_lm_sensor_metrics() -> String {
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
                let plus_5_voltage = v["asuswmisensors-isa-0000"]["+5V Voltage"]["in2_input"]
                    .as_f64()
                    .unwrap_or(0.0);
                let plus_12_voltage = v["asuswmisensors-isa-0000"]["+12V Voltage"]["in1_input"]
                    .as_f64()
                    .unwrap_or(0.0);
                let three_vsb_voltage = v["asuswmisensors-isa-0000"]["3VSB Voltage"]["in3_input"]
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
                result.push_str(&format!("\nlm_sensors_cpu_temp {}", cpu_temp));
                result.push_str(&format!("\nlm_sensors_mb_temp {}", mb_temp));
                result.push_str(&format!("\nlm_sensors_chipset_temp {}", chipset_temp));
                result.push_str(&format!("\nlm_sensors_cpu_fan {}", cpu_fan));
                result.push_str(&format!("\nlm_sensors_chassis_fan {}", chassis_fan));
                result.push_str(&format!("\nlm_sensors_core_voltage {}", core_voltage));
                result.push_str(&format!("\nlm_sensors_plus_5_voltage {}", plus_5_voltage));
                result.push_str(&format!("\nlm_sensors_plus_12_voltage {}", plus_12_voltage));
                result.push_str(&format!("\nlm_sensors_3vsb_voltage {}", three_vsb_voltage));
                return result;
            }
            Err(e) => {
                eprintln!("error parsing lm_sensors stdout {}", e);
                panic!()
            }
        },
        Err(e) => {
            eprintln!("error running lm_sensors {}", e);
            panic!()
        }
    }
}
