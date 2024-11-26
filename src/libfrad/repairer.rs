/**                                 Repairer                                  */
/**
 * Copyright 2024 HaמuL
 * Description: FrAD repairer
 */

use crate::{
    backend::{SplitFront, VecPatternFind},
    common:: {crc16_ansi, crc32, FRM_SIGN},
    fourier::profiles::{COMPACT, LOSSLESS},
    tools::  {asfh::{ASFH, ParseResult::{Complete, Incomplete, ForceFlush}}, ecc},
};

/** Repairer
* Struct for FrAD repairer
*/
pub struct Repairer {
    asfh: ASFH,
    buffer: Vec<u8>,

    olap_len: usize,
    ecc_ratio: [u8; 2],
}

impl Repairer {
    pub fn new(mut ecc_ratio: [u8; 2]) -> Repairer {
        if ecc_ratio[0] == 0 {
            eprintln!("ECC data size must not be zero");
            eprintln!("Setting ECC to default 96 24");
            ecc_ratio = [96, 24];
        }
        if ecc_ratio[0] as i16 + ecc_ratio[1] as i16 > 255 {
            eprintln!("ECC data size and check size must not exceed 255, given: {} and {}",
                ecc_ratio[0], ecc_ratio[1]);
            eprintln!("Setting ECC to default 96 24");
            ecc_ratio = [96, 24];
        }

        Repairer {
            asfh: ASFH::new(),
            buffer: Vec::new(),

            olap_len: 0,
            ecc_ratio,
        }
    }

    /** is_empty
     * Check if the buffer is shorter than the frame sign(, which means it's virtually empty)
     * Returns: Empty flag
     */
    pub fn is_empty(&self) -> bool { return self.buffer.len() < FRM_SIGN.len(); }

    /** get_asfh
     * Get a reference to the ASFH struct
     * Returns: Immutable reference to the ASFH struct
     */
    pub fn get_asfh(&self) -> &ASFH { return &self.asfh; }

    /** process
     * Process the input stream and repair the FrAD stream
    * Parameters: Input stream
    * Returns: Repaired FrAD stream
    */
    pub fn process(&mut self, stream: Vec<u8>) -> Vec<u8> {
        self.buffer.extend(stream);
        let mut ret = Vec::new();

        loop {
            // If every parameter in the ASFH struct is set,
            /* 1. Repairing FrAD Frame */
            if self.asfh.all_set {
                // 1.0. If the buffer is not enough to decode the frame, break
                if self.buffer.len() < self.asfh.frmbytes as usize { break; }

                let samples_real = if self.asfh.overlap_ratio == 0 || LOSSLESS.contains(&self.asfh.profile) { self.asfh.fsize as usize } else {
                    (self.asfh.fsize as usize * (self.asfh.overlap_ratio as usize - 1)) / self.asfh.overlap_ratio as usize };
                self.olap_len = self.asfh.fsize as usize - samples_real;

                // 1.1. Split out the frame data
                let mut frad: Vec<u8> = self.buffer.split_front(self.asfh.frmbytes as usize);

                // 1.2. Correct the error if ECC is enabled
                if self.asfh.ecc {
                    let repair = // and if CRC mismatch
                        LOSSLESS.contains(&self.asfh.profile) && crc32(&frad) != self.asfh.crc32 ||
                        COMPACT.contains(&self.asfh.profile) && crc16_ansi(&frad) != self.asfh.crc16;
                    frad = ecc::decode(frad, self.asfh.ecc_ratio, repair);
                }

                // 1.3. Create Reed-Solomon error correction code
                frad = ecc::encode(frad, self.ecc_ratio);
                (self.asfh.ecc, self.asfh.ecc_ratio) = (true, self.ecc_ratio);

                // 1.4. Write the frame data to the buffer
                ret.extend(self.asfh.write(frad));

                // 1.5. Clear the ASFH struct
                self.asfh.clear();
            }

            /* 2. Finding header / Gathering more data to parse */
            else {
                // 2.1. If the header buffer not found, find the header buffer
                if !self.asfh.buffer.starts_with(&FRM_SIGN) {
                    match self.buffer.find_pattern(&FRM_SIGN) {
                        // If pattern found in the buffer
                        // 2.1.1. Split out the buffer to the header buffer
                        Some(i) => {
                            ret.extend(self.buffer.split_front(i));
                            self.asfh.buffer = self.buffer.split_front(FRM_SIGN.len());
                        },
                        // 2.1.2. else, Split out the buffer to the last 4 bytes and return
                        None => {
                            ret.extend(self.buffer.split_front(self.buffer.len().saturating_sub(FRM_SIGN.len() - 1)));
                            break;
                        }
                    }
                }
                // 2.2. If header buffer found, try parsing the header
                let force_flush = self.asfh.read(&mut self.buffer);

                // 2.3. Check header parsing result
                match force_flush {
                    // 2.3.1. If header is complete and not forced to flush, continue
                    Complete => {},
                    // 2.3.2. If header is complete and forced to flush, flush and return
                    ForceFlush => {
                        ret.extend(self.asfh.force_flush());
                        self.olap_len = 0;
                        break;
                    },
                    // 2.3.3. If header is incomplete, return
                    Incomplete => break,
                }
            }
        }
        return ret;
    }

    /** flush
     * Flush the remaining buffer
    * Parameters: None
    * Returns: Repairer buffer
    */
    pub fn flush(&mut self) -> Vec<u8> {
        let ret = self.buffer.clone();
        self.buffer.clear();
        return ret;
    }
}