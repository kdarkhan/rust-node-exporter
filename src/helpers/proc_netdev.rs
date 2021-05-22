use std::fs::File;
use std::io;
use std::io::prelude::*;

lazy_static! {
    static ref IFACE_FIELD_MAP: &'static [&'static str] = &[
        "rx",
        "rx_packets",
        "rx_errs",
        "rx_drop",
        "rx_fifo",
        "rx_frame",
        "rx_compressed",
        "rx_multicast",
        "tx",
        "tx_packets",
        "tx_errs",
        "tx_drop",
        "tx_fifo",
        "tx_colls",
        "tx_carrier",
        "tx_compressed",
    ];
}

pub fn get_proc_netdev() -> String {
    let file = File::open("/proc/net/dev").expect("cannot open /proc/net/dev");
    let lines = io::BufReader::new(file).lines();

    let mut result = String::new();

    for line in lines {
        if let Ok(line) = line {
            let mut iter = line.split_ascii_whitespace();

            let iface_name = iter.next().unwrap();

            if iface_name.ends_with(":") {
                let stuff: Vec<&str> = iter.collect();
                if stuff.len() == 16 {
                    for (idx, item) in stuff.iter().enumerate() {
                        result.push_str(&format!(
                            "procnetdev_{}_bytes{{label=\"{}\"}} {}\n",
                            IFACE_FIELD_MAP[idx],
                            &iface_name[..(iface_name.len() - 1)],
                            item
                        ));
                    }
                } else {
                    panic!("Unexpected line {} with iface", line);
                }
            }
        }
    }
    result
}
