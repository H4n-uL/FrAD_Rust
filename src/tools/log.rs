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
fn format_time(mut n: f64) -> String {
    if n < 0.0 { return format!("-{}", format_time(-n)); }
    
    let julian = (n / 31557600.0) as u16; n = n % 31557600.0;
    let days = (n / 86400.0) as u16; n = n % 86400.0;
    let hours = (n / 3600.0) as u8; n = n % 3600.0;
    let minutes = (n / 60.0) as u8; n = n % 60.0;

    return {
        if julian > 0 { format!("J{}.{:03}:{:02}:{:02}:{:06.3}", julian, days, hours, minutes, n) }
        else if days > 0 { format!("{}:{:02}:{:02}:{:06.3}", days, hours, minutes, n) }
        else if hours > 0 { format!("{}:{:02}:{:06.3}", hours, minutes, n) }
        else if minutes > 0 { format!("{}:{:06.3}", minutes, n) }
        else if n >= 1.0 { format!("{:.3} s", n) }
        else if n >= 0.001 { format!("{:.3} ms", n * 1000.0) }
        else if n >= 0.000001 { format!("{:.3} µs", n * 1000000.0) }
        else if n > 0.0 { format!("{:.3} ns", n * 1000000000.0) }
        else { "0".to_string() }
    };
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

/** format_speed
 * Formats speed in x to short and easy-to-read format
 * Parameters: Speed in x
 * Returns: Formatted speed string
 */
fn format_speed(n: f64) -> String {
    if n >= 100.0 { format!("{:.0}", n) }
    else if n >= 10.0 { format!("{:.1}", n) }
    else if n >= 1.0 { format!("{:.2}", n) }
    else { format!("{:.3}", n) }
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
    pub fn update(&mut self, size: &u128, samples: usize, srate: &u32) {
        self.total_size += size;
        self.duration.insert(*srate, if self.duration.contains_key(&srate) { self.duration[&srate] } else { 0 } + samples as u128);
        self.bitrate.insert(*srate, if self.bitrate.contains_key(&srate) { self.bitrate[&srate] } else { 0 } + *size as u128);
    }
    pub fn logging(&mut self, force: bool) {
        if !force && self.last_logging.elapsed() < self.log_interval { return; }
        if self.level == 0 { return; }
        self.last_logging = Instant::now();
        let total_duration: f64 = self.duration.iter().map(|(k, v)| *v as f64 / *k as f64).sum();
        let total_bits: f64 = self.bitrate.values().sum::<u128>() as f64 * 8.0;
        let bitrate = if total_duration > 0.0 { total_bits / total_duration } else { 0.0 };
        let encoding_time = self.start_time.elapsed().as_secs_f64();
        let speed = if encoding_time > 0.0 { total_duration / encoding_time } else { 0.0 };

        let mut x = String::new();

        if self.level == 1 {
            x = format!("size={}B time={} bitrate={}bits/s speed={}x",
                format_bytes(self.total_size as f64),
                format_time(total_duration),
                format_bytes(bitrate),
                format_speed(speed)
            );
        }
        if force { eprintln!("{}    \r", x); } else { eprint!("{}    \r", x); }
    }
}