/**                            Encode application                             */
/**
 * Copyright 2024 HaמuL
 * Description: Encoder implementation example
 */

use frad::{Encode, fourier::profiles::LOSSLESS, tools::{head, stream::StreamInfo}};
use crate::{
    common::{logging, read_exact, PIPEIN, PIPEOUT},
    tools::cli::CliParams
};
use std::{fs::File, io::{ErrorKind, IsTerminal, Read, Write}, path::Path, process::exit};

// use rand::{seq::{IteratorRandom, SliceRandom}, Rng};
use same_file::is_same_file;

/** set_files
 * Sets input and output files
 * Parameters: Input file, Output file, Profile, Overwrite flag
 * Returns: Input file reader, Output file writer
 */
pub fn set_files(input: String, mut output: String, profile: u8, overwrite: bool) -> (Box<dyn Read>, Box<dyn Write>) {
    let (mut rpipe, mut wpipe) = (false, false);
    if PIPEIN.contains(&input.as_str()) { rpipe = true; }
    else if !Path::new(&input).exists() { eprintln!("Input file doesn't exist"); exit(1); }
    if PIPEOUT.contains(&output.as_str()) { wpipe = true; }
    else {
        match is_same_file(&input, &output) {
            Ok(true) => { eprintln!("Input and output files cannot be the same"); exit(1); }
            _ => {}
        }
    }

    if output.is_empty() {
        let wfrf = Path::new(&input).file_name().unwrap().to_str().unwrap().to_string();
        output = wfrf.split(".").collect::<Vec<&str>>()[..wfrf.split(".").count() - 1].join(".");
    }
    if !(output.ends_with(".frad") || output.ends_with(".dsin")
        || output.ends_with(".fra") || output.ends_with(".dsn")) {
        if LOSSLESS.contains(&profile) {
            if output.len() <= 8 { output = format!("{}.fra", output); }
            else { output = format!("{}.frad", output); }
        }
        else if output.len() <= 8 { output = format!("{}.dsn", output); }
        else { output = format!("{}.dsin", output); }
    }

    if Path::new(&output).exists() && !overwrite {
        if std::io::stdin().is_terminal() {
            eprintln!("Output file already exists, overwrite? (Y/N)");
            loop {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                if input.trim().to_lowercase() == "y" { break; }
                else if input.trim().to_lowercase() == "n" { eprintln!("Aborted."); exit(0); }
            }
        }
        else { eprintln!("Output file already exists, please provide -y flag to overwrite."); exit(0); }
    }

    let readfile: Box<dyn Read> = if !rpipe { Box::new(File::open(input).unwrap()) } else { Box::new(std::io::stdin()) };
    let writefile: Box<dyn Write> = if !wpipe { Box::new(File::create(&output).unwrap()) } else { Box::new(std::io::stdout()) };

    return (readfile, writefile);
}

/** encode
 * Encodes PCM to FrAD
 * Parameters: Input file, CLI parameters, Log level
 */
pub fn encode(input: String, params: CliParams, loglevel: u8) {
    if input.is_empty() { eprintln!("Input file must be given"); exit(1); }

    let mut encoder = Encode::new(params.profile, params.pcm);
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

    let header = head::builder(&params.meta, image);
    wfile.write_all(&header).unwrap_or_else(
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