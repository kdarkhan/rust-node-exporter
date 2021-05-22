#[macro_use]
extern crate lazy_static;
extern crate quick_xml;
extern crate serde;
mod helpers;

#[macro_use]
extern crate clap;

use clap::App;
use std::collections::HashSet;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::time::Instant;

const EXPORTER_NVIDIA: &str = "nvidia";
const EXPORTER_NZXT_AIO: &str = "nzxt_aio";
const EXPORTER_PROC_STAT: &str = "proc_stat";
const EXPORTER_HDDTEMP: &str = "hddtemp";
const EXPORTER_LM_SENSORS: &str = "lm_sensors";

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let port: i32 = matches.value_of("port").unwrap().parse().unwrap();

    let exporters: HashSet<&str> = if matches.is_present("exporters") {
        matches.values_of("exporters").unwrap().collect()
    } else {
        [
            EXPORTER_LM_SENSORS,
            EXPORTER_HDDTEMP,
            EXPORTER_PROC_STAT,
            EXPORTER_NZXT_AIO,
            EXPORTER_NVIDIA,
        ]
        .iter()
        .cloned()
        .collect()
    };

    let mut aio_metrics = helpers::nzxt_aio::get_aio_metrics();

    if exporters.contains(EXPORTER_NZXT_AIO) {
        aio_metrics.init();
    }

    let handle_connection = |mut stream: TcpStream| {
        let mut buffer = [0; 1024];

        stream.read(&mut buffer).unwrap();

        let mut result = String::new();
        if exporters.contains(EXPORTER_NZXT_AIO) {
            result.push_str(&aio_metrics.get_aio_metrics());
        }
        if exporters.contains(EXPORTER_LM_SENSORS) {
            result.push_str(&helpers::lm_sensors::get_lm_sensor_metrics());
        }
        if exporters.contains(EXPORTER_HDDTEMP) {
            result.push_str(&helpers::hddtemp::get_hddtemp_metrics());
        }
        if exporters.contains(EXPORTER_NVIDIA) {
            result.push_str(&helpers::nvidia::get_nvidia_metrics());
        }
        if exporters.contains(EXPORTER_PROC_STAT) {
            result.push_str(&helpers::proc_stat::get_proc_stat());
        }

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            result.len(),
            result
        );

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    };

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            let now = Instant::now();
            handle_connection(stream);
            println!("Responded in {} ms", now.elapsed().as_millis());
        }
    }
}
