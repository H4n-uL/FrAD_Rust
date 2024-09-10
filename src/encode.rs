/**                                  Encode                                   */
/**
 * Copyright 2024 HaמuL
 * Function: Encode f64be PCM to FrAD
 */

use crate::{
    backend::{Prepend, SplitFront},
    common:: {any_to_f64, read_exact, PCMFormat, PIPEIN, PIPEOUT},
    fourier::{profiles::{compact, profile0, profile1, profile4, COMPACT, LOSSLESS}, BIT_DEPTHS, SEGMAX},
    tools::  {asfh::ASFH, cli::CliParams, ecc, head, log::LogObj}
};
use std::{fs::File, io::{ErrorKind, IsTerminal, Read, Write}, path::Path, process::exit};

// use rand::{seq::{IteratorRandom, SliceRandom}, Rng};
use same_file::is_same_file;

/** Encoder
 * Struct for FrAD encoder
 */
pub struct Encoder {
    asfh: ASFH,
    bit_depth: i16, channels: i16, fsize: u32,
    buffer: Vec<u8>,
    overlap_fragment: Vec<Vec<f64>>,
    log: LogObj,

    pcm_format: PCMFormat,
    loss_level: u8,
}

impl Encoder {
    pub fn new(profile: u8, pcm_format: PCMFormat) -> Encoder {
        let mut asfh = ASFH::new();
        asfh.profile = profile;
        Encoder {
            asfh,
            bit_depth: 0, channels: 0, fsize: 0,
            buffer: Vec::new(),
            overlap_fragment: Vec::new(),
            log: LogObj::new(0, 0.5),

            pcm_format,
            loss_level: 0,
        }
    }

    // true dynamic info - set every frame
    pub fn set_channels(&mut self, channels: i16) {
        if channels == 0 { panic!("Channel count cannot be zero"); }
        self.channels = channels;
    }
    pub fn set_frame_size(&mut self, frame_size: u32) {
        if frame_size > SEGMAX[self.asfh.profile as usize] { panic!("Samples per frame cannot exceed {}", SEGMAX[self.asfh.profile as usize]); }
        self.fsize = frame_size;
    }

    // half-dynamic info - should be converted to bit depth index
    pub fn set_bit_depth(&mut self, bit_depth: i16) { self.bit_depth = bit_depth; }

    // static info - set once before encoding
    pub fn set_srate(&mut self, srate: u32) {
        if srate == 0 { panic!("Sample rate cannot be zero"); }
        self.asfh.srate = srate;
    }
    pub fn set_ecc(&mut self, ecc: bool, ecc_ratio: [u8; 2]) {
        self.asfh.ecc = ecc;
        if ecc_ratio[0] == 0 {
            eprintln!("ECC data size must not be zero");
            eprintln!("Setting ECC to default 96 24");
            self.asfh.ecc_ratio = [96, 24];
        }
        if ecc_ratio[0] as i16 + ecc_ratio[1] as i16 > 255 {
            eprintln!("ECC data size and check size must not exceed 255, given: {} and {}",
                ecc_ratio[0], ecc_ratio[1]);
            eprintln!("Setting ECC to default 96 24");
            self.asfh.ecc_ratio = [96, 24];
        }
        self.asfh.ecc_ratio = ecc_ratio;
    }
    pub fn set_little_endian(&mut self, little_endian: bool) { self.asfh.endian = little_endian; }
    // pub fn set_profile(&mut self, profile: u8) { self.asfh.profile = profile; }
    pub fn set_loss_level(&mut self, loss_level: u8) { self.loss_level = loss_level; }
    pub fn set_overlap(&mut self, mut overlap: u16) { if overlap != 0 { overlap = overlap.max(2).min(256); } self.asfh.olap = overlap; }


    /** overlap
     * Overlaps the current frame with the overlap fragment
     * Parameters: Current frame, Overlap fragment, Overlap rate, Profile
     * Returns: Overlapped frame, Next overlap fragment
     */
    fn overlap(&mut self, mut frame: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
        // 1. If overlap fragment is not empty,
        if !self.overlap_fragment.is_empty() {
            // prepent the fragment to the frame
            frame.prepend(&self.overlap_fragment);
        }
    
        // 2. If overlap is enabled and profile uses overlap
        let mut next_overlap = Vec::new();
        if COMPACT.contains(&self.asfh.profile) && self.asfh.olap > 1 {
            // Copy the last olap samples to the next overlap fragment
            let cutoff = (frame.len() * (self.asfh.olap as usize - 1)) / self.asfh.olap as usize;
            next_overlap = frame[cutoff..].to_vec();
        }
        self.overlap_fragment = next_overlap;
        return frame;
    }

    /** inner
     * Inner encoder loop
     * Parameters: PCM stream, Flush flag
     * Returns: Encoded audio data
     */
    fn inner(&mut self, stream: Vec<u8>, flush: bool) -> Vec<u8> {
        self.buffer.extend(stream);
        let mut ret:Vec<u8> = Vec::new();
        // let rng = &mut rand::thread_rng();

        loop {
            // self.asfh.profile = *vec![0, 1, 4].choose(rng).unwrap();
            // self.bit_depth = *BIT_DEPTHS[self.asfh.profile as usize].iter().filter(|&&x| x != 0).choose(rng).unwrap();
            // self.set_frame_size(*compact::SAMPLES_LI.choose(rng).unwrap());
            // self.set_loss_level(rng.gen_range(0..21));
            // let ecc_dsize = rng.gen_range(1..254);
            // let ecc_codesize = rng.gen_range(1..255 - ecc_dsize);
            // self.set_ecc(rng.gen_bool(0.5), [ecc_dsize, ecc_codesize]);
            // self.set_overlap(rng.gen_range(2..256));

            // 0. Set read length in samples
            let mut rlen = self.fsize as usize;
            if COMPACT.contains(&self.asfh.profile) {
                // Read length = smallest value in SMPLS_LI bigger than frame size and overlap fragment size
                let li_val = *compact::SAMPLES_LI.iter().filter(|&x| *x >= self.fsize as u32).min().unwrap() as usize;
                if li_val < self.overlap_fragment.len() // if overlap fragment is bigger than frame size
                { // find the smallest value in SMPLS_LI bigger than fragment and subtract fragment size
                    rlen = *compact::SAMPLES_LI.iter().filter(|&x| *x >= self.overlap_fragment.len() as u32).min().unwrap() as usize - self.overlap_fragment.len();
                }
                else { // else, just subtract fragment size
                    rlen = li_val - self.overlap_fragment.len();
                };
            }
            let bytes_per_sample = self.pcm_format.bit_depth() / 8;
            let read_bytes = rlen * self.channels as usize * bytes_per_sample;
            if self.buffer.len() < read_bytes && !flush { break; }

            // 1. Cut out the frame from the buffer
            let pcm_bytes: Vec<u8> = self.buffer.split_front(read_bytes);
            let pcm_flat: Vec<f64> = pcm_bytes.chunks(bytes_per_sample).map(|bytes| any_to_f64(bytes, &self.pcm_format)).collect();

            // Unravel flat PCM to 2D PCM array
            let mut frame: Vec<Vec<f64>> = (0..rlen).take_while(|&i| (i as usize + 1) * self.channels as usize <= pcm_flat.len())
            .map(|i| pcm_flat[i as usize * (self.channels as usize)..(i + 1) as usize * (self.channels as usize)].to_vec()).collect();
            if frame.is_empty() { self.asfh.force_flush(); break; } // If frame is empty, break
            let samples = frame.len();

            // 2. Overlap the frame with the previous overlap fragment
            frame = self.overlap(frame);
            let fsize: u32 = frame.len() as u32;

            // 3. Encode the frame
            if !BIT_DEPTHS[self.asfh.profile as usize].contains(&self.bit_depth) { panic!("Invalid bit depth"); }
            let (mut frad, bit_ind, chnl) = match self.asfh.profile {
                1 => profile1::analogue(frame, self.bit_depth, self.asfh.srate, self.loss_level),
                4 => profile4::analogue(frame, self.bit_depth, self.asfh.endian),
                _ => profile0::analogue(frame, self.bit_depth, self.asfh.endian)
            };

            // 4. Create Reed-Solomon error correction code
            if self.asfh.ecc {
                frad = ecc::encode_rs(frad, self.asfh.ecc_ratio[0] as usize, self.asfh.ecc_ratio[1] as usize);
            }

            // 5. Write the frame to the buffer
            (self.asfh.bit_depth, self.asfh.channels, self.asfh.fsize) = (bit_ind, chnl, fsize);
            ret.extend(self.asfh.write(frad));

            // Logging
            self.log.update(&self.asfh.total_bytes, samples, &self.asfh.srate);
            self.log.logging(false);
        }

        return ret;
    }

    pub fn process(&mut self, stream: Vec<u8>) -> Vec<u8> {
        return self.inner(stream, false);
    }

    pub fn flush(&mut self) -> Vec<u8> {
        return self.inner(Vec::new(), true);
    }
}

/** set_files
 * Sets input and output files
 * Parameters: Input file, Output file, Profile, Overwrite flag
 * Returns: Input file reader, Output file writer
 */
pub fn set_files(input: String, mut output: String, profile: u8, overwrite: bool) -> (Box<dyn Read>, Box<dyn Write>) {
    let (mut rpipe, mut wpipe) = (false, false);
    if input.is_empty() { eprintln!("Input file must be given"); exit(1); }
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
    let mut encoder = Encoder::new(params.profile, params.pcm);
    encoder.log = LogObj::new(loglevel, 0.5);

    encoder.set_ecc(params.enable_ecc, params.ecc_ratio);
    encoder.set_little_endian(params.little_endian);
    encoder.set_bit_depth(params.bits);

    encoder.set_srate(params.srate);
    encoder.set_channels(params.channels as i16);
    encoder.set_frame_size(params.frame_size);
    encoder.set_overlap(params.overlap);

    encoder.set_loss_level(params.losslevel);

    let (mut rfile, mut wfile) = set_files(input, params.output, encoder.asfh.profile, params.overwrite);

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

    loop {
        let mut pcm_buf = vec![0u8; 32768];
        let readlen = read_exact(&mut rfile, &mut pcm_buf);
        if readlen == 0 { break; }
        wfile.write_all(&encoder.process(pcm_buf[..readlen].to_vec())).unwrap();
    }
    wfile.write_all(&encoder.flush()).unwrap();
    encoder.log.logging(true);
}