use crate::{fourier, fourier::profiles::profile1, 
    common, tools::{asfh::ASFH, ecc}};

use std::{fs::File, io::{Read, Write}, path::Path};

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

fn flush(mut file: &File, pcm: Vec<Vec<f64>>) {
    let pcm_flat: Vec<f64> = pcm.into_iter().flatten().collect();
    let pcm_bytes: Vec<u8> = pcm_flat.iter().map(|x| x.to_be_bytes()).flatten().collect();
    file.write(&pcm_bytes).unwrap();
}

pub fn decode(rfile: String, wfile: String, fix_error: bool) {
    if rfile.len() == 0 { panic!("Input file must be given"); }
    if rfile == wfile { panic!("Input and output files cannot be the same"); }
    let mut wfile = wfile;
    if wfile.len() == 0 {
        let wfile_prefix = rfile.split(".").collect::<Vec<&str>>()[..rfile.split(".").count() - 1].join(".");
        wfile = format!("{}.pcm", wfile_prefix);
    }

    if Path::new(&wfile).exists() {
        println!("Output file already exists, overwrite? (Y/N)");
        loop {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            if input.trim().to_lowercase() == "y" { break; }
            else if input.trim().to_lowercase() == "n" { 
                println!("Aborted.");
                std::process::exit(0);
            }
        }
    }

    let mut readfile = File::open(rfile).unwrap();
    let writefile = File::create(wfile).unwrap();
    let mut asfh = ASFH::new();

    let mut head = Vec::new();
    let mut prev = Vec::new();
    loop {
        if head.len() == 0 {
            let mut buf = vec![0u8; 4];
            let readlen = readfile.read(&mut buf).unwrap();
            if readlen == 0 { flush(&writefile, prev); break; }
            head = buf.to_vec();
        }
        if head != common::FRM_SIGN {
            let mut buf = vec![0u8; 1];
            let readlen = readfile.read(&mut buf).unwrap();
            if readlen == 0 { flush(&writefile, prev); break; }
            head.extend(buf);
            head = head[1..].to_vec();
            continue;
        }
        asfh.update(&mut readfile);

        let mut frad = vec![0u8; asfh.frmbytes as usize];
        let _ = readfile.read(&mut frad).unwrap();

        if asfh.ecc {
            if fix_error && (
                asfh.profile == 0 && common::crc32(&frad) != asfh.crc32 ||
                asfh.profile == 1 && common::crc16_ansi(&frad) != asfh.crc16
            ) { frad = ecc::decode_rs(frad, asfh.ecc_rate[0] as usize, asfh.ecc_rate[1] as usize); }
            else { frad = ecc::unecc(frad, asfh.ecc_rate[0] as usize, asfh.ecc_rate[1] as usize); }
        }

        let mut pcm =
        if asfh.profile == 1 { profile1::digital(frad, asfh.bit_depth, asfh.channels, asfh.endian, asfh.srate) }
        else { fourier::digital(frad, asfh.bit_depth, asfh.channels, asfh.endian) };

        (pcm, prev) = overlap(pcm, prev, &asfh);
        flush(&writefile, pcm);
        head = Vec::new();
    }
}