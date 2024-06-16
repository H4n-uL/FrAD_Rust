use std::fs::File;
use std::io::{Read, Write};

use crate::{common, tools::{asfh::ASFH, ecc}};

pub fn reecc() {
    let mut readfile = File::open("test.frad").unwrap();
    let mut writefile = File::create("test.ecc.frad").unwrap();
    let ecc_rate: [u8; 2] = [96, 24];

    let mut asfh = ASFH::new();

    let mut head = Vec::new();
    loop {
        if head.len() == 0 {
            let mut buf = vec![0u8; 4];
            let readlen = readfile.read(&mut buf).unwrap();
            if readlen == 0 { break; }
            head = buf.to_vec();
        }
        if head != common::FRM_SIGN {
            let mut buf = vec![0u8; 1];
            let readlen = readfile.read(&mut buf).unwrap();
            if readlen == 0 { break; }
            head.extend(buf);
            head = head[1..].to_vec();
            continue;
        }
        asfh.update(&mut readfile);

        let mut frad = vec![0u8; asfh.frmbytes as usize];
        let _ = readfile.read(&mut frad).unwrap();

        if asfh.ecc {
            if asfh.profile == 0 && common::crc32(&frad) != asfh.crc32 ||
                asfh.profile == 1 && common::crc16_ansi(&frad) != asfh.crc16
            { frad = ecc::decode_rs(frad, asfh.ecc_rate[0] as usize, asfh.ecc_rate[1] as usize); }
            else { frad = ecc::unecc(frad, asfh.ecc_rate[0] as usize, asfh.ecc_rate[1] as usize); }
        }

        frad = ecc::encode_rs(frad, ecc_rate[0] as usize, ecc_rate[1] as usize);

        // Writing to file
        (asfh.ecc, asfh.ecc_rate) = (true, ecc_rate);

        let frad: Vec<u8> = asfh.write_vec(frad);

        writefile.write(frad.as_slice()).unwrap();
        head = Vec::new();
    }
}