/**                            Encode application                             */
/**
 * Copyright 2024 HaמuL
 * Description: Encoder implementation example
 */

use frad::{Encoder, profiles::LOSSLESS, head, StreamInfo};
use crate::{
    common::{check_overwrite, logging, read_exact, PIPEIN, PIPEOUT},
    tools::cli::CliParams
};
use std::{fs::File, io::{ErrorKind, Read, Write}, path::Path, process::exit};
use same_file::is_same_file;

/** set_files
 * Sets input and output files
 * Parameters: Input file, Output file, Profile, Overwrite flag
 * Returns: Input file reader, Output file writer
 */
pub fn set_files(rfile: String, mut wfile: String, profile: u8, overwrite: bool) -> (Box<dyn Read>, Box<dyn Write>) {
    let (mut rpipe, mut wpipe) = (false, false);
    if PIPEIN.contains(&rfile.as_str()) { rpipe = true; }
    else if !Path::new(&rfile).exists() { eprintln!("Input file doesn't exist"); exit(1); }
    if PIPEOUT.contains(&wfile.as_str()) { wpipe = true; }
    else {
        match is_same_file(&rfile, &wfile) {
            Ok(true) => { eprintln!("Input and wfile files cannot be the same"); exit(1); }
            _ => {}
        }
    }

    if wfile.is_empty() {
        let wfrf = Path::new(&rfile).file_name().unwrap().to_str().unwrap().to_string();
        wfile = wfrf.split(".").collect::<Vec<&str>>()[..wfrf.split(".").count() - 1].join(".");
    }
    if !(wfile.ends_with(".frad") || wfile.ends_with(".dsin")
        || wfile.ends_with(".fra") || wfile.ends_with(".dsn")) {
        if LOSSLESS.contains(&profile) {
            if wfile.len() <= 8 { wfile = format!("{}.fra", wfile); }
            else { wfile = format!("{}.frad", wfile); }
        }
        else if wfile.len() <= 8 { wfile = format!("{}.dsn", wfile); }
        else { wfile = format!("{}.dsin", wfile); }
    }

    check_overwrite(&wfile, overwrite);

    let readfile: Box<dyn Read> = if !rpipe { Box::new(File::open(rfile).unwrap()) } else { Box::new(std::io::stdin()) };
    let writefile: Box<dyn Write> = if !wpipe { Box::new(File::create(wfile).unwrap()) } else { Box::new(std::io::stdout()) };

    return (readfile, writefile);
}

/** encode
 * Encodes PCM to FrAD
 * Parameters: Input file, CLI parameters, Log level
 */
pub fn encode(input: String, params: CliParams, loglevel: u8) {
    if input.is_empty() { eprintln!("Input file must be given"); exit(1); }

    let mut encoder = Encoder::new(params.profile, params.pcm);
    if params.srate == 0 { eprintln!("Sample rate should be set except zero"); exit(1); }
    if params.channels == 0 { eprintln!("Channel count should be set except zero"); exit(1); }

    encoder.set_srate(params.srate);
    encoder.set_channels(params.channels as i16);

    encoder.set_frame_size(params.frame_size);

    encoder.set_ecc(params.enable_ecc, params.ecc_ratio);
    encoder.set_little_endian(params.little_endian);
    encoder.set_bit_depth(params.bits);
    encoder.set_overlap_ratio(params.overlap_ratio);

    let loss_level = 1.25_f64.powi(params.losslevel as i32) / 19.0 + 0.5;
    encoder.set_loss_level(loss_level);

    let (mut rfile, mut wfile) = set_files(input, params.output, params.profile, params.overwrite);

    let mut image = Vec::new();
    if !params.image_path.is_empty() {
        match File::open(&params.image_path) {
            Ok(mut imgfile) => { imgfile.read_to_end(&mut image).unwrap(); },
            Err(_) => { eprintln!("Image not found"); }
        }
    }

    wfile.write_all(&head::builder(&params.meta, image)).unwrap_or_else(
        |err| { eprintln!("Error writing to stdout: {}", err);
        if err.kind() == ErrorKind::BrokenPipe { exit(0); } else { panic!("Error writing to stdout: {}", err); } }
    );

    encoder.streaminfo = StreamInfo::new();
    loop {
        let mut pcm_buf = vec![0u8; 32768];
        let readlen = read_exact(&mut rfile, &mut pcm_buf);
        if readlen == 0 { break; }
        wfile.write_all(&encoder.process(pcm_buf[..readlen].to_vec())).unwrap();
        logging(loglevel, &encoder.streaminfo, false);
    }
    wfile.write_all(&encoder.flush()).unwrap();
    logging(loglevel, &encoder.streaminfo, true);
}