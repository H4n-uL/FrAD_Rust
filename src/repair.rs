/**                                  Repair                                   */
/**
 * Copyright 2024 HaמuL
 * Function: Repair or Apply ECC to FrAD stream
 */

use crate::{common, fourier::profiles::{COMPACT, LOSSLESS}, tools::{asfh::ASFH, cli, ecc, log::LogObj}};
use std::{fs::File, io::{Read, Write}, path::Path, process::exit};
use same_file::is_same_file;

/** repair
 * Repair or Apply ECC to FrAD stream
 * Parameters: Input file, CLI parameters
 * Returns: Repaired FrAD stream on File
 */
pub fn repair(rfile: String, params: cli::CliParams, loglevel: u8) {
    let mut wfile = params.output;
    let ecc_ratio = params.ecc_ratio;
    if rfile.is_empty() { panic!("Input file must be given"); }

    let mut rpipe = false;
    if common::PIPEIN.contains(&rfile.as_str()) { rpipe = true; }
    else if !Path::new(&rfile).exists() { panic!("Input file does not exist"); }

    let mut wpipe = false;
    if common::PIPEOUT.contains(&wfile.as_str()) { wpipe = true; }
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

    let (mut asfh, mut head, mut olap_fragment_len) = (ASFH::new(), Vec::new(), 0);
    let mut log = LogObj::new(loglevel, 0.5);
    loop {
        // 1. Reading the header
        if head.is_empty() {
            let mut buf = vec![0u8; 4];
            let readlen = readfile.read(&mut buf).unwrap();
            if readlen == 0 { log.update(&0, olap_fragment_len, &asfh.srate); break; }
            head = buf.to_vec();
        }
        // all the way until hitting the header or EOF
        if head != common::FRM_SIGN {
            let mut buf = vec![0u8; 1];
            let readlen = readfile.read(&mut buf).unwrap();
            if readlen == 0 { log.update(&0, olap_fragment_len, &asfh.srate); writefile.write_all(&head).unwrap(); break; }
            head.extend(buf);
            writefile.write_all(&[head[0]]).unwrap();
            head = head[1..].to_vec();
            continue;
        }
        // 2. Reading the frame
        head = Vec::new();
        let force_flush = asfh.update(&mut readfile);
        if force_flush { asfh.force_flush(&mut writefile); continue; }

        let samples = if asfh.olap == 0 || LOSSLESS.contains(&asfh.profile) { asfh.fsize as usize } else {
        (asfh.fsize as usize * (asfh.olap as usize - 1)) / asfh.olap as usize };
        olap_fragment_len = asfh.fsize as usize - samples;

        // 3. Reading the frame data
        let mut frad = vec![0u8; asfh.frmbytes as usize];
        let _ = common::read_exact(&mut readfile, &mut frad);

        // 4. Repairing the frame
        if asfh.ecc {
            if LOSSLESS.contains(&asfh.profile) && common::crc32(&frad) != asfh.crc32 ||
            COMPACT.contains(&asfh.profile) && common::crc16_ansi(&frad) != asfh.crc16
            { frad = ecc::decode_rs(frad, asfh.ecc_ratio[0] as usize, asfh.ecc_ratio[1] as usize); }
            else { frad = ecc::unecc(frad, asfh.ecc_ratio[0] as usize, asfh.ecc_ratio[1] as usize); }
        }

        // 5. Applying ECC
        frad = ecc::encode_rs(frad, ecc_ratio[0] as usize, ecc_ratio[1] as usize);

        // 6. Writing to file
        (asfh.ecc, asfh.ecc_ratio) = (true, ecc_ratio);
        asfh.write(&mut writefile, frad);

        log.update(&asfh.total_bytes, samples, &asfh.srate);
        log.logging(false);
    }
    log.logging(true);
}