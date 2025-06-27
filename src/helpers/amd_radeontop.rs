use regex::Captures;
use regex::Regex;
use serde::Deserialize;
use std::io::prelude::*;
use std::io::BufReader;
use std::process::{Command, Stdio};
use std::sync::LazyLock;
use std::sync::Mutex;
use std::thread;

use std::time::SystemTime;

const RADEONTOP_PRELUDE: &str = "Dumping to -, until termination.";
const MAX_RESULT_LIFE: u64 = 20;

static CURRENT_STATS: LazyLock<Mutex<Option<Stats>>> = LazyLock::new(|| Mutex::new(None));
static CURRENT_CLK_STATS: LazyLock<Mutex<Option<ClkStats>>> = LazyLock::new(|| Mutex::new(None));
static LAST_UPDATE: LazyLock<Mutex<u64>> = LazyLock::new(|| Mutex::new(0));
static RADEONTOP_LINE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    // Example line:
    // 1732491561.725899: bus 09, gpu 100.00%, ee 0.00%, vgt 100.00%, ta 100.00%, sx 100.00%, sh 100.00%, spi 100.00%, sc 100.00%, pa 0.00%, db 100.00%, cb 100.00%, vram 8.29% 1351.82mb, gtt 6.52% 517.27mb, mclk 9.60% 0.096ghz, sclk 2.97% 0.079ghz
    Regex::new(
        r"^[\d.]+: bus \w+, gpu ([\d.]+)%, ee ([\d.]+)%, vgt ([\d.]+)%, ta ([\d.]+)%, sx ([\d.]+)%, sh ([\d.]+)%, spi ([\d.]+)%, sc ([\d.]+)%, pa ([\d.]+)%, db ([\d.]+)%, cb ([\d.]+)%, vram ([\d.]+)% ([\d.]+)mb, gtt ([\d.]+)% ([\d.]+)mb",
    ).unwrap()
});
static RADEONTOP_CLK_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"mclk ([\d.]+)% ([\d.]+)ghz, sclk ([\d.]+)% ([\d.]+)ghz$").unwrap()
});

fn get_sys_time_in_secs() -> u64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

pub fn init() {
    println!("Spawning radeontop process");
    let mut child = Command::new("radeontop")
        .arg("-d")
        .arg("-")
        .arg("-i")
        .arg("15")
        .arg("-t")
        .arg("1")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start radeontop command");

    let child_id = child.id();
    println!("Child id is {child_id}");

    let status_code = child
        .try_wait()
        .unwrap_or_else(|_| panic!("Could not get status of child process {child_id}"));
    if status_code.is_some() {
        panic!("Expected the child {child_id} to not finish but it looks like it finished");
    }

    let stdout = child
        .stdout
        .unwrap_or_else(|| panic!("Could not get stdout of {child_id}"));

    thread::spawn(move || {
        let reader = BufReader::new(stdout);

        reader.lines().map_while(Result::ok).for_each(|line| {
            println!("{line}");
            if !line.eq(RADEONTOP_PRELUDE) {
                if let Some(m) = RADEONTOP_LINE_PATTERN.captures(&line) {
                    let mut current_stats = CURRENT_STATS.lock().unwrap();
                    let mut last_update = LAST_UPDATE.lock().unwrap();
                    *current_stats = Some(parse_stdout_line(m));
                    *last_update = get_sys_time_in_secs();

                    // mclk and sclk results are sometimes not returned. Missing them,
                    // is not problematic.
                    if let Some(m) = RADEONTOP_CLK_PATTERN.captures(&line) {
                        let mut current_clk_stats = CURRENT_CLK_STATS.lock().unwrap();
                        *current_clk_stats = Some(parse_clk_captures(m));
                    }
                } else {
                    // TODO: make sure to panic the whole process and not only this thread
                    panic!("Could not parse {line}")
                }
            }
        });
    });
}

fn parse_stdout_line(captures: Captures) -> Stats {
    Stats {
        gpu: to_f64(&captures[1]),
        ee: to_f64(&captures[2]),
        vgt: to_f64(&captures[3]),
        ta: to_f64(&captures[4]),
        sx: to_f64(&captures[5]),
        sh: to_f64(&captures[6]),
        spi: to_f64(&captures[7]),
        sc: to_f64(&captures[8]),
        pa: to_f64(&captures[9]),
        db: to_f64(&captures[10]),
        cb: to_f64(&captures[11]),
        vram_percent: to_f64(&captures[12]),
        vram: to_f64(&captures[13]),
        gtt_percent: to_f64(&captures[14]),
        gtt: to_f64(&captures[15]),
    }
}

fn parse_clk_captures(captures: Captures) -> ClkStats {
    ClkStats {
        mclk_percent: to_f64(&captures[1]),
        mclk: to_f64(&captures[2]),
        sclk_percent: to_f64(&captures[3]),
        sclk: to_f64(&captures[4]),
    }
}

fn to_f64(input: &str) -> f64 {
    input.parse::<f64>().unwrap()
}

#[derive(Debug, Deserialize, PartialEq)]
struct Stats {
    gpu: f64,
    ee: f64,
    vgt: f64,
    ta: f64,
    sx: f64,
    sh: f64,
    spi: f64,
    sc: f64,
    pa: f64,
    db: f64,
    cb: f64,
    vram_percent: f64,
    gtt_percent: f64,
    vram: f64,
    gtt: f64,
}

#[derive(Debug, Deserialize, PartialEq)]
struct ClkStats {
    mclk_percent: f64,
    sclk_percent: f64,
    mclk: f64,
    sclk: f64,
}

pub fn get_radeontop_stats() -> String {
    let current_stats = CURRENT_STATS.lock().unwrap();
    let last_update = LAST_UPDATE.lock().unwrap();
    if current_stats.is_none() {
        return "".to_string();
    }
    let current_clk_stats = CURRENT_CLK_STATS.lock().unwrap();

    // If the result is too outdated
    let current_time = get_sys_time_in_secs();
    if current_time - *last_update > MAX_RESULT_LIFE {
        return "".to_string();
    }

    let stats = current_stats.as_ref().unwrap();

    let mut result = String::new();
    result.push_str(&format!("amdgpu_radeontop_gpu {}\n", stats.gpu));
    result.push_str(&format!("amdgpu_radeontop_ee {}\n", stats.ee));
    result.push_str(&format!("amdgpu_radeontop_vgt {}\n", stats.vgt));
    result.push_str(&format!("amdgpu_radeontop_ta {}\n", stats.ta));
    result.push_str(&format!("amdgpu_radeontop_sx {}\n", stats.sx));
    result.push_str(&format!("amdgpu_radeontop_sh {}\n", stats.sh));
    result.push_str(&format!("amdgpu_radeontop_spi {}\n", stats.spi));
    result.push_str(&format!("amdgpu_radeontop_sc {}\n", stats.sc));
    result.push_str(&format!("amdgpu_radeontop_pa {}\n", stats.pa));
    result.push_str(&format!("amdgpu_radeontop_db {}\n", stats.db));
    result.push_str(&format!("amdgpu_radeontop_cb {}\n", stats.cb));
    result.push_str(&format!(
        "amdgpu_radeontop_vram_percent {}\n",
        stats.vram_percent
    ));
    result.push_str(&format!("amdgpu_radeontop_vram {}\n", stats.vram));
    result.push_str(&format!(
        "amdgpu_radeontop_gtt_percent {}\n",
        stats.gtt_percent
    ));
    result.push_str(&format!("amdgpu_radeontop_gtt {}\n", stats.gtt));

    if current_clk_stats.is_none() {
        return result;
    }

    // Add clk stats
    let clk_stats = current_clk_stats.as_ref().unwrap();
    result.push_str(&format!(
        "amdgpu_radeontop_mclk_percent {}\n",
        clk_stats.mclk_percent
    ));
    result.push_str(&format!("amdgpu_radeontop_mclk {}\n", clk_stats.mclk));
    result.push_str(&format!(
        "amdgpu_radeontop_sclk_percent {}\n",
        clk_stats.sclk_percent
    ));
    result.push_str(&format!("amdgpu_radeontop_sclk {}\n", clk_stats.sclk));
    result
}
