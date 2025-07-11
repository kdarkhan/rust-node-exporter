extern crate quick_xml;
extern crate serde;
mod helpers;

use clap::{Parser, ValueEnum};
use helpers::amd_radeontop;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[derive(Parser)]
#[clap(author, version, about)]
#[clap(about = "Prometheus exporter for my desktop metrics")]
struct Cli {
    /// Port where the HTTP server is listening
    #[arg(default_value_t = 7878, short = 'p')]
    port: u32,

    /// List of enabled exporters (all are enabled if none provided)
    #[arg(value_enum, short = 'x')]
    exporters: Vec<Exporter>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[clap(rename_all = "snake_case")]
enum Exporter {
    Hddtemp,
    LmSensors,
    ProcMeminfo,
    ProcNetdev,
    ProcStat,
    Nvidia,
    NzxtAio, // No longer needed because these sensors are exposed in lm_sensors
    AmdRadeontop,
}

fn main() {
    let cli = Cli::parse();

    let port: u32 = cli.port;
    let mut exporters: Vec<Exporter> = cli.exporters;
    if exporters.is_empty() {
        // default exporters to enable
        exporters = vec![
            Exporter::Hddtemp,
            Exporter::LmSensors,
            Exporter::ProcMeminfo,
            Exporter::ProcNetdev,
            Exporter::ProcStat,
            Exporter::AmdRadeontop,
        ]
    }

    let mut lm_sensors = helpers::lm_sensors::get_lm_sensors();

    let should_run = Arc::new(AtomicBool::new(true));
    let r = should_run.clone();

    let listener =
        TcpListener::bind(format!("127.0.0.1:{port}")).expect("Could not bind to address");

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

    if exporters.contains(&Exporter::LmSensors) {
        lm_sensors.init();
    }

    if exporters.contains(&Exporter::AmdRadeontop) {
        amd_radeontop::init();
    }

    let mut handle_connection = |mut stream: TcpStream| {
        let mut buffer = [0; 1024];

        let bytes_read = stream.read(&mut buffer).unwrap();
        println!("Read bytes {bytes_read}");

        let mut result = String::new();
        if exporters.contains(&Exporter::NzxtAio) {
            result.push_str(&helpers::nzxt_aio::get_aio_metrics());
        }
        if exporters.contains(&Exporter::LmSensors) {
            result.push_str(&lm_sensors.get_lm_sensor_metrics());
        }
        if exporters.contains(&Exporter::Hddtemp) {
            result.push_str(&helpers::hddtemp::get_hddtemp_metrics());
        }
        if exporters.contains(&Exporter::Nvidia) {
            result.push_str(&helpers::nvidia::get_nvidia_metrics());
        }
        if exporters.contains(&Exporter::ProcStat) {
            result.push_str(&helpers::proc_stat::get_proc_stat());
        }
        if exporters.contains(&Exporter::ProcNetdev) {
            result.push_str(&helpers::proc_netdev::get_proc_netdev());
        }
        if exporters.contains(&Exporter::ProcMeminfo) {
            result.push_str(&helpers::proc_meminfo::get_proc_memifo());
        }

        if exporters.contains(&Exporter::AmdRadeontop) {
            result.push_str(&helpers::amd_radeontop::get_radeontop_stats());
        }

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4;\r\nContent-Length: {}\r\n\r\n{}",
            result.len(),
            result
        );

        stream.write_all(response.as_bytes()).unwrap();
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
