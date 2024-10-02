/**                            Decode application                             */
/**
 * Copyright 2024 HaמuL
 * Description: Decoder implementation example
 */

use frad::{f64cvt::f64_to_any, PCMFormat, Decoder};
use crate::{
    common::{check_overwrite, logging, read_exact, write_safe, PIPEIN, PIPEOUT},
    tools::cli::CliParams
};
use std::{fs::File, io::{Read, Write}, path::Path, process::exit};

use rodio::{buffer::SamplesBuffer, OutputStream, Sink};
use same_file::is_same_file;

/** write
 * Writes PCM data to file or sink
 * Parameters: Play flag, Output file/sink, PCM data, PCM format, Sample rate
 * Parameters: Output file, PCM data
 * Returns: None
 */
fn write(isplay: bool, file: &mut Box<dyn Write>, sink: &mut Sink, pcm: Vec<Vec<f64>>, fmt: &PCMFormat, srate: u32) {
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
        write_safe(file, &pcm_bytes);
    }
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

    if wfile_prim.is_empty() {
        let wfrf = Path::new(&rfile).file_name().unwrap().to_str().unwrap().to_string();
        wfile_prim = wfrf.split(".").collect::<Vec<&str>>()[..wfrf.split(".").count() - 1].join(".");
    }
    else if wfile_prim.ends_with(".pcm") { wfile_prim = wfile_prim[..wfile_prim.len() - 4].to_string(); }

    let mut wfile = format!("{}.pcm", wfile_prim);
    if !wpipe { check_overwrite(&wfile, params.overwrite); }

    let mut readfile: Box<dyn Read> = if !rpipe { Box::new(File::open(rfile).unwrap()) } else { Box::new(std::io::stdin()) };
    let mut writefile: Box<dyn Write> = if !wpipe { Box::new(File::create(wfile).unwrap()) } else { Box::new(std::io::stdout()) };
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let mut sink = Sink::try_new(&stream_handle).unwrap();
    sink.set_speed(params.speed as f32);

    if play { params.loglevel = 0; }
    let mut decoder = Decoder::new(params.enable_ecc);
    let pcm_fmt = params.pcm;

    let mut no = 0;
    loop {
        let mut buf = vec![0u8; 32768];
        let readlen = read_exact(&mut readfile, &mut buf);

        if readlen == 0 && decoder.is_empty() && (!play || sink.empty()) { break; }

        let decoded = decoder.process(buf[..readlen].to_vec());
        write(play, &mut writefile, &mut sink, decoded.pcm, &pcm_fmt, decoded.srate);
        logging(params.loglevel, &decoder.streaminfo, false);

        if decoded.crit && !wpipe {
            no += 1; wfile = format!("{}.{}.pcm", wfile_prim, no);
            decoder.streaminfo.block();
            check_overwrite(&wfile, params.overwrite);
            decoder.streaminfo.unblock();
            writefile = Box::new(File::create(wfile).unwrap());
        }
    }
    let decoded = decoder.flush();
    write(play, &mut writefile, &mut sink, decoded.pcm, &pcm_fmt, decoded.srate);
    logging(params.loglevel, &decoder.streaminfo, true);
    if play { sink.sleep_until_end(); }
}