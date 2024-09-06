/**                                  Decode                                   */
/**
 * Copyright 2024 HaמuL
 * Function: Decode any file containing FrAD frames to PCM
 */

use crate::{backend::linspace, common::{self, f64_to_any, PCMFormat},
    fourier::{self, profiles::{profile0, profile1, profile4, COMPACT, LOSSLESS}},
    tools::{asfh::ASFH, cli, ecc, log::LogObj}};
use std::{fs::File, io::{ErrorKind, Read, Write}, path::Path};
use rodio::{buffer::SamplesBuffer, OutputStream, Sink};

/** overlap
 * Overlaps the current frame with the overlap fragment
 * Parameters: Current frame, Overlap fragment, ASFH
 * Returns: Overlapped frame, Next overlap fragment
 */
fn overlap(mut frame: Vec<Vec<f64>>, overlap_fragment: Vec<Vec<f64>>, asfh: &ASFH) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    if !overlap_fragment.is_empty() {
        let fade_in: Vec<f64> = linspace(0.0, 1.0, overlap_fragment.len());
        let fade_out: Vec<f64> = linspace(1.0, 0.0, overlap_fragment.len());
        for c in 0..asfh.channels as usize {
            for i in 0..overlap_fragment.len() {
                frame[i][c] = frame[i][c] * fade_in[i] + overlap_fragment[i][c] * fade_out[i];
            }
        }
    }
    let mut next_overlap = Vec::new();
    if COMPACT.contains(&asfh.profile) && asfh.olap != 0 {
        let olap = asfh.olap.max(2);
        next_overlap = frame.split_off((frame.len() * (olap as usize - 1)) / olap as usize);
    }
    return (frame, next_overlap);
}

/** flush
 * Flushes the PCM data to the output
 * Parameters: Play flag, Output file/sink, PCM data, PCM format, Sample rate
 * Parameters: Output file, PCM data
 * Returns: None
 */
fn flush(isplay: bool, file: &mut Box<dyn Write>, sink: &mut Sink, pcm: Vec<Vec<f64>>, fmt: &PCMFormat, srate: u32) {
    if pcm.is_empty() { return; }
    if isplay {
        sink.append(SamplesBuffer::new(
            pcm[0].len() as u16,
            srate,
            pcm.into_iter().flatten().map(|x| x as f32).collect::<Vec<f32>>()
        ));
    }
    else {
        let pcm_bytes: Vec<u8> = pcm.into_iter().flatten().flat_map(|x| f64_to_any(x, fmt)).collect();
        file.write_all(&pcm_bytes)
        .unwrap_or_else(|err|
            if err.kind() == ErrorKind::BrokenPipe { std::process::exit(0); } else { panic!("Error writing to stdout: {}", err); }
        );
    }
}

/** decode
 * Decodes any found FrAD frames in the input file to f64be PCM
 * Parameters: Input file, CLI parameters
 * Returns: Decoded PCM on File or stdout
 */
pub fn decode(rfile: String, params: cli::CliParams, mut loglevel: u8) {
    let mut wfile = params.output;
    let fix_error = params.enable_ecc;
    if rfile.is_empty() { panic!("Input file must be given"); }

    let mut rpipe = false;
    if common::PIPEIN.contains(&rfile.as_str()) { rpipe = true; }
    else if !Path::new(&rfile).exists() { panic!("Input file does not exist"); }

    let mut wpipe = false;
    if common::PIPEOUT.contains(&wfile.as_str()) { wpipe = true; }
    else {
        if rfile == wfile { panic!("Input and output files cannot be the same"); }
        if wfile.is_empty() {
            let wfrf = Path::new(&rfile).file_name().unwrap().to_str().unwrap().to_string();
            wfile = wfrf.split(".").collect::<Vec<&str>>()[..wfrf.split(".").count() - 1].join(".");
        }
        else if wfile.ends_with(".pcm") { wfile = wfile[..wfile.len() - 4].to_string(); }

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
    let mut no: u32 = 0;
    let play = params.play;

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let mut sink = Sink::try_new(&stream_handle).unwrap();
    sink.set_speed(params.speed as f32);

    let mut readfile: Box<dyn Read> = if !rpipe { Box::new(File::open(rfile).unwrap()) } else { Box::new(std::io::stdin()) };
    let mut writefile: Box<dyn Write> = if !wpipe && !play { Box::new(File::create(format!("{}.pcm", wfile)).unwrap()) } else { Box::new(std::io::stdout()) };
    let (mut asfh, mut info) = (ASFH::new(), ASFH::new());

    let (mut head, mut overlap_fragment) = (Vec::new(), Vec::new());
    let pcm_fmt = params.pcm;

    if play { loglevel = 0; }
    let mut log = LogObj::new(loglevel, 0.5);

    loop { // Main decode loop
        // 1. Reading the header
        if head != common::FRM_SIGN {
            let mut buf = vec![0u8; if head.is_empty() { 4 } else { 1 }];
            let readlen = common::read_exact(&mut readfile, &mut buf);
            if readlen == 0 {
                log.update(0, overlap_fragment.len(), asfh.srate);
                flush(play, &mut writefile, &mut sink, overlap_fragment, &pcm_fmt, asfh.srate);
                break;
            }
            if head.is_empty() { head = buf.to_vec(); }
            else { head = head.iter().chain(buf.iter()).skip(1).cloned().collect(); }
            continue;
        }
        // 2. Reading the frame
        head = Vec::new();
        let force_flush = asfh.update(&mut readfile);

        // 2.5. Force flush
        if force_flush {
            flush(play, &mut writefile, &mut sink, overlap_fragment, &pcm_fmt, asfh.srate);
            overlap_fragment = Vec::new(); continue;
        }

        // 3. Reading the frame data
        let mut frad = vec![0u8; asfh.frmbytes as usize];
        let _ = common::read_exact(&mut readfile, &mut frad);

        // 3.5. Checking if ASFH info has changed
        if !asfh.eq(&info) {
            if no != 0 { log.logging(true); }
            if !play {
                eprintln!("Track {}: Profile {}", no, asfh.profile);
                eprintln!("{}b@{} Hz / {} channel{}",
                    fourier::BIT_DEPTHS[asfh.profile as usize][asfh.bit_depth as usize],
                    asfh.srate, asfh.channels, if asfh.channels > 1 { "s" } else { "" }
                );
            }
            if info.srate != 0 || info.channels != 0 {
                flush(play, &mut writefile, &mut sink, overlap_fragment, &pcm_fmt, asfh.srate); // flush
                let name = format!("{}.{}.pcm", wfile, no);
                writefile = if !wpipe && !play { Box::new(File::create(name).unwrap()) } else { Box::new(std::io::stdout()) };
            }
            (info, overlap_fragment, no) = (asfh, Vec::new(), no + 1); // and create new file
        }

        // 4. Fixing errors
        if asfh.ecc {
            if fix_error && (
                LOSSLESS.contains(&asfh.profile) && common::crc32(&frad) != asfh.crc32 ||
                COMPACT.contains(&asfh.profile) && common::crc16_ansi(&frad) != asfh.crc16
            ) { frad = ecc::decode_rs(frad, asfh.ecc_ratio[0] as usize, asfh.ecc_ratio[1] as usize); }
            else { frad = ecc::unecc(frad, asfh.ecc_ratio[0] as usize, asfh.ecc_ratio[1] as usize); }
        }

        // 5. Decoding the frame
        let mut pcm =
        if asfh.profile == 1 { profile1::digital(frad, asfh.bit_depth, asfh.channels, asfh.srate) }
        else if asfh.profile == 4 { profile4::digital(frad, asfh.bit_depth, asfh.channels, asfh.endian) }
        else { profile0::digital(frad, asfh.bit_depth, asfh.channels, asfh.endian) };

        // 6. Overlapping
        (pcm, overlap_fragment) = overlap(pcm, overlap_fragment, &asfh);
        let samples = pcm.len();
        // 7. Writing to output
        flush(play, &mut writefile, &mut sink, pcm, &pcm_fmt, asfh.srate);

        log.update(asfh.total_bytes, samples, asfh.srate); log.logging(false);
    }
    log.logging(true);
    if play { sink.sleep_until_end(); }
}