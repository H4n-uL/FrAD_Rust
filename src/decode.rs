/**                                  Decode                                   */
/**
 * Copyright 2024 HaמuL
 * Function: Decode any file containing FrAD frames to PCM
 */

use crate::{fourier, fourier::profiles::profile1, 
    common, tools::{asfh::ASFH, cli, ecc}};

use std::{fs::File, io::{Write, ErrorKind}, path::Path};

/** overlap
 * Overlaps the current frame with the previous fragment
 * Parameters: Current frame, Previous frame fragment, ASFH
 * Returns: Overlapped frame, Updated fragment
 */
fn overlap(mut frame: Vec<Vec<f64>>, mut prev: Vec<Vec<f64>>, asfh: &ASFH) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    if prev.len() != 0 {
        let fade_in: Vec<f64> = prev.iter().enumerate().map(|(i, _)| i as f64 / prev.len() as f64).collect();
        let fade_out: Vec<f64> = prev.iter().enumerate().map(|(i, _)| 1.0 - i as f64 / prev.len() as f64).collect();
        for c in 0..asfh.channels as usize {
            for i in 0..prev.len() {
                frame[i][c] = frame[i][c] * fade_in[i] + prev[i][c] * fade_out[i];
            }
        }
    }
    if asfh.profile == 1 && asfh.olap != 0 {
        let olap = if asfh.olap > 2 { asfh.olap } else { 2 };
        prev = frame.split_off(frame.len() - frame.len() / olap as usize);
    }
    else { prev = Vec::new(); }

    (frame, prev)
}

/** flush
 * Flushes the PCM data to the output
 * Parameters: Output file, PCM data, Pipe toggle
 * Returns: None
 */
fn flush(mut file: &File, pcm: Vec<Vec<f64>>, pipe: bool) {
    let pcm_flat: Vec<f64> = pcm.into_iter().flatten().collect();
    let pcm_bytes: Vec<u8> = pcm_flat.iter().map(|x| x.to_be_bytes()).flatten().collect();
    if pipe { std::io::stdout().lock().write_all(&pcm_bytes)
        .unwrap_or_else(|err| 
            if err.kind() == ErrorKind::BrokenPipe { std::process::exit(0); } else { panic!("Error writing to stdout: {}", err); }
        ); 
    }
    else { file.write_all(&pcm_bytes).unwrap(); }
}

/** decode
 * Decodes any found FrAD frames in the input file to f64be PCM
 * Parameters: Input file, CLI parameters
 * Returns: Decoded PCM on File or stdout
 */
pub fn decode(rfile: String, params: cli::CliParams) {
    let mut wfile = params.output;
    let fix_error = params.enable_ecc;
    if rfile.len() == 0 { panic!("Input file must be given"); }

    let mut rpipe = false;
    if common::PIPEIN.contains(&rfile.as_str()) { rpipe = true; }
    else if !Path::new(&rfile).exists() { panic!("Input file does not exist"); }

    let mut wpipe = false;
    if common::PIPEOUT.contains(&wfile.as_str()) { wpipe = true; }
    else {
        if rfile == wfile { panic!("Input and output files cannot be the same"); }
        if wfile.len() == 0 {
            let wfrf = Path::new(&rfile).file_name().unwrap().to_str().unwrap().to_string();
            let wfile_prefix = wfrf.split(".").collect::<Vec<&str>>()[..wfrf.split(".").count() - 1].join(".");
            wfile = format!("{}.pcm", wfile_prefix);
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

    let mut readfile = if !rpipe { File::open(rfile).unwrap() } else { File::open(common::DEVNULL).unwrap() };
    let writefile = if !wpipe { File::create(wfile).unwrap() } else { File::create(common::DEVNULL).unwrap() };
    let mut asfh = ASFH::new();

    let mut head = Vec::new();
    let mut prev = Vec::new();
    loop { // Main decode loop
        if head.len() == 0 {
            let mut buf = vec![0u8; 4];
            let readlen = common::read_exact(&mut readfile, &mut buf, rpipe);
            if readlen == 0 { flush(&writefile, prev, wpipe); break; }
            head = buf.to_vec();
        }
        if head != common::FRM_SIGN {
            let mut buf = vec![0u8; 1];
            let readlen = common::read_exact(&mut readfile, &mut buf, rpipe);
            if readlen == 0 { flush(&writefile, prev, wpipe); break; }
            head.extend(buf);
            head = head[1..].to_vec();
            continue;
        }
        asfh.update(&mut readfile, rpipe);

        let mut frad = vec![0u8; asfh.frmbytes as usize];
        let _ = common::read_exact(&mut readfile, &mut frad, rpipe);

        if asfh.ecc {
            if fix_error && (
                asfh.profile == 0 && common::crc32(&frad) != asfh.crc32 ||
                asfh.profile == 1 && common::crc16_ansi(&frad) != asfh.crc16
            ) { frad = ecc::decode_rs(frad, asfh.ecc_rate[0] as usize, asfh.ecc_rate[1] as usize); }
            else { frad = ecc::unecc(frad, asfh.ecc_rate[0] as usize, asfh.ecc_rate[1] as usize); }
        }

        let mut pcm =
        if asfh.profile == 1 { profile1::digital(frad, asfh.bit_depth, asfh.channels, asfh.srate) }
        else { fourier::digital(frad, asfh.bit_depth, asfh.channels, asfh.endian) };

        (pcm, prev) = overlap(pcm, prev, &asfh);
        flush(&writefile, pcm, wpipe);
        head = Vec::new();
    }
}