//!                            Repair application                            !//
//!
//! Copyright 2024-2025 HaמuL
//! Description: Repairer implementation example

use libfrad::Repairer;
use crate::{
    common::{check_overwrite, format_si, get_file_stem, read_exact, write_safe, PIPEIN, PIPEOUT},
    tools::{cli::CliParams, process::ProcessInfo}
};
use std::{fs::File, io::{Read, Write}, path::Path, process::exit};

use same_file::is_same_file;

/// logging_repair
/// Logs a message to stderr
/// Parameters: Log level, Processing info, line feed flag
pub fn logging_repair(loglevel: u8, log: &ProcessInfo, linefeed: bool) {
    if loglevel == 0 { return; }
    let total_size = log.get_total_size() as f64;
    eprint!("size={}B speed={}B/s    \r",
        format_si(total_size),
        format_si(total_size / log.start_time.elapsed().as_secs_f64())
    );
    if linefeed { eprintln!(); }
}

/// repair
/// Repair or Apply ECC to FrAD stream
/// Parameters: Input file, CLI parameters
/// Returns: Repaired FrAD stream on File
pub fn repair(rfile: String, params: CliParams) {
    let mut wfile = params.output;
    if rfile.is_empty() { eprintln!("Input file must be given"); exit(1); }

    let mut rpipe = false;
    if PIPEIN.contains(&rfile.as_str()) { rpipe = true; }
    else if !Path::new(&rfile).exists() { eprintln!("Input file does not exist"); exit(1); }

    let mut wpipe = false;
    if PIPEOUT.contains(&wfile.as_str()) { wpipe = true; }
    else if let Ok(true) = is_same_file(&rfile, &wfile) {
        eprintln!("Input and output files cannot be the same"); exit(1);
    }

    if wfile.is_empty() {
        let ext = Path::new(&rfile).extension().unwrap();
        wfile = if !rpipe { format!("{}.repaired.{}", get_file_stem(&rfile), ext.to_str().unwrap()) } else { "repaired.frad".to_string() };
    }

    check_overwrite(&wfile, params.overwrite);

    let mut readfile: Box<dyn Read> = if !rpipe { Box::new(File::open(&rfile).unwrap()) } else { Box::new(std::io::stdin()) };
    let mut writefile: Box<dyn Write> = if !wpipe { Box::new(File::create(&wfile).unwrap()) } else { Box::new(std::io::stdout()) };

    let mut repairer = Repairer::new(params.ecc_ratio);
    let mut procinfo = ProcessInfo::new();
    loop {
        let mut buffer = vec![0; 32768];
        let bytes_read = read_exact(&mut readfile, &mut buffer);
        if bytes_read == 0 && repairer.is_empty() { break; }

        let repaired = repairer.process(&buffer[..bytes_read]);
        procinfo.update(repaired.len(), 0, 0);
        write_safe(&mut writefile, &repaired);
        logging_repair(params.loglevel, &procinfo, false);
    }
    let repaired = repairer.flush();
    procinfo.update(repaired.len(), 0, 0);
    write_safe(&mut writefile, &repaired);
    logging_repair(params.loglevel, &procinfo, true);

    if params.overwrite_repair && !(rpipe || wpipe) {
        std::fs::rename(wfile, rfile).unwrap();
    }
}