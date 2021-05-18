use regex::Regex;
use serde_json::Value;
use std::io::prelude::*;
use std::net::TcpStream;
use std::process::Command;
use std::str;
use xmltree::Element;

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

fn get_text_at_path(e: &Element, args: &[&str]) -> String {
    let mut x = e;
    for p in args {
        x = x.get_child(*p).unwrap();
    }

    x.get_text().unwrap().split_whitespace().next().unwrap().to_owned()
}


pub fn get_nvidia_metrics() -> String {
    match Command::new("nvidia-smi").arg("-q").arg("-x").output() {
        Ok(output) => match str::from_utf8(&output.stdout) {
            Ok(out) => {
                let parsed = Element::parse(out.as_bytes()).unwrap();
                let gpu = parsed.get_child("gpu").unwrap();

                let gpu_temp = get_text_at_path(gpu, &["temperature", "gpu_temp"]);
                let power_draw = get_text_at_path(gpu, &["power_readings", "power_draw"]);

                let clocks = gpu.get_child("clocks").unwrap();
                let graphics_clock = get_text_at_path(clocks, &["graphics_clock"]);
                let sm_clock = get_text_at_path(clocks, &["sm_clock"]);
                let mem_clock = get_text_at_path(clocks, &["mem_clock"]);
                let video_clock = get_text_at_path(clocks, &["video_clock"]);


                let fan_speed = get_text_at_path(gpu, &["fan_speed"]);


                let fb_memory = gpu.get_child("fb_memory_usage").unwrap();
                let fb_memory_total = get_text_at_path(fb_memory, &["total"]);
                let fb_memory_used = get_text_at_path(fb_memory, &["used"]);
                let fb_memory_free = get_text_at_path(fb_memory, &["free"]);

                let bar1_memory = gpu.get_child("bar1_memory_usage").unwrap();
                let bar1_memory_total = get_text_at_path(bar1_memory, &["total"]);
                let bar1_memory_used = get_text_at_path(bar1_memory, &["used"]);
                let bar1_memory_free = get_text_at_path(bar1_memory, &["free"]);

                let utilization = gpu.get_child("utilization").unwrap();
                let utilization_gpu = get_text_at_path(utilization, &["gpu_util"]);
                let utilization_mem = get_text_at_path(utilization, &["memory_util"]);
                let utilization_enc = get_text_at_path(utilization, &["encoder_util"]);
                let utilization_dec = get_text_at_path(utilization, &["decoder_util"]);


                let mut result = String::new();
                result.push_str(&format!("\nnvidia_temp {}", gpu_temp));
                result.push_str(&format!("\nnvidia_power_draw {}", power_draw));
                result.push_str(&format!("\nnvidia_graphics_clock {}", graphics_clock));
                result.push_str(&format!("\nnvidia_sm_clock {}", sm_clock));
                result.push_str(&format!("\nnvidia_mem_clock {}", mem_clock));
                result.push_str(&format!("\nnvidia_video_clock {}", video_clock));
                result.push_str(&format!("\nnvidia_fan_speed {}", fan_speed));
                result.push_str(&format!("\nnvidia_fb_memory_total {}", fb_memory_total));
                result.push_str(&format!("\nnvidia_fb_memory_free {}", fb_memory_free));
                result.push_str(&format!("\nnvidia_fb_memory_used {}", fb_memory_used));
                result.push_str(&format!("\nnvidia_bar1_memory_total {}", bar1_memory_total));
                result.push_str(&format!("\nnvidia_bar1_memory_free {}", bar1_memory_free));
                result.push_str(&format!("\nnvidia_bar1_memory_used {}", bar1_memory_used));
                result.push_str(&format!("\nnvidia_utilization_gpu {}", utilization_gpu));
                result.push_str(&format!("\nnvidia_utilization_mem {}", utilization_mem));
                result.push_str(&format!("\nnvidia_utilization_enc {}", utilization_enc));
                result.push_str(&format!("\nnvidia_utilization_dec {}", utilization_dec));
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
