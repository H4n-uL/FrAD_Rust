/**                                ASFH Tools                                 */
/**
 * Copyright 2024 HaמuL
 * Function: Audio Stream Frame Header tools
 */

use crate::{common::{crc16_ansi, crc32, read_exact, FRM_SIGN}, fourier::profiles::{compact, COMPACT}};
use std::io::Read;

/** encode_pfb
 * Encodes PFloat byte (containing necessary info for the frame)
 * Parameters: Profile, ECC toggle, Little-endian toggle, Bit depth index
 * Returns: Encoded byte
 */
fn encode_pfb(profile: u8, enable_ecc: bool, little_endian: bool, bits: i16) -> Vec<u8> {
    let prf = profile << 5;
    let ecc = (enable_ecc as u8) << 4;
    let endian = (little_endian as u8) << 3;
    return vec![(prf | ecc | endian | bits as u8)];
}

/** encode_css
 * Encodes Channel-Srate-Smpcount byte for Compact Profiles
 * Parameters: Channel count, Sample rate, Sample count
 * Returns: Encoded CSS
 */
fn encode_css(channels: i16, srate: u32, fsize: u32) -> Vec<u8> {
    let chnl = (channels as u16 - 1) << 10;
    let srate = (compact::SRATES.iter().position(|&x| x == srate).unwrap() as u16) << 6;
    let fsize = *compact::SAMPLES_LI.iter().find(|&&x| x >= fsize).unwrap();
    let mult = compact::get_samples_from_value(&fsize);
    let px = (compact::SAMPLES.iter().position(|&(key, _)| key == mult).unwrap() as u16) << 4;
    let fsize = ((fsize as f64 / mult as f64).log2() as u16) << 1;
    return (chnl | srate | px | fsize).to_be_bytes().to_vec();
}

/** decode_pfb
 * Decodes PFloat byte
 * Parameters: Encoded byte
 * Returns: Profile, ECC toggle, Little-endian toggle, Bit depth index
 */
fn decode_pfb(pfb: u8) -> (u8, bool, bool, i16) {
    let prf = pfb >> 5;
    let ecc = (pfb >> 4) & 1 == 1;
    let endian = (pfb >> 3) & 1 == 1;
    let bits = pfb & 0b111;
    return (prf, ecc, endian, bits as i16);
}

/** decode_css
 * Decodes Channel-Srate-Smpcount byte for Compact Profiles
 * Parameters: Encoded CSS
 * Returns: Channel count, Sample rate, Sample count
 */
fn decode_css(css: Vec<u8>) -> (i16, u32, u32) {
    let css_int = u16::from_be_bytes(css[0..2].try_into().unwrap());
    let chnl = (css_int >> 10) as i16 + 1;
    let srate = compact::SRATES[(css_int >> 6) as usize & 0b1111];

    let fsize_prefix = compact::SAMPLES[(css_int >> 4) as usize & 0b11].0;
    let fsize = fsize_prefix * 2u32.pow(((css_int >> 1) & 0b111) as u32);

    return (chnl, srate, fsize);
}

/** ASFH
 * Audio Stream Frame Header
 */
#[derive(Clone, Copy)]
pub struct ASFH {
    // Audio Stream Frame Header
    pub total_bytes: u128,
    pub frmbytes: u64,

    // PFloat byte
    pub profile: u8,
    pub ecc: bool,
    pub endian: bool,
    pub bit_depth: i16,

    // Profile 0
    pub channels: i16,
    pub ecc_ratio: [u8; 2],
    pub srate: u32,
    pub fsize: u32,
    pub crc32: [u8; 4],

    // Profile 1
    pub olap: u8,
    pub crc16: [u8; 2],
}

impl ASFH {
    pub fn new() -> ASFH {
        ASFH {
            total_bytes: 0,
            frmbytes: 0,
            profile: 0,
            ecc: false,
            endian: false,
            bit_depth: 0,
            channels: 0,
            ecc_ratio: [0; 2],
            srate: 0,
            fsize: 0,
            olap: 0,
            crc32: [0; 4],
            crc16: [0; 2],
        }
    }

    pub fn eq(&self, other: &ASFH) -> bool {
        return self.profile == other.profile && self.bit_depth == other.bit_depth &&
            self.channels == other.channels && self.srate == other.srate && self.olap == other.olap;
    }

    /** write_vec
     * Makes a frame from audio frame and metadata
     * Parameters: Audio frame
     */
    pub fn write_vec(&mut self, frad: Vec<u8>) -> Vec<u8> {
        let mut fhead = FRM_SIGN.to_vec();

        fhead.extend(&(frad.len() as u32).to_be_bytes().to_vec());
        fhead.push(encode_pfb(self.profile, self.ecc, self.endian, self.bit_depth)[0]);

        if COMPACT.contains(&self.profile) {
            fhead.extend(encode_css(self.channels, self.srate, self.fsize));
            fhead.push(self.olap);
            if self.ecc {
                fhead.extend(self.ecc_ratio.to_vec());
                fhead.extend(crc16_ansi(&frad).to_vec());
            }
        }
        else {
            fhead.push((self.channels - 1) as u8);
            fhead.extend(self.ecc_ratio.to_vec());
            fhead.extend(self.srate.to_be_bytes().to_vec());
            fhead.extend([0u8; 8].to_vec());
            fhead.extend(self.fsize.to_be_bytes().to_vec());
            fhead.extend(crc32(&frad).to_vec());
        }

        let frad = fhead.iter().chain(frad.iter()).cloned().collect::<Vec<u8>>();
        self.total_bytes = frad.len() as u128;
        return frad;
    }

    /** update
     * Updates the ASFH from a file
     * Parameters: File
     */
    pub fn update(&mut self, file: &mut Box<dyn Read>) {
        let mut fhead = FRM_SIGN.to_vec();

        let mut buf = vec![0u8; 5]; let _ = read_exact(file, &mut buf);
        fhead.extend(buf);

        self.frmbytes = u32::from_be_bytes(fhead[0x4..0x8].try_into().unwrap()) as u64;
        (self.profile, self.ecc, self.endian, self.bit_depth) = decode_pfb(fhead[0x8]);

        if COMPACT.contains(&self.profile) {
            buf = vec![0u8; 3]; let _ = read_exact(file, &mut buf);
            fhead.extend(buf);
            (self.channels, self.srate, self.fsize) = decode_css(fhead[0x9..0xb].to_vec());
            self.olap = fhead[0xb];
            if self.ecc {
                buf = vec![0u8; 4]; let _ = file.read(&mut buf).unwrap();
                fhead.extend(buf);
                self.ecc_ratio = [fhead[0xc], fhead[0xd]];
                self.crc16 = fhead[0xe..0x10].try_into().unwrap();
            }
        }
        else {
            buf = vec![0u8; 23]; let _ = read_exact(file, &mut buf);
            fhead.extend(buf);
            self.channels = fhead[0x9] as i16 + 1;
            self.ecc_ratio = [fhead[0xa], fhead[0xb]];
            self.srate = u32::from_be_bytes(fhead[0xc..0x10].try_into().unwrap());

            self.fsize = u32::from_be_bytes(fhead[0x18..0x1c].try_into().unwrap());
            self.crc32 = fhead[0x1c..0x20].try_into().unwrap();
        }

        if self.frmbytes == u32::MAX as u64 {
            buf = vec![0u8; 8]; let _ = read_exact(file, &mut buf);
            fhead.extend(buf);
            self.frmbytes = u64::from_be_bytes(fhead[fhead.len()-8..].try_into().unwrap());
        }

        self.total_bytes = fhead.len() as u128 + self.frmbytes as u128;
    }
}