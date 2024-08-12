/**                                  Logging                                  */
/**
 * Copyright 2024 HaמuL
 * Function: Logging tools
 */

use std::{collections::HashMap, time::{Duration, Instant}};

/** format_time
 * Formats time in seconds to human-readable format
 * Parameters: Time in seconds
 * Returns: Formatted time string
 */
fn format_time(n: f64) -> String {
    if n < 0.0 { return format!("-{}", format_time(n)); }
    let julian = (n / 31557600.0) as u8;
    let days = ((n % 31557600.0 / 86400.0) % 365.25) as u16;
    let hours = ((n % 86400.0 / 3600.0) % 24.0) as u8;
    let minutes = ((n % 3600.0 / 60.0) % 60.0) as u8;
    let seconds = (n % 60.0) as f64;

    if julian > 0 { return format!("J{}.{:03}:{:02}:{:02}:{:06.3}", julian, days, hours, minutes, seconds); }
    else if days > 0 { return format!("{}:{:02}:{:02}:{:06.3}", days, hours, minutes, seconds); }
    else if hours > 0 { return format!("{}:{:02}:{:06.3}", hours, minutes, seconds); }
    else if minutes > 0 { return format!("{}:{:06.3}", minutes, seconds); }
    else if seconds >= 1.0 { return format!("{:.3} s", seconds); }
    else if seconds >= 0.001 { return format!("{:.3} ms", seconds * 1000.0); }
    else if seconds >= 0.000001 { return format!("{:.3} µs", seconds * 1000000.0); }
    else if seconds > 0.0 { return format!("{:.3} ns", seconds * 1000000.0); }
    else { return "0".to_string(); }
}

/** format_bytes
 * Formats bytes count to human-readable format
 * Parameters: Bytes count
 * Returns: Formatted bytes count string
 */
fn format_bytes(n: f64) -> String {
    if n < 1000.0 { return format!("{}", n); }
    let exp = (n as f64).log10().floor() as u8 / 3;
    let unit = ["", "k", "M", "G", "T", "P", "E", "Z", "Y"];
    format!("{:.3} {}", n as f64 / 1000.0f64.powi(exp as i32), unit[exp as usize])
}

/** LogObj
 * Struct containing logging data
 */
pub struct LogObj {
    level: u8,
    start_time: Instant,
    duration: HashMap<u32, u128>,
    total_size: u128,
    bitrate: HashMap<u32, u128>,
    last_logging: Instant,
    log_interval: Duration,
}

impl LogObj {
    pub fn new(level: u8, log_intv: f64) -> LogObj {
        let log_intv = Duration::from_secs_f64(log_intv);
        LogObj {
            level,
            start_time: Instant::now(),
            duration: HashMap::new(),
            total_size: 0,
            bitrate: HashMap::new(),
            log_interval: log_intv,
            last_logging: Instant::now(),
        }
    }
    pub fn update(&mut self, size: usize, samples: usize, srate: u32) {
        self.total_size += size as u128;
        self.duration.insert(srate, if self.duration.contains_key(&srate) { self.duration[&srate] } else { 0 } + samples as u128);
        self.bitrate.insert(srate, if self.bitrate.contains_key(&srate) { self.bitrate[&srate] } else { 0 } + size as u128);
    }
    pub fn logging(&mut self, force: bool) {
        if !force && self.last_logging.elapsed() < self.log_interval { return; }
        self.last_logging = Instant::now();
        let total_duration: f64 = self.duration.iter().map(|(k, v)| *v as f64 / *k as f64).sum();
        let total_bits: f64 = self.bitrate.values().sum::<u128>() as f64 * 8.0;
        let bitrate = if total_duration > 0.0 { total_bits / total_duration } else { 0.0 };
        let encoding_time = self.start_time.elapsed().as_secs_f64();
        let speed = if encoding_time > 0.0 { total_duration / encoding_time } else { 0.0 };

        let mut x = String::new();

        if self.level == 1 {
            x = format!("size={}B time={} bitrate={}bits/s speed={:.1}x    \r",
                format_bytes(self.total_size as f64),
                format_time(total_duration),
                format_bytes(bitrate), speed
            );
        }
        if force { eprintln!("{}", x); } else { eprint!("{}", x); }
    }
}