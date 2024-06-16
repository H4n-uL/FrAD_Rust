use std::fs::File;
use std::io::{Read, Write};
use crate::{fourier, fourier::profiles::profile1,
    common, tools};

// use libsoxr::Soxr;

const FRM_SIGN: [u8; 4] = [0xff, 0xd0, 0xd2, 0x97];

fn encode_pfb(profile: u8, enable_ecc: bool, little_endian: bool, bits: i16) -> Vec<u8> {
    let prf = profile << 5;
    let ecc = (enable_ecc as u8) << 4;
    let endian = (little_endian as u8) << 3;
    return vec![(prf | ecc | endian | bits as u8) as u8];
}

fn encode_css_prf1(channels: i16, srate: u32, fsize: u32) -> Vec<u8> {
    let chnl = (channels as u16 - 1) << 10;
    let srate = (profile1::SRATES.iter().position(|&x| x == srate).unwrap() as u16) << 6;
    let fsize = *profile1::SMPLS_LI.iter().find(|&&x| x >= fsize).unwrap();
    let mult = profile1::get_smpls_from_value(&fsize);
    let px = (profile1::SMPLS.iter().position(|&(key, _)| key == mult).unwrap() as u16) << 4;
    let fsize = ((fsize as f64 / mult as f64).log2() as u16) << 1;
    return (chnl | srate | px | fsize).to_be_bytes().to_vec();
}

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
    let fsize: u32 = 2048;
    let little_endian: bool = false;

    let enable_ecc = false;
    let ecc_rate: [u8; 2] = [96, 24];
    let profile: u8 = 0;

    let olap: u8 = 16;

    let mut prev: Vec<Vec<f64>> = Vec::new();

    loop {
        let mut rlen = fsize as usize;

        if profile == 1 {
            rlen = *profile1::SMPLS_LI.iter().find(|&&x| x >= fsize).unwrap() as usize - prev.len();
            if rlen <= 0 { rlen = *profile1::SMPLS_LI.iter().find(|&&x| x - prev.len() as u32 >= fsize).unwrap() as usize - prev.len(); }
        }
        let fbytes = rlen * channels as usize * 8;
        // thread::sleep(Duration::from_millis(100));
        let mut pcm_buf = vec![0u8; fbytes];
        match readfile.read(&mut pcm_buf) {
            Ok(readlen) => {
                if readlen == 0 { break; }
                let pcm: Vec<f64> = pcm_buf[..readlen].chunks(8)
                .map(|bytes: &[u8]| f64::from(f64::from_be_bytes(bytes.try_into().unwrap())))
                .collect();

                let mut frame: Vec<Vec<f64>> = (0..fsize)
                .take_while(|&i| (i as usize + 1) * channels as usize <= pcm.len())
                .map(|i| pcm[i as usize * (channels as usize)..(i + 1) as usize * (channels as usize)].to_vec())
                .collect();

                // Overlapping for Profile 1
                (frame, prev) = overlap(frame, prev, olap, profile);
                let plen: u32 = frame.len() as u32;

                // Encoding
                let (mut frad, bits) = 
                if profile == 1 { profile1::analogue(frame, bit_depth, srate, 0) }
                else { fourier::analogue(frame, bit_depth, little_endian) };

                if enable_ecc { // Creating Reed-Solomon error correction code
                    frad = tools::ecc::encode_rs(frad, ecc_rate[0] as usize, ecc_rate[1] as usize);
                }

                // Frame writing

                let pfb = encode_pfb(profile, enable_ecc, little_endian, bits);

                let mut buffer = Vec::new();
                buffer.extend_from_slice(&FRM_SIGN);
                buffer.extend_from_slice(&(frad.len() as u32).to_be_bytes());
                buffer.extend_from_slice(&pfb);
                if profile == 1 {
                    buffer.extend_from_slice(&encode_css_prf1(channels, srate, fsize));
                    buffer.extend_from_slice(&olap.to_be_bytes());
                    if enable_ecc {
                        buffer.extend_from_slice(&ecc_rate);
                        buffer.extend_from_slice(&common::crc16_ansi(&frad));
                    }
                }
                else {
                    buffer.extend_from_slice(&[(channels-1) as u8]);
                    buffer.extend_from_slice(&(if enable_ecc { ecc_rate } else { [0; 2] }));
                    buffer.extend_from_slice(&srate.to_be_bytes());

                    buffer.extend_from_slice(&[0; 8]);

                    buffer.extend_from_slice(&plen.to_be_bytes());
                    buffer.extend_from_slice(&common::crc32(&frad));
                }
                buffer.extend_from_slice(&frad);

                writefile.write(buffer.as_slice()).unwrap();
            },
            Err(_e) => {
                break;
            }
        }
    }
}