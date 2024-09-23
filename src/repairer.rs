/**                            Repair application                             */
/**
 * Copyright 2024 HaמuL
 * Description: Repairer implementation example
 */

use frad::Repairer;
use crate::{
    common::{check_overwrite, logging, PIPEIN, PIPEOUT},
    tools::cli::CliParams
};
use std::{fs::File, io::{Read, Write}, path::Path, process::exit};

use same_file::is_same_file;

/** repair
 * Repair or Apply ECC to FrAD stream
 * Parameters: Input file, CLI parameters
 * Returns: Repaired FrAD stream on File
 */
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
        let wfrf = Path::new(&rfile).file_name().unwrap().to_string_lossy().split(".").map(|s| s.to_string()).collect::<Vec<String>>();
        wfile = [wfrf[..wfrf.len() - 1].join("."), "recov".to_string(), wfrf[wfrf.len() - 1].clone()].join(".");
    }

    check_overwrite(&wfile, params.overwrite);

    let mut readfile: Box<dyn Read> = if !rpipe { Box::new(File::open(rfile).unwrap()) } else { Box::new(std::io::stdin()) };
    let mut writefile: Box<dyn Write> = if !wpipe { Box::new(File::create(wfile).unwrap()) } else { Box::new(std::io::stdout()) };

    let mut repairer = Repairer::new(params.ecc_ratio);
    loop {
        let mut buffer = vec![0; 32768];
        let bytes_read = readfile.read(&mut buffer).unwrap();
        if bytes_read == 0 && repairer.is_empty() { break; }

        let mut repaired = repairer.process(buffer[..bytes_read].to_vec());
        writefile.write_all(&mut repaired).unwrap();
        logging(params.loglevel, &repairer.streaminfo, false);
    }
    writefile.write_all(&mut repairer.flush()).unwrap();
    logging(params.loglevel, &repairer.streaminfo, true);
}