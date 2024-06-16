use std::fs::File;
use std::io::{Read, Write};

use crate::{fourier, fourier::profiles::profile1,
    tools::{asfh::ASFH, ecc}};

// use libsoxr::Soxr;

fn overlap(data: Vec<Vec<f64>>, prev: Vec<Vec<f64>>, olap: u8, profile: u8) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let mut ndata = Vec::new();
    let mut _nprev = Vec::new();
    let fsize = data.len() + prev.len();
    let olap = if olap > 0 { if olap > 2 { olap } else { 2 } } else { 0 };

    if prev.len() != 0 {
        ndata.extend(prev.iter().cloned());
        ndata.extend(data.iter().cloned());
    }
    else { ndata = data.clone(); }

    if profile == 1 || profile == 2 && olap > 0 {
        let cutoff = ndata.len() - (fsize as usize / olap as usize);
        _nprev = ndata[cutoff..].to_vec();
    }
    else { _nprev = Vec::new(); }
    return (ndata, _nprev);
}

pub fn encode() {
    let bit_depth: i16 = 24;
    let channels: i16 = 2;
    let srate: u32 = 48000;
    let mut readfile = File::open("test.pcm").unwrap();
    let mut writefile = File::create("test.frad").unwrap();
    let buffersize: u32 = 2048;
    let little_endian: bool = false;

    let enable_ecc = false;
    let ecc_rate: [u8; 2] = [96, 24];
    let profile: u8 = 0;

    let olap: u8 = 16;

    let mut asfh = ASFH::new();
    let mut prev: Vec<Vec<f64>> = Vec::new();

    loop {
        let mut rlen = buffersize as usize;

        if profile == 1 {
            rlen = *profile1::SMPLS_LI.iter().find(|&&x| x >= buffersize).unwrap() as usize - prev.len();
            if rlen <= 0 { rlen = *profile1::SMPLS_LI.iter().find(|&&x| x - prev.len() as u32 >= buffersize).unwrap() as usize - prev.len(); }
        }
        let fbytes = rlen * channels as usize * 8;
        // thread::sleep(Duration::from_millis(100));
        let mut pcm_buf = vec![0u8; fbytes];

        let readlen = readfile.read(&mut pcm_buf).unwrap();
        if readlen == 0 { break; }
        let pcm: Vec<f64> = pcm_buf[..readlen].chunks(8)
        .map(|bytes: &[u8]| f64::from(f64::from_be_bytes(bytes.try_into().unwrap())))
        .collect();

        let mut frame: Vec<Vec<f64>> = (0..buffersize)
        .take_while(|&i| (i as usize + 1) * channels as usize <= pcm.len())
        .map(|i| pcm[i as usize * (channels as usize)..(i + 1) as usize * (channels as usize)].to_vec())
        .collect();

        // Overlapping for Profile 1
        (frame, prev) = overlap(frame, prev, olap, profile);
        let fsize: u32 = frame.len() as u32;

        // Encoding
        let (mut frad, bits) = 
        if profile == 1 { profile1::analogue(frame, bit_depth, srate, 0) }
        else { fourier::analogue(frame, bit_depth, little_endian) };

        if enable_ecc { // Creating Reed-Solomon error correction code
            frad = ecc::encode_rs(frad, ecc_rate[0] as usize, ecc_rate[1] as usize);
        }

        // Writing to file
        (asfh.profile, asfh.ecc, asfh.endian, asfh.bit_depth) = (profile, enable_ecc, little_endian, bits);
        (asfh.channels, asfh.srate, asfh.fsize) = (channels, srate, fsize);
        (asfh.olap, asfh.ecc, asfh.ecc_rate) = (olap, enable_ecc, ecc_rate);

        let frad: Vec<u8> = asfh.write_vec(frad);

        writefile.write(frad.as_slice()).unwrap();
    }
}