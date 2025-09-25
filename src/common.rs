//!                         Common application tools                         !//
//!
//! Copyright 2024-2025 HaמuL
//! Description: Common tools for FrAD Executable

use std::{fs::File, io::{ErrorKind, IsTerminal, Read, Write}, path::Path, process::exit};

// Pipe and null device
pub const PIPEIN: &[&str] = &["-", "/dev/stdin", "/dev/fd/0"];
pub const PIPEOUT: &[&str] = &["-", "/dev/stdout", "/dev/fd/1"];

/// read_exact
/// Reads a file or stdin to a buffer with exact size
/// Parameters: File(&mut), Buffer(&mut)
/// Returns: Total bytes read
pub fn read_exact(file: &mut Box<dyn Read>, buf: &mut [u8]) -> usize {
    let mut total_read = 0;

    while total_read < buf.len() {
        let read_size = file.read(&mut buf[total_read..]).unwrap();
        if read_size == 0 { break; }
        total_read += read_size;
    }
    return total_read;
}

/// write_safe
/// Writes data to stdout with broken pipe handling
/// Parameters: Output file writer, Data buffer
pub fn write_safe(wfile: &mut Box<dyn Write>, buf: &[u8]) {
    wfile.write_all(buf).unwrap_or_else(|err| {
        match err.kind() {
            ErrorKind::BrokenPipe => { exit(0); },
            _ => { panic!("Error writing to stdout: {}", err); }
        }
    });
}

/// get_file_stem
/// Gets the file stem from a file path
/// Parameters: File path
/// Returns: File stem
pub fn get_file_stem(file_path: &str) -> String {
    if PIPEIN.contains(&file_path) || PIPEOUT.contains(&file_path) { return "pipe".to_string(); }
    return match Path::new(file_path).file_stem() {
        Some(stem) => stem.to_str().unwrap().to_string(),
        None => file_path.to_string()
    }
}

/// format_time
/// Formats time in seconds to human-readable format
/// Parameters: Time in seconds
/// Returns: Formatted time string
pub fn format_time(mut n: f64) -> String {
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

const UNITS: [&str; 11] = ["", "k", "M", "G", "T", "P", "E", "Z", "Y", "R", "Q"];

/// format_si
/// Formats a number to SI prefixed format
/// Parameters: Number
/// Returns: Formatted number string
pub fn format_si(n: f64) -> String {
    let exp = (n.abs().log10() / 3.0).floor().clamp(0.0, UNITS.len() as f64 - 1.0);
    format!("{:.3} {}", n as f64 / 1000.0f64.powi(exp as i32), UNITS[exp as usize])
}

/// format_speed
/// Formats speed in x to short and easy-to-read format
/// Parameters: Speed in x
/// Returns: Formatted speed string
pub fn format_speed(n: f64) -> String {
    if n >= 100.0 { format!("{:.0}", n) }
    else if n >= 10.0 { format!("{:.1}", n) }
    else if n >= 1.0 { format!("{:.2}", n) }
    else { format!("{:.3}", n) }
}

/// move_all
/// Moves all data from readfile to writefile with given buffer size
/// Parameters: Input file reader, Output file writer, Buffer size
pub fn move_all(readfile: &mut File, writefile: &mut File, bufsize: usize) {
    loop {
        let mut buf: Vec<u8> = vec![0; bufsize];
        let mut total_read = 0;

        while total_read < buf.len() {
            let read_size = readfile.read(&mut buf[total_read..]).unwrap();
            if read_size == 0 { break; }
            total_read += read_size;
        }
        if total_read == 0 { break; }
        writefile.write_all(&buf[..total_read]).unwrap();
    }
}

/// check_overwrite
/// Checks if the output file exists and asks for overwrite
/// Parameters: Output file, Overwrite flag
pub fn check_overwrite(writefile: &str, overwrite: bool) {
    if Path::new(writefile).exists() && !overwrite {
        if std::io::stdin().is_terminal() {
            eprintln!("Output file already exists, overwrite? (Y/N)");
            loop {
                eprint!("> ");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                if input.trim().to_lowercase() == "y" { break; }
                else if input.trim().to_lowercase() == "n" { eprintln!("Aborted."); exit(0); }
            }
        }
        else { eprintln!("Output file already exists, please provide --force(-y) flag to overwrite."); exit(0); }
    }
}
