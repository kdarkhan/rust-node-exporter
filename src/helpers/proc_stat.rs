use regex::Regex;
use std::fs::File;
use std::io;
use std::io::prelude::*;

pub fn get_proc_stat() -> String {
    lazy_static! {
        static ref CPU_PATTERN: Regex = Regex::new(
            r"^cpu(?P<cpu>\d*)?\s+(?P<user>\d+)\s+(?P<nice>\d+)\s+(?P<system>\d+)\s+(?P<idle>\d+)\s+(?P<iowait>\d+)\s+(?P<irq>\d+)\s+(?P<softirq>\d+)\s+(?P<steal>\d+)\s+(?P<guest>\d+)\s+(?P<guestnice>\d+)\s*$"
        )
        .unwrap();
        static ref CTXT_PATTERN: Regex = Regex::new(
            r"^ctxt\s+(\d+)\s*$"
        )
        .unwrap();
        static ref BTIME_PATTERN: Regex = Regex::new(
            r"^btime\s+(\d+)\s*$"
        )
        .unwrap();
        static ref PROCESSES_PATTERN: Regex = Regex::new(
            r"^processes\s+(\d+)\s*$"
        )
        .unwrap();
        static ref PROCS_RUNNING_PATTERN: Regex = Regex::new(
            r"^processes\s+(\d+)\s*$"
        )
        .unwrap();
        static ref PROCS_BLOCKED_PATTERN: Regex = Regex::new(
            r"^processes\s+(\d+)\s*$"
        )
        .unwrap();
        static ref SOFTIRQ_PATTERN: Regex = Regex::new(
            r"^softirq\s+(\d+)\s[\d.]+$"
        )
        .unwrap();
    }

    let file = File::open("/proc/stat").expect("cannot open file");
    let lines = io::BufReader::new(file).lines();

    let mut result = String::new();

    for line in lines {
        if let Ok(line) = line {
            if let Some(m) = CPU_PATTERN.captures(&line) {
                result.push_str(&format!(
                    "cpu{}_user_hz {}\n",
                    m["cpu"].to_string(),
                    m["user"].to_string()
                ));
                result.push_str(&format!(
                    "cpu{}_nice_hz {}\n",
                    m["cpu"].to_string(),
                    m["nice"].to_string()
                ));
                result.push_str(&format!(
                    "cpu{}_system_hz {}\n",
                    m["cpu"].to_string(),
                    m["system"].to_string()
                ));
                result.push_str(&format!(
                    "cpu{}_idle_hz {}\n",
                    m["cpu"].to_string(),
                    m["idle"].to_string()
                ));
                result.push_str(&format!(
                    "cpu{}_iowait_hz {}\n",
                    m["cpu"].to_string(),
                    m["iowait"].to_string()
                ));
                result.push_str(&format!(
                    "cpu{}_irq_hz {}\n",
                    m["cpu"].to_string(),
                    m["irq"].to_string()
                ));
                result.push_str(&format!(
                    "cpu{}_softirq_hz {}\n",
                    m["cpu"].to_string(),
                    m["softirq"].to_string()
                ));
                result.push_str(&format!(
                    "cpu{}_steal_hz {}\n",
                    m["cpu"].to_string(),
                    m["steal"].to_string()
                ));
                result.push_str(&format!(
                    "cpu{}_guest_hz {}\n",
                    m["cpu"].to_string(),
                    m["guest"].to_string()
                ));
                result.push_str(&format!(
                    "cpu{}_guestnice_hz {}\n",
                    m["cpu"].to_string(),
                    m["guestnice"].to_string()
                ));
            } else if let Some(m) = CTXT_PATTERN.captures(&line) {
                result.push_str(&format!("procstat_ctxt {}\n", m[1].to_string()));
            } else if let Some(m) = BTIME_PATTERN.captures(&line) {
                result.push_str(&format!("procstat_btime_sec {}\n", m[1].to_string()));
            } else if let Some(m) = PROCESSES_PATTERN.captures(&line) {
                result.push_str(&format!("procstat_processes {}\n", m[1].to_string()));
            } else if let Some(m) = PROCS_RUNNING_PATTERN.captures(&line) {
                result.push_str(&format!("procstat_procs_running {}\n", m[1].to_string()));
            } else if let Some(m) = PROCS_BLOCKED_PATTERN.captures(&line) {
                result.push_str(&format!("procstat_procs_blocked {}\n", m[1].to_string()));
            }
        }
    }
    result
}
