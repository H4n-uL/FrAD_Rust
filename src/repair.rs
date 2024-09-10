/**                                  Repair                                   */
/**
 * Copyright 2024 HaמuL
 * Function: Repair or Apply ECC to FrAD stream
 */

use crate::{
    backend::{SplitFront, VecPatternFind},
    common:: {crc16_ansi, crc32, FRM_SIGN, PIPEIN, PIPEOUT},
    fourier::profiles::{COMPACT, LOSSLESS},
    tools::  {asfh::ASFH, cli::CliParams, ecc, log::LogObj}
};
use std::{fs::File, io::{Read, Write}, path::Path, process::exit};

use same_file::is_same_file;

/** Repair
* Struct for FrAD Repairer
*/
pub struct Repair {
    asfh: ASFH,
    buffer: Vec<u8>,
    log: LogObj,

    fix_error: bool,
    olap_len: usize,
    ecc_ratio: [u8; 2],
}

impl Repair {
    pub fn new(mut ecc_ratio: [u8; 2]) -> Repair {
        if ecc_ratio[0] == 0 {
            eprintln!("ECC data size must not be zero");
            eprintln!("Setting ECC to default 96 24");
            ecc_ratio = [96, 24];
        }
        if ecc_ratio[0] as i16 + ecc_ratio[1] as i16 > 255 {
            eprintln!("ECC data size and check size must not exceed 255, given: {} and {}",
                ecc_ratio[0], ecc_ratio[1]);
            eprintln!("Setting ECC to default 96 24");
            ecc_ratio = [96, 24];
        }

        Repair {
            asfh: ASFH::new(),
            buffer: Vec::new(),
            log: LogObj::new(0, 0.5),

            fix_error: true,
            olap_len: 0,
            ecc_ratio,
        }
    }

    pub fn is_empty(&self) -> bool {
        return self.buffer.len() < FRM_SIGN.len();
    }

    /** process
     * Process the input stream and repair the FrAD stream
    * Parameters: Input stream
    * Returns: Repaired FrAD stream
    */
    pub fn process(&mut self, stream: Vec<u8>) -> Vec<u8> {
        self.buffer.extend(stream);
        let mut ret = Vec::new();

        loop {
            // If every parameter in the ASFH struct is set,
            /* 1. Repairing FrAD Frame */
            if self.asfh.all_set {
                // 1.0. If the buffer is not enough to decode the frame, break
                if self.buffer.len() < self.asfh.frmbytes as usize { break; }

                let samples = if self.asfh.olap == 0 || LOSSLESS.contains(&self.asfh.profile) { self.asfh.fsize as usize } else {
                    (self.asfh.fsize as usize * (self.asfh.olap as usize - 1)) / self.asfh.olap as usize };
                self.olap_len = self.asfh.fsize as usize - samples;

                // 1.1. Split out the frame data
                let mut frad: Vec<u8> = self.buffer.split_front(self.asfh.frmbytes as usize);

                // 1.2. Correct the error if ECC is enabled
                if self.asfh.ecc {
                    if self.fix_error && ( // and if the user requested
                        // and if CRC mismatch
                        LOSSLESS.contains(&self.asfh.profile) && crc32(&frad) != self.asfh.crc32 ||
                        COMPACT.contains(&self.asfh.profile) && crc16_ansi(&frad) != self.asfh.crc16
                    ) { frad = ecc::decode_rs(frad, self.asfh.ecc_ratio[0] as usize, self.asfh.ecc_ratio[1] as usize); } // Error correction
                    else { frad = ecc::unecc(frad, self.asfh.ecc_ratio[0] as usize, self.asfh.ecc_ratio[1] as usize); } // ECC removal
                }

                // 1.3. Create Reed-Solomon error correction code
                frad = ecc::encode_rs(frad, self.ecc_ratio[0] as usize, self.ecc_ratio[1] as usize);
                (self.asfh.ecc, self.asfh.ecc_ratio) = (true, self.ecc_ratio);

                // 1.4. Write the frame data to the buffer
                ret.extend(self.asfh.write(frad));
                self.log.update(&self.asfh.total_bytes, samples, &self.asfh.srate);
                self.log.logging(false);

                // 1.5. Clear the ASFH struct
                self.asfh.clear();
            }

            /* 2. Finding header / Gathering more data to parse */
            else {
                // 2.1. If the header buffer not found, find the header buffer
                if !self.asfh.buffer.starts_with(&FRM_SIGN) {
                    match self.buffer.find_pattern(&FRM_SIGN) {
                        // If pattern found in the buffer
                        // 2.1.1. Split out the buffer to the header buffer
                        Some(i) => {
                            ret.extend(self.buffer.split_front(i));
                            self.asfh.buffer = self.buffer.split_front(FRM_SIGN.len());
                        },
                        // 2.1.2. else, Split out the buffer to the last 4 bytes and return
                        None => {
                            ret.extend(self.buffer.split_front(self.buffer.len().saturating_sub(FRM_SIGN.len() - 1)));
                            break; 
                        }
                    }
                }
                // 2.2. If header buffer found, try parsing the header
                let force_flush = self.asfh.read(&mut self.buffer);

                // 2.3. Check header parsing result
                match force_flush {
                    // 2.3.1. If header is complete and not forced to flush, continue
                    Ok(false) => {},
                    // 2.3.2. If header is complete and forced to flush, flush and return
                    Ok(true) => {
                        self.log.update(&0, self.olap_len, &self.asfh.srate);
                        ret.extend(self.asfh.force_flush());
                        self.olap_len = 0;
                        break;
                    },
                    // 2.3.3. If header is incomplete, return
                    Err(_) => break,
                }
            }
        }
        return ret;
    }

    /** flush
     * Flush the remaining buffer
    * Parameters: None
    * Returns: Repairer buffer
    */
    pub fn flush(&mut self) -> Vec<u8> {
        let ret = self.buffer.clone();
        self.buffer.clear();
        return ret;
    }
}

/** repair
 * Repair or Apply ECC to FrAD stream
 * Parameters: Input file, CLI parameters
 * Returns: Repaired FrAD stream on File
 */
pub fn repair(rfile: String, params: CliParams, loglevel: u8) {
    let mut wfile = params.output;
    if rfile.is_empty() { panic!("Input file must be given"); }

    let mut rpipe = false;
    if PIPEIN.contains(&rfile.as_str()) { rpipe = true; }
    else if !Path::new(&rfile).exists() { panic!("Input file does not exist"); }

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
    repairer.log = LogObj::new(loglevel, 0.5);
    loop {
        let mut buffer = vec![0; 32768];
        let bytes_read = readfile.read(&mut buffer).unwrap();
        if bytes_read == 0 && repairer.is_empty() { break; }

        let mut repaired = repairer.process(buffer[..bytes_read].to_vec());
        writefile.write_all(&mut repaired).unwrap();
    }
    writefile.write_all(&mut repairer.flush()).unwrap();
    repairer.log.logging(true);
}