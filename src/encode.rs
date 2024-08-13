/**                                  Encode                                   */
/**
 * Copyright 2024 HaמuL
 * Function: Encode f64be PCM to FrAD
 */

use crate::{common, fourier::{profiles::{compact, profile0, profile1, profile4, COMPACT, LOSSLESS}, SEGMAX},
    tools::{asfh::ASFH, cli, ecc, head, log::LogObj}};
use std::{fs::File, io::{ErrorKind, IsTerminal, Read, Write}, path::Path, process::exit};

/** EncodeParameters
 * Struct containing all parameters for encoding
 */
pub struct EncodeParameters {
    rfilename: String, wfilename: String,
    rpipe: bool, wpipe: bool,
    srate: u32, channels: u8, bit_depth: i16,
    pcmfmt: common::PCMFormat,
    enable_ecc: bool, ecc_ratio: [u8; 2],
    frame_size: u32, little_endian: bool,
    profile: u8, loss_level: u8, overlap: u8,
    metadata: Vec<(String, Vec<u8>)>,
    image: Vec<u8>,
}

impl EncodeParameters {
    pub fn _new() -> EncodeParameters {
        EncodeParameters {
            rfilename: String::new(), wfilename: String::new(),
            rpipe: false, wpipe: false,
            srate: 48000, channels: 2, bit_depth: 0,
            pcmfmt: common::PCMFormat::F64(common::Endian::Big),
            enable_ecc: false, ecc_ratio: [96, 24],
            frame_size: 2048, little_endian: false,
            profile: 4, loss_level: 0, overlap: 16,
            metadata: Vec::new(),
            image: Vec::new(),
        }
    }

    pub fn from_cli(rfile: String, mut params: cli::CliParams) -> EncodeParameters {
        let mut wfile = params.output;
        let profile = params.profile;

        let (mut rpipe, mut wpipe) = (false, false);
        if rfile.is_empty() { eprintln!("Input file must be given"); exit(1); }
        if common::PIPEIN.contains(&rfile.as_str()) { rpipe = true; }
        else if !Path::new(&rfile).exists() { eprintln!("Input file doesn't exist"); exit(1); }
        if common::PIPEOUT.contains(&wfile.as_str()) { wpipe = true; }
        else if rfile == wfile { eprintln!("Input and output files cannot be the same"); exit(1); }

        if params.srate == 0 { eprintln!("Sample rate must be given"); exit(1); }
        if params.channels == 0 { eprintln!("Channel count must be given"); exit(1); }

        // Making sure the output file is set
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

        if Path::new(&wfile).exists() && !params.overwrite {
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

        let mut img = Vec::new();
        if !params.image_path.is_empty() {
            match File::open(&params.image_path) {
                Ok(mut imgfile) => { imgfile.read_to_end(&mut img).unwrap(); },
                Err(_) => { eprintln!("Image not found"); }
            }
        }

        if params.ecc_ratio[0] as i16 + params.ecc_ratio[1] as i16 > 255 {
            eprintln!("ECC data size and check size must not exceed 255, given: {} and {}",
                params.ecc_ratio[0], params.ecc_ratio[1]);
            eprintln!("Setting ECC to default 96 24");
            params.ecc_ratio = [96, 24];
        }

        EncodeParameters {
            rfilename: rfile, wfilename: wfile,
            rpipe: rpipe, wpipe: wpipe,
            srate: params.srate, channels: params.channels as u8, bit_depth: params.bits,
            pcmfmt: params.pcm, enable_ecc: params.enable_ecc, ecc_ratio: params.ecc_ratio,
            frame_size: params.frame_size, little_endian: params.little_endian,
            profile: params.profile, loss_level: params.losslevel, overlap: params.overlap,
            metadata: params.meta, image: img,
        }
    }
}

/** overlap
 * Overlaps the current frame with the overlap fragment
 * Parameters: Current frame, Overlap fragment, Overlap rate, Profile
 * Returns: Overlapped frame, Next overlap fragment
 */
fn overlap(mut frame: Vec<Vec<f64>>, overlap_fragment: Vec<Vec<f64>>, olap: u8, profile: u8) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let olap = if olap > 0 { olap.max(2) } else { 0 };

    if !overlap_fragment.is_empty() {
        let mut nframe = Vec::new();
        nframe.extend(overlap_fragment.iter().cloned());
        nframe.extend(frame.iter().cloned());
        frame = nframe;
    }

    let mut next_overlap = Vec::new();
    if COMPACT.contains(&profile) && olap > 0 {
        let cutoff = (frame.len() * (olap as usize - 1)) / olap as usize;
        next_overlap = frame[cutoff..].to_vec();
    }
    return (frame, next_overlap);
}

/** encode
 * Encodes f64be PCM to FrAD frames
 * Parameters: Input file, CLI parameters
 * Returns: Encoded FrAD frames on File or stdout
 */
pub fn encode(encparam: EncodeParameters, loglevel: u8) {
    let mut asfh = ASFH::new();
    let mut overlap_fragment: Vec<Vec<f64>> = Vec::new();

    let mut readfile: Box<dyn Read> = if !encparam.rpipe { Box::new(File::open(encparam.rfilename).unwrap()) } else { Box::new(std::io::stdin()) };
    let mut writefile: Box<dyn Write> = if !encparam.wpipe { Box::new(File::open(encparam.wfilename).unwrap()) } else { Box::new(std::io::stdout()) };

    let header = head::builder(&encparam.metadata, encparam.image);
    writefile.write_all(&header).unwrap_or_else(
        |err| { eprintln!("Error writing to stdout: {}", err);
        if err.kind() == ErrorKind::BrokenPipe { exit(0); } else { panic!("Error writing to stdout: {}", err); } }
    );

    let mut log = LogObj::new(loglevel, 0.5);

    loop { // Main encode loop
        // 1. Encoding parameter verification
        if encparam.srate == 0 { panic!("Sample rate cannot be zero"); }
        if encparam.channels == 0 { panic!("Channel count cannot be zero"); }
        if encparam.frame_size > SEGMAX[encparam.profile as usize] { panic!("Samples per frame cannot exceed {}", SEGMAX[encparam.profile as usize]); }

        // 2. Reading PCM data
        let mut rlen = encparam.frame_size as usize;
        if COMPACT.contains(&encparam.profile) {
            // Read length = smallest value in SMPLS_LI bigger than frame size and overlap fragment size
            let li_val = *compact::SAMPLES_LI.iter().filter(|&x| *x >= encparam.frame_size as u32).min().unwrap() as usize;
            rlen = if li_val < overlap_fragment.len()
            { *compact::SAMPLES_LI.iter().filter(|&x| *x >= overlap_fragment.len() as u32).min().unwrap() as usize - overlap_fragment.len() } else { li_val - overlap_fragment.len() };
        }
        let fbytes = rlen * encparam.channels as usize * encparam.pcmfmt.bit_depth() / 8;
        let mut pcm_buf = vec![0u8; fbytes];
        let readlen = common::read_exact(&mut readfile, &mut pcm_buf);
        if readlen == 0 { break; }

        // 3. RAW PCM bitstream to f64 PCM
        let pcm: Vec<f64> = pcm_buf[..readlen].chunks(encparam.pcmfmt.bit_depth() / 8)
        .map(|bytes: &[u8]| common::any_to_f64(bytes, &encparam.pcmfmt)).collect();

        let mut frame: Vec<Vec<f64>> = (0..encparam.frame_size)
        .take_while(|&i| (i as usize + 1) * encparam.channels as usize <= pcm.len())
        .map(|i| pcm[i as usize * (encparam.channels as usize)..(i + 1) as usize * (encparam.channels as usize)].to_vec())
        .collect();
        let samples = frame.len();

        // 3.5. Overlapping for Profile 1
        (frame, overlap_fragment) = overlap(frame, overlap_fragment, encparam.overlap, encparam.profile);
        let fsize: u32 = frame.len() as u32;

        // 4. Encoding
        if !(
            profile0::DEPTHS.contains(&encparam.bit_depth) ||
            profile1::DEPTHS.contains(&encparam.bit_depth) ||
            profile4::DEPTHS.contains(&encparam.bit_depth)
        ) { panic!("Invalid bit depth"); }

        let (mut frad, bit_ind, chnl) = match encparam.profile {
            1 => profile1::analogue(frame, encparam.bit_depth, encparam.srate, encparam.loss_level),
            4 => profile4::analogue(frame, encparam.bit_depth, encparam.little_endian),
            _ => profile0::analogue(frame, encparam.bit_depth, encparam.little_endian)
        };

        if encparam.enable_ecc { // Creating Reed-Solomon error correction code
            frad = ecc::encode_rs(frad, encparam.ecc_ratio[0] as usize, encparam.ecc_ratio[1] as usize);
        }

        // 5. Writing to file
        (asfh.bit_depth, asfh.channels, asfh.endian, asfh.profile) = (bit_ind, chnl, encparam.little_endian, encparam.profile);
        (asfh.srate, asfh.fsize, asfh.olap) = (encparam.srate, fsize, encparam.overlap);
        (asfh.ecc, asfh.ecc_ratio) = (encparam.enable_ecc, encparam.ecc_ratio);
        // i rly wish i dont need to do this

        let frad: Vec<u8> = asfh.write_vec(frad);
        writefile.write_all(&frad).unwrap_or_else(|err| {
            if err.kind() == ErrorKind::BrokenPipe { exit(0); }
            else { panic!("Error writing to stdout: {}", err); }
        });

        log.update(asfh.total_bytes, samples, asfh.srate);
        log.logging(false);
    }
    log.logging(true);
}