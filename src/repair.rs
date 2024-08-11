/**                                  Repair                                   */
/**
 * Copyright 2024 HaמuL
 * Function: Repair or Apply ECC to FrAD stream
 */

use crate::{common, tools::{asfh::ASFH, cli, ecc}};
use std::{fs::File, io::{Read, Write}, path::Path};

/** repair
 * Repair or Apply ECC to FrAD stream
 * Parameters: Input file, CLI parameters
 * Returns: Repaired FrAD stream on File
 * Note: Pipe is not supported
 */
pub fn repair(rfile: String, params: cli::CliParams) {
    let wfile = params.output;
    let ecc_ratio = params.ecc_ratio;
    if rfile.is_empty() { panic!("Input file must be given"); }
    if wfile.is_empty() { panic!("Output file must be given"); }
    if rfile == wfile { panic!("Input and output files cannot be the same"); }

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

    let mut readfile: Box<dyn Read> = Box::new(File::open(rfile).unwrap());
    let mut writefile: Box<dyn Write> = Box::new(File::create(wfile).unwrap());

    let mut asfh = ASFH::new();

    let mut head = Vec::new();
    loop {
        if head.is_empty() {
            let mut buf = vec![0u8; 4];
            let readlen = readfile.read(&mut buf).unwrap();
            if readlen == 0 { break; }
            head = buf.to_vec();
        }
        if head != common::FRM_SIGN {
            let mut buf = vec![0u8; 1];
            let readlen = readfile.read(&mut buf).unwrap();
            if readlen == 0 { break; }
            head.extend(buf);
            head = head[1..].to_vec();
            continue;
        }
        asfh.update(&mut readfile);

        let mut frad = vec![0u8; asfh.frmbytes as usize];
        let _ = common::read_exact(&mut readfile, &mut frad);

        if asfh.ecc {
            if [0, 4].contains(&asfh.profile) && common::crc32(&frad) != asfh.crc32 ||
                asfh.profile == 1 && common::crc16_ansi(&frad) != asfh.crc16
            { frad = ecc::decode_rs(frad, asfh.ecc_ratio[0] as usize, asfh.ecc_ratio[1] as usize); }
            else { frad = ecc::unecc(frad, asfh.ecc_ratio[0] as usize, asfh.ecc_ratio[1] as usize); }
        }

        frad = ecc::encode_rs(frad, ecc_ratio[0] as usize, ecc_ratio[1] as usize);

        // Writing to file
        (asfh.ecc, asfh.ecc_ratio) = (true, ecc_ratio);

        let frad: Vec<u8> = asfh.write_vec(frad);

        writefile.write_all(frad.as_slice()).unwrap();
        head = Vec::new();
    }
}