mod fourier;
mod tools;
use std::fs::File;
use std::io::{Read, Write};

use std::time::Instant;

// use libsoxr::Soxr;

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffffffff;
    for byte in data {
        crc ^= *byte as u32;
        for _ in 0..8 {
            if crc & 1 == 1 { crc = (crc >> 1) ^ 0xedb88320; }
            else            { crc >>= 1; }
        }
    }
    crc ^ 0xffffffff
}

const FRM_SIGN: [u8; 4] = [0xff, 0xd0, 0xd2, 0x97];

fn encode_pfb(profile: u8, enable_ecc: bool, little_endian: bool, bits: i16) -> Vec<u8> {
    let prf = profile << 5;
    let ecc = (enable_ecc as u8) << 4;
    let endian = (little_endian as u8) << 3;
    return vec![(prf | ecc | endian | bits as u8) as u8];
}
fn _decode_pfb(pfb: Vec<u8>) -> (u8, bool, bool, i16) {
    let pfb = pfb[0];
    let profile = pfb >> 5;
    let enable_ecc = (pfb >> 4) & 1 == 1;
    let little_endian = (pfb >> 3) & 1 == 1;
    let bits = pfb & 0x07;
    return (profile, enable_ecc, little_endian, bits as i16);
}

fn main() {
    let bit_depth: i16 = 64;
    let channels: i16 = 2;
    let srate: u32 = 96000;
    let mut readfile = File::open("test.pcm").unwrap();
    let mut writefile = File::create("test.frad").unwrap();
    let fsize: u32 = 2048;
    let fbytes: u32 = fsize * channels as u32 * 8;
    let little_endian: bool = false;

    let enable_ecc = true;
    let ecc_rate: [u8; 2] = [96, 24];
    let profile: u8 = 0;

    // let prev = Vec::new();

    loop {
        let mut pcm_buf = vec![0u8; fbytes as usize];
        match readfile.read(&mut pcm_buf){
            Ok(rlen) => {
                if rlen == 0 { break; }

                let start = Instant::now();
                let pcm: Vec<f64> = pcm_buf[..rlen].chunks(8)
                    .map(|bytes| f64::from(f64::from_be_bytes(bytes.try_into().unwrap())))
                    .collect();
                println!("Read time: {:?}", start.elapsed());

                let start = Instant::now();
                let pcm_t: Vec<Vec<f64>> = (0..fsize)
                    .take_while(|&i| (i as usize + 1) * channels as usize <= pcm.len())
                    .map(|i| pcm[i as usize * (channels as usize)..(i + 1) as usize * (channels as usize)].to_vec())
                    .collect();
                let plen: u32 = pcm_t.len() as u32;
                println!("Interleaving time: {:?}", start.elapsed());

                let start = Instant::now();
                let (mut frad, bits) = fourier::analogue(pcm_t, bit_depth, little_endian);
                println!("Transform time: {:?}", start.elapsed());

                let start = Instant::now();
                if enable_ecc {
                    frad = tools::ecc::encode_rs(frad, ecc_rate[0] as usize, ecc_rate[1] as usize);
                }
                println!("ECC time: {:?}", start.elapsed());

                let start = Instant::now();
                let pfb = encode_pfb(profile, enable_ecc, little_endian, bits);

                let mut buffer = Vec::new();
                buffer.extend_from_slice(&FRM_SIGN);
                buffer.extend_from_slice(&(frad.len() as u32).to_be_bytes());
                buffer.extend_from_slice(&pfb);
                if profile == 0 {
                    buffer.extend_from_slice(&[(channels-1) as u8]);
                    buffer.extend_from_slice(&(if enable_ecc { ecc_rate } else { [0; 2] }));
                    buffer.extend_from_slice(&srate.to_be_bytes());

                    buffer.extend_from_slice(&[0; 8]);

                    buffer.extend_from_slice(&plen.to_be_bytes());
                    buffer.extend_from_slice(&crc32(&frad).to_be_bytes());
                }
                buffer.extend_from_slice(&frad);

                writefile.write(buffer.as_slice()).unwrap();
                println!("Write time: {:?}", start.elapsed());
                println!();
            },
            Err(_e) => {
                break;
            }
        }
    }
    // // Transmission as bytestream

    // let frad = tools::ecc::decode_rs(stream, 96, 24);
    // let pcm = fourier::digital(frad, bits, channels, little_endian);
    // println!("{:?}", pcm);

    // // let srate = 44100.0;
    // // let new_srate = 48000.0;
    // // let soxr = Soxr::create(srate, new_srate, channels as u32, None, None, None).unwrap();
    // // let mut target = vec![vec![0.0; channels as usize]; (pcm.len() as f64 * new_srate / srate).ceil() as usize];

    // // let _ = soxr.process(Some(&pcm), &mut target);
    // // soxr.process::<f64, _>(None, &mut target[0..]).unwrap();

    // // println!("{:?}", target);
}