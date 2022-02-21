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

struct SensorValueWrapper {
    name: *const sensors_chip_name,
    subfeature_number: i32,
}

pub struct LmSensors {
    subfeatures: HashMap<String, SensorValueWrapper>,
}

pub fn get_lm_sensors() -> LmSensors {
    return LmSensors {
        subfeatures: HashMap::new(),
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
            ("asuswmisensors/temp1_input", "lm_sensors_cpu_temp"),
            ("asuswmisensors/temp2_input", "lm_sensors_mb_temp"),
            ("asuswmisensors/temp3_input", "lm_sensors_chipset_temp"),
            ("asuswmisensors/fan1_input", "lm_sensors_cpu_fan"),
            ("asuswmisensors/fan3_input", "lm_sensors_chassis_fan"),
            ("asuswmisensors/in0_input", "lm_sensors_core_voltage"),
            ("asuswmisensors/in2_input", "lm_sensors_plus_5_voltage"),
            ("asuswmisensors/in1_input", "lm_sensors_plus_12_voltage"),
            ("asuswmisensors/in3_input", "lm_sensors_plus_3vsb_voltage"),
            // NZXT kraken2 data
            ("kraken2/temp1_input", "aio_liquid_temp"),
            ("kraken2/fan1_input", "aio_fan_speed"),
            ("kraken2/fan2_input", "aio_pump_speed"),
        ]
        .into_iter()
        .collect();

        unsafe {
            println!("lm_sensors initialization started");
            if sensors_init(ptr::null_mut()) != 0 {
                panic!("lm_sensors init failed");
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
    }

    pub fn get_lm_sensor_metrics(&self) -> String {
        let mut result = String::new();
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
