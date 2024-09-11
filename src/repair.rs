/**                                Repair app                                 */
/**
 * Copyright 2024 HaמuL
 * Description: Repairer implementation example
 */

use frad::Repair;
use crate::{
    common::{logging, PIPEIN, PIPEOUT},
    tools::cli::CliParams
};
use std::{fs::File, io::{Read, Write}, path::Path, process::exit};

use same_file::is_same_file;

/** repair
 * Repair or Apply ECC to FrAD stream
 * Parameters: Input file, CLI parameters
 * Returns: Repaired FrAD stream on File
 */
pub fn repair(rfile: String, params: CliParams, loglevel: u8) {
    let mut wfile = params.output;
    if rfile.is_empty() { eprintln!("Input file must be given"); exit(1); }

    let mut rpipe = false;
    if PIPEIN.contains(&rfile.as_str()) { rpipe = true; }
    else if !Path::new(&rfile).exists() { eprintln!("Input file does not exist"); exit(1); }

    let mut wpipe = false;
    if PIPEOUT.contains(&wfile.as_str()) { wpipe = true; }
    else {
        match is_same_file(&rfile, &wfile) {
            Ok(true) => { eprintln!("Input and output files cannot be the same"); exit(1); }
            _ => {}
        }
        if wfile.is_empty() {
            let wfrf = Path::new(&rfile).file_name().unwrap().to_string_lossy().split(".").map(|s| s.to_string()).collect::<Vec<String>>();
            wfile = [wfrf[..wfrf.len() - 1].join("."), "recov".to_string(), wfrf[wfrf.len() - 1].clone()].join(".");
        }

        if Path::new(&wfile).exists() && !params.overwrite {
            eprintln!("Output file already exists, overwrite? (Y/N)");
            loop {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                if input.trim().to_lowercase() == "y" { break; }
                else if input.trim().to_lowercase() == "n" {
                    eprintln!("Aborted.");
                    std::process::exit(0);
                }
            }
        }
    }

    let mut readfile: Box<dyn Read> = if !rpipe { Box::new(File::open(rfile).unwrap()) } else { Box::new(std::io::stdin()) };
    let mut writefile: Box<dyn Write> = if !wpipe { Box::new(File::create(wfile).unwrap()) } else { Box::new(std::io::stdout()) };

    let mut repairer = Repair::new(params.ecc_ratio);
    loop {
        let mut buffer = vec![0; 32768];
        let bytes_read = readfile.read(&mut buffer).unwrap();
        if bytes_read == 0 && repairer.is_empty() { break; }

        let mut repaired = repairer.process(buffer[..bytes_read].to_vec());
        writefile.write_all(&mut repaired).unwrap();
        logging(loglevel, &repairer.streaminfo, false);
    }
    writefile.write_all(&mut repairer.flush()).unwrap();
    logging(loglevel, &repairer.streaminfo, true);
}