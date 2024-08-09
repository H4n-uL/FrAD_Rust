/**                                  Encode                                   */
/**
 * Copyright 2024 HaמuL
 * Function: Encode f64be PCM to FrAD
 */

use crate::{common, fourier::{self, profiles::{profile1, profile4}}, tools::{asfh::ASFH, cli, ecc, head}};
use std::{fs::File, io::{ErrorKind, IsTerminal, Read, Write}, path::Path};

/** overlap
 * Overlaps the current frame with the previous fragment
 * Parameters: Current frame, Previous frame fragment, Overlap rate, Profile
 * Returns: Overlapped frame, Updated fragment
 */
fn overlap(mut data: Vec<Vec<f64>>, mut prev: Vec<Vec<f64>>, olap: u8, profile: u8) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let fsize = data.len() + prev.len();
    let olap = if olap > 0 { olap.max(2) } else { 0 };

    if !prev.is_empty() {
        let mut ndata = Vec::new();
        ndata.extend(prev.iter().cloned());
        ndata.extend(data.iter().cloned());
        data = ndata;
    }

    if profile == 1 || profile == 2 && olap > 0 {
        let cutoff = data.len() - (fsize / olap as usize);
        prev = data[cutoff..].to_vec();
    }
    else { prev = Vec::new(); }
    return (data, prev);
}

/** encode
 * Encodes f64be PCM to FrAD frames
 * Parameters: Input file, CLI parameters
 * Returns: Encoded FrAD frames on File or stdout
 */
pub fn encode(rfile: String, params: cli::CliParams) {
    let wfile = params.output;
    let bit_depth = params.bits;
    let channels = params.channels;
    let srate = params.srate;

    let buffersize = params.frame_size;
    let little_endian = params.little_endian;

    let enable_ecc = params.enable_ecc;
    let ecc_rate = params.ecc_rate;

    let profile = params.profile;
    let olap = params.overlap;
    let losslevel = params.losslevel;

    let (mut rpipe, mut wpipe) = (false, false);
    if rfile.is_empty() { panic!("Input file must be given"); }
    if common::PIPEIN.contains(&rfile.as_str()) { rpipe = true; }
    else if !Path::new(&rfile).exists() { panic!("Input file does not exist"); }
    if common::PIPEOUT.contains(&wfile.as_str()) { wpipe = true; }
    else if rfile == wfile { panic!("Input and output files cannot be the same"); }

    if srate == 0 { panic!("Sample rate must be given"); }
    if channels == 0 { panic!("Number of channels must be given"); }

    if !fourier::DEPTHS.contains(&bit_depth)
    && !profile1::DEPTHS.contains(&bit_depth)
    { panic!("Invalid bit depth"); }

    let segmax =
        if profile == 1 { *profile1::SMPLS_LI.iter().max().unwrap() }
        else { u32::MAX };
    if buffersize > segmax { panic!("Samples per frame cannot exceed {}", segmax); }

    // Making sure the output file is set
    let mut wfile = wfile;
    if wfile.is_empty() {
        let wfrf = Path::new(&rfile).file_name().unwrap().to_str().unwrap().to_string();
        wfile = wfrf.split(".").collect::<Vec<&str>>()[..wfrf.split(".").count() - 1].join(".");
    }
    if !(wfile.ends_with(".frad") || wfile.ends_with(".dsin")
        || wfile.ends_with(".fra") || wfile.ends_with(".dsn")) {
        if [0, 4].contains(&profile) {
            if wfile.len() <= 8 { wfile = format!("{}.fra", wfile); }
            else { wfile = format!("{}.frad", wfile); }
        }
        else if wfile.len() <= 8 { wfile = format!("{}.dsn", wfile); }
        else { wfile = format!("{}.dsin", wfile); }
    }

    if Path::new(&wfile).exists() && !params.overwrite {
        if std::io::stdin().is_terminal() {
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
        else {
            eprintln!("Output file already exists, please provide -y flag to overwrite.");
            eprintln!("Aborted.");
            std::process::exit(0);
        }
    }

    let mut readfile: Box<dyn Read> = if !rpipe { Box::new(File::open(rfile).unwrap()) } else { Box::new(std::io::stdin()) };
    let mut writefile: Box<dyn Write> = if !wpipe { Box::new(File::create(wfile).unwrap()) } else { Box::new(std::io::stdout()) };

    let mut asfh = ASFH::new();
    let mut prev: Vec<Vec<f64>> = Vec::new();

    let mut img = Vec::new();
    if !params.image_path.is_empty() {
        match File::open(&params.image_path) {
            Ok(mut imgfile) => { imgfile.read_to_end(&mut img).unwrap(); },
            Err(_) => { eprintln!("Image not found"); }
        }
    }
    let header = head::builder(&params.meta, img);
    writefile.write_all(&header).unwrap_or_else(
        |err| { eprintln!("Error writing to stdout: {}", err);
        if err.kind() == ErrorKind::BrokenPipe { std::process::exit(0); } else { panic!("Error writing to stdout: {}", err); } }
    );
    let pcm_format = params.pcm;

    loop { // Main encode loop
        let mut rlen = buffersize as usize;

        if profile == 1 {
            let li_val = *profile1::SMPLS_LI.iter().find(|&&x| x >= buffersize).unwrap() as usize;
            rlen = if li_val < prev.len() 
            { *profile1::SMPLS_LI.iter().find(|&&x| x >= prev.len() as u32).unwrap() as usize - prev.len() }  else { li_val }
        }
        let fbytes = rlen * channels as usize * pcm_format.bit_depth() / 8;
        let mut pcm_buf = vec![0u8; fbytes];
        let readlen = common::read_exact(&mut readfile, &mut pcm_buf);
        if readlen == 0 { break; }

        let pcm: Vec<f64> = pcm_buf[..readlen].chunks(pcm_format.bit_depth() / 8)
        .map(|bytes: &[u8]| common::any_to_f64(bytes, &pcm_format)).collect();

        let mut frame: Vec<Vec<f64>> = (0..buffersize)
        .take_while(|&i| (i as usize + 1) * channels as usize <= pcm.len())
        .map(|i| pcm[i as usize * (channels as usize)..(i + 1) as usize * (channels as usize)].to_vec())
        .collect();

        // Overlapping for Profile 1
        (frame, prev) = overlap(frame, prev, olap, profile);
        let fsize: u32 = frame.len() as u32;

        // Encoding
        let (mut frad, bit_ind, chnl) =
        if profile == 1 { profile1::analogue(frame, bit_depth, srate, losslevel) }
        else if profile == 4 { profile4::analogue(frame, bit_depth, little_endian) }
        else { fourier::analogue(frame, bit_depth, little_endian) };

        if enable_ecc { // Creating Reed-Solomon error correction code
            frad = ecc::encode_rs(frad, ecc_rate[0] as usize, ecc_rate[1] as usize);
        }

        // Writing to file
        (asfh.bit_depth, asfh.channels, asfh.endian, asfh.profile) = (bit_ind, chnl, little_endian, profile);
        (asfh.srate, asfh.fsize, asfh.olap) = (srate, fsize, olap);
        (asfh.ecc, asfh.ecc_rate) = (enable_ecc, ecc_rate);
        // i rly wish i dont need to do this

        let frad: Vec<u8> = asfh.write_vec(frad);

        writefile.write_all(&frad).unwrap_or_else(|err|
            { eprintln!("Error writing to stdout: {}", err);
            if err.kind() == ErrorKind::BrokenPipe { std::process::exit(0); } else { panic!("Error writing to stdout: {}", err); } }
        );
    }
}