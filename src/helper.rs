use regex::Regex;
use serde_json::Value;
use std::io::prelude::*;
use std::net::TcpStream;
use std::process::Command;
use std::str;

use quick_xml::de::from_str;
use serde::Deserialize;

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

pub fn get_hddtemp_metrics() -> String {
    // hddtemp service is listening on port 7634
    let mut stream =
        TcpStream::connect("127.0.0.1:7634").expect("could not connect to hddtemp service");

    lazy_static! {
        static ref HDDTEMP_PATTERN: Regex = Regex::new(
            r"\|CT1000MX500SSD4\|([0-9.]+)\|C\|.+\|Samsung SSD 860 EVO M.2\|([0-9.]+)\|C\|.*$"
        )
        .unwrap();
    }

    let mut v: Vec<u8> = Vec::new();
    stream
        .read_to_end(&mut v)
        .expect("hddtemp service did not respond properly");
    let res = String::from_utf8(v).expect("could not parse");

    if let Some(m) = HDDTEMP_PATTERN.captures(&res) {
        format!(
            "\nhddtemp_crucial_mx500_temp {}\nhddtemp_samsung_860_evo_temp {}",
            m[1].to_string(),
            m[2].to_string()
        )
    } else {
        panic!("Could not parse hddtemp output")
    }
}

fn get_first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap()
}

#[derive(Debug, Deserialize, PartialEq)]
struct NvidiaSmiLog {
    gpu: Gpu,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Gpu {
    fan_speed: String,
    temperature: Temperature,
    power_readings: PowerReadings,
    clocks: Clocks,
    bar1_memory_usage: MemoryUsage,
    fb_memory_usage: MemoryUsage,
    utilization: Utilization,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Temperature {
    gpu_temp: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct PowerReadings {
    power_draw: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Clocks {
    graphics_clock: String,
    sm_clock: String,
    mem_clock: String,
    video_clock: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct MemoryUsage {
    total: String,
    used: String,
    free: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Utilization {
    gpu_util: String,
    memory_util: String,
    encoder_util: String,
    decoder_util: String,
}

pub fn get_nvidia_metrics() -> String {
    match Command::new("nvidia-smi").arg("-q").arg("-x").output() {
        Ok(output) => match str::from_utf8(&output.stdout) {
            Ok(out) => {
                let log: NvidiaSmiLog = from_str(out).unwrap();
                let gpu = log.gpu;

                let mut result = String::new();
                result.push_str(&format!(
                    "\nnvidia_temp {}",
                    get_first_word(&gpu.temperature.gpu_temp)
                ));
                result.push_str(&format!(
                    "\nnvidia_power_draw {}",
                    get_first_word(&gpu.power_readings.power_draw)
                ));
                result.push_str(&format!(
                    "\nnvidia_graphics_clock {}",
                    get_first_word(&gpu.clocks.graphics_clock)
                ));
                result.push_str(&format!(
                    "\nnvidia_sm_clock {}",
                    get_first_word(&gpu.clocks.sm_clock)
                ));
                result.push_str(&format!(
                    "\nnvidia_mem_clock {}",
                    get_first_word(&gpu.clocks.mem_clock)
                ));
                result.push_str(&format!(
                    "\nnvidia_video_clock {}",
                    get_first_word(&gpu.clocks.video_clock)
                ));
                result.push_str(&format!(
                    "\nnvidia_fan_speed {}",
                    get_first_word(&gpu.fan_speed)
                ));
                result.push_str(&format!(
                    "\nnvidia_fb_memory_total {}",
                    get_first_word(&gpu.fb_memory_usage.total)
                ));
                result.push_str(&format!(
                    "\nnvidia_fb_memory_free {}",
                    get_first_word(&gpu.fb_memory_usage.free)
                ));
                result.push_str(&format!(
                    "\nnvidia_fb_memory_used {}",
                    get_first_word(&gpu.fb_memory_usage.used)
                ));
                result.push_str(&format!(
                    "\nnvidia_bar1_memory_total {}",
                    get_first_word(&gpu.bar1_memory_usage.total)
                ));
                result.push_str(&format!(
                    "\nnvidia_bar1_memory_free {}",
                    get_first_word(&gpu.bar1_memory_usage.free)
                ));
                result.push_str(&format!(
                    "\nnvidia_bar1_memory_used {}",
                    get_first_word(&gpu.bar1_memory_usage.used)
                ));
                result.push_str(&format!(
                    "\nnvidia_utilization_gpu {}",
                    get_first_word(&gpu.utilization.gpu_util)
                ));
                result.push_str(&format!(
                    "\nnvidia_utilization_mem {}",
                    get_first_word(&gpu.utilization.memory_util)
                ));
                result.push_str(&format!(
                    "\nnvidia_utilization_enc {}",
                    get_first_word(&gpu.utilization.encoder_util)
                ));
                result.push_str(&format!(
                    "\nnvidia_utilization_dec {}",
                    get_first_word(&gpu.utilization.decoder_util)
                ));
                result
            }
            Err(e) => {
                eprintln!("error parsing nvidia-smi stdout {}", e);
                panic!()
            }
        },
        Err(e) => {
            eprintln!("error running nvidia-smi {}", e);
            panic!()
        }
    }
}
