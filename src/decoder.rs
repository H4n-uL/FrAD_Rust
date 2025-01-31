/**                            Decode application                             */
/**
 * Copyright 2024 HaמuL
 * Description: Decoder implementation example
 */

use frad::{Decoder, ASFH, PCMFormat, f64cvt::f64_to_any};
use crate::{
    common::{self, check_overwrite, get_file_stem, read_exact, write_safe, PIPEIN, PIPEOUT},
    tools::{cli::CliParams, process::ProcessInfo}
};
use std::{fs::File, io::{Read, Write}, path::Path, process::exit};

use rodio::{buffer::SamplesBuffer, OutputStream, Sink};
use same_file::is_same_file;

/** write
 * Writes PCM data to file or sink
 * Parameters: Output file/sink, PCM data, PCM format, Sample rate
 * Parameters: Output file, PCM data
 * Returns: None
 */
fn write(file: &mut Box<dyn Write>, sink: Option<&mut Sink>, pcm: Vec<Vec<f64>>, fmt: &PCMFormat, srate: u32) {
    if pcm.is_empty() { return; }
    match sink {
        Some(s) => s.append(SamplesBuffer::new(
                pcm[0].len() as u16, srate,
                pcm.into_iter().flatten().map(|x| x as f32).collect::<Vec<f32>>()
            )),
        None => {
            let pcm_bytes: Vec<u8> = pcm.into_iter().flatten().flat_map(|x| f64_to_any(x, fmt)).collect();
            write_safe(file, &pcm_bytes);
        }
    }
}

/** logging_decode
 * Logs a message to stderr
 * Parameters: Log level, Process info, Linefeed flag, ASFH
 */
fn logging_decode(loglevel: u8, log: &ProcessInfo, linefeed: bool, asfh: &ASFH) {
    if loglevel == 0 { return; }

    let mut out = Vec::new();

    out.push(format!("size={}B time={} bitrate={}bit/s speed={}x    ",
        common::format_si(log.get_total_size() as f64), common::format_time(log.get_duration()), common::format_si(log.get_bitrate()), common::format_speed(log.get_speed())
    ));
    if loglevel > 1 {
        out.push(format!("Profile {}, {}bits {}ch@{}Hz, ECC={}    ", asfh.profile,
            frad::BIT_DEPTHS[asfh.profile as usize][asfh.bit_depth_index as usize], asfh.channels, asfh.srate,
            if asfh.ecc { format!("{}/{}", asfh.ecc_ratio[0], asfh.ecc_ratio[1]) } else { "disabled".to_string() }
        ));
    }

    let line_count = out.len() - 1;
    eprint!("{}", out.join("\n"));

    if linefeed { eprintln!(); }
    else { for _ in 0..line_count { eprint!("\x1b[1A"); } eprint!("\r"); }
}

/** decode
 * Decodes any found FrAD frames in the input file to f64be PCM
 * Parameters: Input file, CLI parameters
 * Returns: Decoded PCM on File or stdout
 */
pub fn decode(rfile: String, mut params: CliParams, play: bool) {
    let mut wfile_prim = params.output;
    if rfile.is_empty() { eprintln!("Input file must be given"); exit(1); }

    let (mut rpipe, mut wpipe) = (false, false);
    if PIPEIN.contains(&rfile.as_str()) { rpipe = true; }
    else if !Path::new(&rfile).exists() { eprintln!("Input file does not exist"); exit(1); }
    if PIPEOUT.contains(&wfile_prim.as_str()) || play { wpipe = true; }
    else if let Ok(true) = is_same_file(&rfile, &wfile_prim) {
        eprintln!("Input and output files cannot be the same"); exit(1);
    }

    if wfile_prim.is_empty() { wfile_prim = get_file_stem(&rfile); }
    else if wfile_prim.ends_with(".pcm") { wfile_prim = wfile_prim[..wfile_prim.len() - 4].to_string(); }

    let mut wfile = format!("{}.pcm", wfile_prim);
    if !wpipe { check_overwrite(&wfile, params.overwrite); }

    let mut readfile: Box<dyn Read> = if !rpipe { Box::new(File::open(rfile).unwrap()) } else { Box::new(std::io::stdin()) };
    let mut writefile: Box<dyn Write> = if !wpipe { Box::new(File::create(wfile).unwrap()) } else { Box::new(std::io::stdout()) };

    let (_stream, _stream_handle, mut sink) = if play {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        (Some(_stream), Some(stream_handle), Some(sink))
    } else { (None, None, None) };

    params.speed = if params.speed > 0.0 { params.speed } else { 1.0 };
    sink.as_mut().map(|s| { s.set_speed(params.speed as f32); params.loglevel = 0; });
    let pcm_fmt = params.pcm;

    let mut decoder = Decoder::new(params.enable_ecc);
    let (mut no, mut procinfo) = (0, ProcessInfo::new());
    loop {
        let mut buf = vec![0u8; 32768];
        let readlen = read_exact(&mut readfile, &mut buf);
        if readlen == 0 && decoder.is_empty() && sink.as_ref().map_or(true, |s| s.empty()) { break; }

        let decoded = decoder.process(&buf[..readlen]);
        procinfo.update(readlen, decoded.pcm.len(), decoded.srate);
        write(&mut writefile, sink.as_mut(), decoded.pcm, &pcm_fmt, decoded.srate);
        logging_decode(params.loglevel, &procinfo, false, decoder.get_asfh());

        if decoded.crit && !wpipe {
            procinfo.block();
            no += 1; wfile = format!("{}.{}.pcm", wfile_prim, no);
            check_overwrite(&wfile, params.overwrite);
            writefile = Box::new(File::create(wfile).unwrap());
            procinfo.unblock();
        }
    }
    let decoded = decoder.flush();
    procinfo.update(0, decoded.pcm.len(), decoded.srate);
    write(&mut writefile, sink.as_mut(), decoded.pcm, &pcm_fmt, decoded.srate);
    logging_decode(params.loglevel, &procinfo, true, decoder.get_asfh());

    sink.map(|s| s.sleep_until_end());
}