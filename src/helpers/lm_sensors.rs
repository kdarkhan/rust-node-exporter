#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
// There is a lot of code in bindings.rs which is unused
#![allow(dead_code)]
// include libsensors bindings generated by bindgen
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw;
use std::ptr;
use std::str;

const FEATURE_INIT_COUNT: i32 = 10;

struct SensorValueWrapper {
    name: *const sensors_chip_name,
    subfeature_number: i32,
}

pub struct LmSensors {
    subfeatures: HashMap<String, SensorValueWrapper>,
    init_count: i32,
}

pub fn get_lm_sensors() -> LmSensors {
    return LmSensors {
        subfeatures: HashMap::new(),
        init_count: 0,
    };
}

impl Drop for LmSensors {
    fn drop(&mut self) {
        if !self.subfeatures.is_empty() {
            println!("executing lm_sensors cleanup");
            unsafe {
                sensors_cleanup();
            }
        }
    }
}

impl LmSensors {
    pub fn init(&mut self) {
        let mut subfeature_map: HashMap<String, SensorValueWrapper> = HashMap::new();

        let relevant_keys: HashMap<&str, &str> = vec![
            ("k10temp/temp1_input", "lm_sensors_tctl"),
            ("k10temp/temp2_input", "lm_sensors_tdie"),
            ("asus_wmi_sensors/temp1_input", "lm_sensors_cpu_temp"),
            ("asus_wmi_sensors/temp2_input", "lm_sensors_mb_temp"),
            ("asus_wmi_sensors/temp3_input", "lm_sensors_chipset_temp"),
            ("asus_wmi_sensors/fan1_input", "lm_sensors_cpu_fan"),
            ("asus_wmi_sensors/fan3_input", "lm_sensors_chassis_fan"),
            ("asus_wmi_sensors/fan4_input", "lm_sensors_chassis_fan_3"),
            ("asus_wmi_sensors/in0_input", "lm_sensors_core_voltage"),
            ("asus_wmi_sensors/in2_input", "lm_sensors_plus_5_voltage"),
            ("asus_wmi_sensors/in1_input", "lm_sensors_plus_12_voltage"),
            ("asus_wmi_sensors/in3_input", "lm_sensors_plus_3vsb_voltage"),
            // NZXT kraken2 data
            ("kraken2/temp1_input", "aio_liquid_temp"),
            ("kraken2/fan1_input", "aio_fan_speed"),
            ("kraken2/fan2_input", "aio_pump_speed"),
            // amdgpu sensors
            ("amdgpu/in0_input", "amdgpu_vddgfx_voltage"),
            ("amdgpu/fan1_input", "amdgpu_fan1_speed"),
            ("amdgpu/temp2_input", "amdgpu_junction_temp"),
            ("amdgpu/temp3_input", "amdgpu_mem_temp"),
            ("amdgpu/power1_average", "amdgpu_slowppt_wattage"),
            // wireless adapater
            ("iwlwifi_1/temp1_input", "wireless_temp"),
            // logitech mouse voltage
            ("hidpp_battery_0/in0_input", "mouse_battery_voltage"),
        ]
        .into_iter()
        .collect();

        unsafe {
            println!("lm_sensors initialization started");
            if self.init_count == 0 {
                println!("calling sensors_init(null), should happen only once");
                if sensors_init(ptr::null_mut()) != 0 {
                    panic!("lm_sensors init failed");
                }
            }

            let mut chip_next: raw::c_int = 0;
            let chip_next_ptr: *mut raw::c_int = &mut chip_next;
            loop {
                let chip = sensors_get_detected_chips(ptr::null_mut(), chip_next_ptr);
                if chip == ptr::null() {
                    break;
                }

                let prefix = CStr::from_ptr((*chip).prefix).to_str().unwrap();
                let path = CStr::from_ptr((*chip).path).to_str().unwrap();
                println!("Found chip with prefix {} and path {}", prefix, path);

                let mut feature_next: raw::c_int = 0;
                let feature_next_ptr: *mut raw::c_int = &mut feature_next;

                loop {
                    let feature = sensors_get_features(chip, feature_next_ptr);
                    if feature == ptr::null() {
                        break;
                    }

                    let feature_name = CStr::from_ptr((*feature).name).to_str().unwrap();
                    println!("Found feature {} from chip {}", feature_name, prefix);

                    let mut subfeature_next: raw::c_int = 0;
                    let subfeature_next_ptr: *mut raw::c_int = &mut subfeature_next;

                    loop {
                        let subfeature =
                            sensors_get_all_subfeatures(chip, feature, subfeature_next_ptr);
                        if subfeature == ptr::null() {
                            break;
                        }
                        let subfeature_name = CStr::from_ptr((*subfeature).name).to_str().unwrap();
                        println!(
                            "\t\tFound subfeature_name {} with number {}",
                            subfeature_name,
                            (*subfeature).number
                        );

                        if ((*subfeature).flags & SENSORS_MODE_R) != 0 {
                            let key = format!("{}/{}", prefix, subfeature_name);
                            if relevant_keys.contains_key(key.as_str()) {
                                println!("Found interesting key {}/{}", prefix, subfeature_name);
                                subfeature_map.insert(
                                    relevant_keys.get(key.as_str()).unwrap().to_string(),
                                    SensorValueWrapper {
                                        name: chip,
                                        subfeature_number: (*subfeature).number,
                                    },
                                );
                            } else {
                                println!("Ignoring key {}/{}", prefix, subfeature_name);
                            }
                        } else {
                            println!("Ignoring non-readable sensor");
                        }
                    }
                }
            }
            println!("lm_sensors initialization complete");
        }
        self.subfeatures = subfeature_map;
        self.init_count += 1;
    }

    pub fn get_lm_sensor_metrics(&mut self) -> String {
        let mut result = String::new();
        if self.init_count < FEATURE_INIT_COUNT {
            self.init()
        }
        for (key, subfeature) in self.subfeatures.iter() {
            let mut value = 0f64;
            let value_ptr: *mut f64 = &mut value;
            unsafe {
                if sensors_get_value(subfeature.name, subfeature.subfeature_number, value_ptr) != 0
                {
                    panic!("Could not get sensor value")
                }
            }
            result.push_str(&format!("{} {}\n", key, value));
        }
        result
    }
}
