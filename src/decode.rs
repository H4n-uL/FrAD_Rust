use crate::{fourier, fourier::profiles::profile1};
use crate::common;
use crate::tools::ecc;

use super::common::FRM_SIGN;
use std::{fs::File, io::{Read, Write}};
use super::tools::asfh::ASFH;

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

pub fn decode() {
    let mut readfile = File::open("test.frad").unwrap();
    let writefile = File::create("res.pcm").unwrap();
    let mut asfh = ASFH::new();
    let fix_error = true;

    let mut head = Vec::new();
    let mut prev = Vec::new();
    loop {
        if head.len() == 0 {
            let mut buf = vec![0u8; 4];
            let readlen = readfile.read(&mut buf).unwrap();
            if readlen == 0 { flush(&writefile, prev); break; }
            head = buf.to_vec();
        }
        if head != FRM_SIGN {
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

        let mut pcm = //fourier::digital(frad, asfh.bit_depth, asfh.channels, asfh.endian);
        if asfh.profile == 1 { profile1::digital(frad, asfh.bit_depth, asfh.channels, asfh.endian, asfh.srate) }
        else { fourier::digital(frad, asfh.bit_depth, asfh.channels, asfh.endian) };

        (pcm, prev) = overlap(pcm, prev, &asfh);

        flush(&writefile, pcm);
        head = Vec::new();
    }
}