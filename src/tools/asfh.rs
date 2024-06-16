use crate::{common::FRM_SIGN, fourier::profiles::profile1};
use std::{fs::File, io::Read};

fn decode_pfb(pfb: u8) -> (u8, bool, bool, i16) {
    let prf = pfb >> 5;
    let ecc = (pfb >> 4) & 1 == 1;
    let endian = (pfb >> 3) & 1 == 1;
    let bits = pfb & 0b111;
    return (prf, ecc, endian, bits as i16);
}

fn decode_css_prf1(css: Vec<u8>) -> (i16, u32, u32) {
    let css_int = u16::from_be_bytes(css[0..2].try_into().unwrap());
    let chnl = (css_int >> 10) as i16 + 1;
    let srate = profile1::SRATES[(css_int >> 6) as usize & 0b1111];

    let fsize_prefix = profile1::SMPLS[(css_int >> 4) as usize & 0b11].0;
    let fsize = fsize_prefix * 2u32.pow(((css_int >> 1) & 0b111) as u32);

    return (chnl, srate, fsize);
}

pub struct ASFH {
    // Audio Stream Frame Header
    pub frmbytes: u64,

    // Profile-Float byte
    pub profile: u8,
    pub ecc: bool,
    pub endian: bool,
    pub bit_depth: i16,

    // Profile 0
    pub channels: i16,
    pub ecc_rate: [u8; 2],
    pub srate: u32,
    pub fsize: u32,
    pub crc32: [u8; 4],

    // Profile 1
    pub olap: u8,
    pub crc16: [u8; 2],

    pub headlen: usize
}

impl ASFH {
    pub fn new() -> ASFH {
        ASFH {
            frmbytes: 0,
            profile: 0,
            ecc: false,
            endian: false,
            bit_depth: 0,
            channels: 0,
            ecc_rate: [0; 2],
            srate: 0,
            fsize: 0,
            olap: 0,
            crc32: [0; 4],
            crc16: [0; 2],
            headlen: 0
        }
    }

    pub fn update(&mut self, file: &mut File) {
        let mut fhead = FRM_SIGN.to_vec();

        let mut buf = vec![0u8; 5]; let _ = file.read(&mut buf).unwrap();
        fhead.extend(buf);

        self.frmbytes = u32::from_be_bytes(fhead[0x4..0x8].try_into().unwrap()) as u64;
        (self.profile, self.ecc, self.endian, self.bit_depth) = decode_pfb(fhead[0x8]);

        if self.profile == 1 {
            buf = vec![0u8; 3]; let _ = file.read(&mut buf).unwrap();
            fhead.extend(buf);
            (self.channels, self.srate, self.fsize) = decode_css_prf1(fhead[0x9..0xb].to_vec());
            self.olap = fhead[0xb];
            if self.ecc {
                buf = vec![0u8; 4]; let _ = file.read(&mut buf).unwrap();
                fhead.extend(buf);
                self.ecc_rate = [fhead[0xc], fhead[0xd]];
                self.crc16 = fhead[0xe..0x10].try_into().unwrap();
            }
        }
        else {
            buf = vec![0u8; 23]; let _ = file.read(&mut buf).unwrap();
            fhead.extend(buf);
            self.channels = fhead[0x9] as i16 + 1;
            self.ecc_rate = [fhead[0xa], fhead[0xb]];
            self.srate = u32::from_be_bytes(fhead[0xc..0x10].try_into().unwrap());

            self.fsize = u32::from_be_bytes(fhead[0x18..0x1c].try_into().unwrap());
            self.crc32 = fhead[0x1c..0x20].try_into().unwrap();
        }

        if self.frmbytes == u32::MAX as u64 {
            buf = vec![0u8; 23]; let _ = file.read(&mut buf).unwrap();
            fhead.extend(buf);
            self.frmbytes = u64::from_be_bytes(fhead[fhead.len()-8..].try_into().unwrap());
        }

        self.headlen = fhead.len();
    }
}