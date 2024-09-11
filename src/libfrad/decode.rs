/**                                  Decode                                   */
/**
 * Copyright 2024 HaמuL
 * Description: FrAD decoder
 */

use crate::{
    backend::{linspace, SplitFront, VecPatternFind},
    common:: {crc16_ansi, crc32, FRM_SIGN},
    fourier::profiles::{profile0, profile1, profile4, COMPACT, LOSSLESS},
    tools::  {asfh::ASFH, ecc, stream::StreamInfo},
};

/** Decode
 * Struct for FrAD decoder
 */
pub struct Decode {
    asfh: ASFH, info: ASFH,
    buffer: Vec<u8>,
    overlap_fragment: Vec<Vec<f64>>,
    pub streaminfo: StreamInfo,

    fix_error: bool,
}

impl Decode {
    pub fn new(fix_error: bool) -> Decode {
        Decode {
            asfh: ASFH::new(), info: ASFH::new(),
            buffer: Vec::new(),
            overlap_fragment: Vec::new(),
            streaminfo: StreamInfo::new(),

            fix_error,
        }
    }

    /** overlap
     * Apply overlap to the decoded PCM
     * Parameters: Decoded PCM
     * Returns: PCM with overlap applied
     */
    fn overlap(&mut self, mut frame: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
        // 1. If overlap buffer not empty, apply Forward-linear overlap-add
        if !self.overlap_fragment.is_empty() {
            let fade_in: Vec<f64> = linspace(0.0, 1.0, self.overlap_fragment.len());
            let fade_out: Vec<f64> = linspace(1.0, 0.0, self.overlap_fragment.len());
            for c in 0..self.asfh.channels as usize {
                for i in 0..self.overlap_fragment.len() {
                    frame[i][c] = frame[i][c] * fade_in[i] + self.overlap_fragment[i][c] * fade_out[i];
                }
            }
        }

        // 2. if COMPACT profile and overlap is enabled, split this frame
        let mut next_overlap = Vec::new();
        if COMPACT.contains(&self.asfh.profile) && self.asfh.olap != 0 {
            let olap = self.asfh.olap.max(2);
            // return_frame         = frame[0 ~ (len*(olap-1)) / olap]
            // new_overlap_fragment = frame[(len*(olap-1)) / olap ~ len]
            // = [2048], olap=16 -> [1920, 128]
            next_overlap = frame.split_off((frame.len() * (olap as usize - 1)) / olap as usize);
        }
        self.overlap_fragment = next_overlap;
        return frame;
    }

    pub fn is_empty(&self) -> bool {
        return self.buffer.len() < FRM_SIGN.len();
    }

    /** process
     * Process the input stream and decode the FrAD frames
     * Parameters: Input stream
     * Returns: Decoded PCM, Sample rate, Critical info modification flag
     */
    pub fn process(&mut self, stream: Vec<u8>) -> (Vec<Vec<f64>>, u32, bool) {
        self.buffer.extend(stream);
        let mut ret = Vec::new();

        loop {
            // If every parameter in the ASFH struct is set,
            /* 1. Decoding FrAD Frame */
            if self.asfh.all_set {
                // 1.0. If the buffer is not enough to decode the frame, break
                if self.buffer.len() < self.asfh.frmbytes as usize { break; }

                // 1.1. Split out the frame data
                let mut frad: Vec<u8> = self.buffer.split_front(self.asfh.frmbytes as usize);

                // 1.2. Correct the error if ECC is enabled
                if self.asfh.ecc {
                    let repair =  self.fix_error && ( // and if the user requested
                        // and if CRC mismatch
                        LOSSLESS.contains(&self.asfh.profile) && crc32(&frad) != self.asfh.crc32 ||
                        COMPACT.contains(&self.asfh.profile) && crc16_ansi(&frad) != self.asfh.crc16
                    );
                    frad = ecc::decode(frad, self.asfh.ecc_ratio, repair);
                }

                // 1.3. Decode the FrAD frame
                let mut pcm =
                match self.asfh.profile {
                    1 => profile1::digital(frad, self.asfh.bit_depth, self.asfh.channels, self.asfh.srate),
                    4 => profile4::digital(frad, self.asfh.bit_depth, self.asfh.channels, self.asfh.endian),
                    _ => profile0::digital(frad, self.asfh.bit_depth, self.asfh.channels, self.asfh.endian)
                };

                // 1.4. Apply overlap
                pcm = self.overlap(pcm); let samples = pcm.len();
                self.streaminfo.update(&self.asfh.total_bytes, samples, &self.asfh.srate);

                // 1.5. Append the decoded PCM and clear header
                ret.extend(pcm);
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
                            self.buffer.split_front(i);
                            self.asfh.buffer = self.buffer.split_front(FRM_SIGN.len());
                        },
                        // 2.1.2. else, Split out the buffer to the last 4 bytes and return
                        None => {
                            self.buffer.split_front(self.buffer.len().saturating_sub(FRM_SIGN.len() - 1));
                            break;
                        }
                    }
                }
                // 2.2. If header buffer found, try parsing the header
                let force_flush = self.asfh.read(&mut self.buffer);

                // 2.3. Check header parsing result
                match force_flush {
                    // 2.3.1. If header is complete and not forced to flush, continue
                    Ok(false) => {
                        // 2.3.1.1. If any critical parameter has changed, flush the overlap buffer
                        if !self.asfh.criteq(&self.info) {
                            if self.info.srate != 0 || self.info.channels != 0 { // If the info struct is not empty
                                ret.extend(self.flush().0); // Flush the overlap buffer
                                let srate = self.info.srate; // Save the sample rate
                                self.info = self.asfh.clone(); // Update the info struct
                                return (ret, srate, true); // and return
                            }
                            self.info = self.asfh.clone(); // else, Update the info struct and continue
                        }
                    },
                    // 2.3.2. If header is complete and forced to flush, flush and return
                    Ok(true) => { ret.extend(self.flush().0); break; },
                    // 2.3.3. If header is incomplete, return
                    Err(_) => break,
                }
            }
        }
        return (ret, self.asfh.srate, false);
    }

    /** flush
     * Flush the overlap buffer
     * Parameters: None
     * Returns: Overlap buffer, Sample rate
     */
    pub fn flush(&mut self) -> (Vec<Vec<f64>>, u32) {
        // 1. Extract the overlap buffer
        // 2. Update stream info
        // 3. Clear the overlap buffer
        // 4. Clear the ASFH struct
        // 5. Return exctacted buffer

        let ret = self.overlap_fragment.clone();
        self.streaminfo.update(&0, self.overlap_fragment.len(), &self.asfh.srate);
        self.overlap_fragment.clear();
        self.asfh.clear();
        return (ret, self.asfh.srate);
    }
}