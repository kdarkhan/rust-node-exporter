use std::process::Command;
use std::str;

use serde::Deserialize;

#[derive(Deserialize)]
struct Sensors {
    #[serde(rename = "k10temp-pci-00c3")]
    k10temp: K10Temp,
    #[serde(rename = "asuswmisensors-isa-0000")]
    asus_wmi_sensors: AsusWmiSensors,
}

#[derive(Deserialize)]
struct K10Temp {
    #[serde(rename = "Tctl")]
    tctl: F64Wrapper,
    #[serde(rename = "Tdie")]
    tdie: F64Wrapper,
}

#[derive(Deserialize)]
struct AsusWmiSensors {
    #[serde(rename = "CPU Core Voltage")]
    core_voltage: F64Wrapper,
    #[serde(rename = "+12V Voltage")]
    plus_12_voltage: F64Wrapper,
    #[serde(rename = "+5V Voltage")]
    plus_5_voltage: F64Wrapper,
    #[serde(rename = "3VSB Voltage")]
    three_vsb_voltage: F64Wrapper,
    #[serde(rename = "CPU Fan")]
    cpu_fan: F64Wrapper,
    #[serde(rename = "Chassis Fan 2")]
    chassis_fan_2: F64Wrapper,
    #[serde(rename = "CPU Temperature")]
    cpu_temperature: F64Wrapper,
    #[serde(rename = "Motherboard Temperature")]
    mb_temperature: F64Wrapper,
    #[serde(rename = "Chipset Temperature")]
    chipset_temperature: F64Wrapper,
}

#[derive(Deserialize)]
struct F64Wrapper {
    #[serde(
        alias = "temp1_input",
        alias = "temp2_input",
        alias = "temp3_input",
        alias = "temp4_input",
        alias = "in0_input",
        alias = "in1_input",
        alias = "in2_input",
        alias = "in3_input",
        alias = "fan1_input",
        alias = "fan2_input",
        alias = "fan3_input",
        alias = "fan4_input"
    )]
    value: f64,
}

pub fn get_lm_sensor_metrics() -> String {
    match Command::new("sensors").arg("-j").output() {
        Ok(output) => {
            let sens: Sensors = serde_json::from_slice(&output.stdout).unwrap();

            let mut result = String::new();
            result.push_str(&format!("lm_sensors_tctl {}", sens.k10temp.tctl.value));
            result.push_str(&format!("\nlm_sensors_tdie {}", sens.k10temp.tdie.value));
            result.push_str(&format!(
                "\nlm_sensors_cpu_temp {}",
                sens.asus_wmi_sensors.cpu_temperature.value
            ));
            result.push_str(&format!(
                "\nlm_sensors_mb_temp {}",
                sens.asus_wmi_sensors.mb_temperature.value
            ));
            result.push_str(&format!(
                "\nlm_sensors_chipset_temp {}",
                sens.asus_wmi_sensors.chipset_temperature.value
            ));
            result.push_str(&format!(
                "\nlm_sensors_cpu_fan {}",
                sens.asus_wmi_sensors.cpu_fan.value
            ));
            result.push_str(&format!(
                "\nlm_sensors_chassis_fan {}",
                sens.asus_wmi_sensors.chassis_fan_2.value
            ));
            result.push_str(&format!(
                "\nlm_sensors_core_voltage {}",
                sens.asus_wmi_sensors.core_voltage.value
            ));
            result.push_str(&format!(
                "\nlm_sensors_plus_5_voltage {}",
                sens.asus_wmi_sensors.plus_5_voltage.value
            ));
            result.push_str(&format!(
                "\nlm_sensors_plus_12_voltage {}",
                sens.asus_wmi_sensors.plus_12_voltage.value
            ));
            result.push_str(&format!(
                "\nlm_sensors_3vsb_voltage {}\n",
                sens.asus_wmi_sensors.three_vsb_voltage.value
            ));
            return result;
        }
        Err(e) => {
            eprintln!("error running lm_sensors {}", e);
            panic!()
        }
    }
}
