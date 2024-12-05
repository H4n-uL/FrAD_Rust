/**                            Encode application                             */
/**
 * Copyright 2024 HaמuL
 * Description: Encoder implementation example
 */

use frad::{Encoder, profiles::LOSSLESS, head};
use crate::{
    common::{check_overwrite, format_si, format_speed, format_time, read_exact, write_safe, PIPEIN, PIPEOUT},
    tools::{cli::CliParams, process::ProcessInfo}
};
use std::{fs::File, io::{Read, Write}, path::Path, process::exit};
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
    else if let Ok(true) = is_same_file(&rfile, &wfile) {
        eprintln!("Input and output files cannot be the same"); exit(1);
    }

    if wfile.is_empty() {
        let wfrf = Path::new(&rfile).file_name().unwrap().to_str().unwrap().to_string();
        wfile = wfrf.split(".").collect::<Vec<&str>>()[..wfrf.split(".").count() - 1].join(".");
    }
    if !(wfile.ends_with(".frad") || wfile.ends_with(".dsin")
        || wfile.ends_with(".fra") || wfile.ends_with(".dsn")) {
        if LOSSLESS.contains(&profile) {
            if wfile.len() <= 8 && wfile.is_ascii() { wfile = format!("{}.fra", wfile); }
            else { wfile = format!("{}.frad", wfile); }
        }
        else if wfile.len() <= 8 && wfile.is_ascii() { wfile = format!("{}.dsn", wfile); }
        else { wfile = format!("{}.dsin", wfile); }
    }

    check_overwrite(&wfile, overwrite);

    let readfile: Box<dyn Read> = if !rpipe { Box::new(File::open(rfile).unwrap()) } else { Box::new(std::io::stdin()) };
    let writefile: Box<dyn Write> = if !wpipe { Box::new(File::create(wfile).unwrap()) } else { Box::new(std::io::stdout()) };

    return (readfile, writefile);
}

/** logging_encode
 * Logs a message to stderr
 * Parameters: Log level, Processing info, line feed flag
 */
pub fn logging_encode(loglevel: u8, log: &ProcessInfo, linefeed: bool) {
    if loglevel == 0 { return; }
    eprint!("size={}B time={} bitrate={}bits/s speed={}x    \r",
        format_si(log.get_total_size() as f64), format_time(log.get_duration()), format_si(log.get_bitrate()), format_speed(log.get_speed())
    );
    if linefeed { eprintln!(); }
}

/** encode
 * Encodes PCM to FrAD
 * Parameters: Input file, CLI parameters, Log level
 */
pub fn encode(input: String, params: CliParams) {
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

    let (mut readfile, mut writefile) = set_files(input, params.output, params.profile, params.overwrite);

    let mut image = Vec::new();
    if !params.image_path.is_empty() {
        match File::open(&params.image_path) {
            Ok(mut imgfile) => { imgfile.read_to_end(&mut image).unwrap(); },
            Err(_) => { eprintln!("Image not found"); }
        }
    }

    write_safe(&mut writefile, &head::builder(&params.meta, image, None));

    let mut procinfo = ProcessInfo::new();
    loop {
        let mut pcm_buf = vec![0u8; 32768];
        let readlen = read_exact(&mut readfile, &mut pcm_buf);
        if readlen == 0 { break; }

        let encoded = encoder.process(&pcm_buf[..readlen]);
        procinfo.update(encoded.buf.len(), encoded.samples, encoder.get_srate());
        write_safe(&mut writefile, &encoded.buf);
        logging_encode(params.loglevel, &procinfo, false);
    }
    let encoded = encoder.flush();
    procinfo.update(encoded.buf.len(), encoded.samples, encoder.get_srate());
    write_safe(&mut writefile, &encoded.buf);
    logging_encode(params.loglevel, &procinfo, true);
}