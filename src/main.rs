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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

const EXPORTER_HDDTEMP: &str = "hddtemp";
const EXPORTER_LM_SENSORS: &str = "lm_sensors";
const EXPORTER_PROC_MEMINFO: &str = "proc_meminfo";
const EXPORTER_PROC_NETDEV: &str = "proc_netdev";
const EXPORTER_PROC_STAT: &str = "proc_stat";
const EXPORTER_NVIDIA: &str = "nvidia";
const EXPORTER_NZXT_AIO: &str = "nzxt_aio";

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
            EXPORTER_PROC_MEMINFO,
            EXPORTER_PROC_NETDEV,
            EXPORTER_PROC_STAT,
            EXPORTER_NZXT_AIO,
            EXPORTER_NVIDIA,
        ]
        .iter()
        .cloned()
        .collect()
    };

    let mut aio_metrics = helpers::nzxt_aio::get_aio_metrics();
    let mut lm_sensors = helpers::lm_sensors::get_lm_sensors();

    let should_run = Arc::new(AtomicBool::new(true));
    let r = should_run.clone();

    let listener =
        TcpListener::bind(format!("127.0.0.1:{}", port)).expect("Could not bind to address");

    let listener_addr = listener
        .local_addr()
        .expect("Could not get listener address");

    ctrlc::set_handler(move || {
        println!("Setting flag for termination from CTRLC handler");
        r.store(false, Ordering::SeqCst);
        // hackily wake up the listener thread
        let _ = TcpStream::connect(listener_addr);
    })
    .expect("Error setting Ctrl-C handler");

    if exporters.contains(EXPORTER_NZXT_AIO) {
        aio_metrics.init();
    }
    if exporters.contains(EXPORTER_LM_SENSORS) {
        lm_sensors.init();
    }

    let handle_connection = |mut stream: TcpStream| {
        let mut buffer = [0; 1024];

        stream.read(&mut buffer).unwrap();

        let mut result = String::new();
        if exporters.contains(EXPORTER_NZXT_AIO) {
            result.push_str(&aio_metrics.get_aio_metrics());
        }
        if exporters.contains(EXPORTER_LM_SENSORS) {
            // result.push_str(&helpers::lm_sensors::get_lm_sensor_metrics());
            result.push_str(&lm_sensors.get_lm_sensor_metrics());
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
        if exporters.contains(EXPORTER_PROC_NETDEV) {
            result.push_str(&helpers::proc_netdev::get_proc_netdev());
        }
        if exporters.contains(EXPORTER_PROC_MEMINFO) {
            result.push_str(&helpers::proc_meminfo::get_proc_memifo());
        }

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            result.len(),
            result
        );

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    };

    for stream in listener.incoming() {
        if !should_run.load(Ordering::SeqCst) {
            println!("CTRLC detected, terminating");
            return;
        }
        if let Ok(stream) = stream {
            let now = Instant::now();
            handle_connection(stream);
            println!("Responded in {} ms", now.elapsed().as_millis());
        }
    }
}
