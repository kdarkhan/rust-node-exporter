#[macro_use]
extern crate lazy_static;
mod helper;

use std::env;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();

    let port: i32 = args.get(1).and_then(|x| x.parse().ok()).unwrap_or(7878);

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            let now = Instant::now();
            handle_connection(stream);
            println!("Responded in {} ms", now.elapsed().as_millis());
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    let mut result = helper::get_aio_metrics();
    result.push_str(&helper::get_lm_sensor_metrics());
    result.push_str(&helper::get_hddtemp_metrics());
    result.push_str(&helper::get_nvidia_metrics());

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
        result.len(),
        result
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
