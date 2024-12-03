/**                                  Decoder                                  */
/**
 * Copyright 2024 HaמuL
 * Description: FrAD decoder
 */

use crate::{
    backend::{hanning_in_overlap, SplitFront, VecPatternFind},
    common:: {crc16_ansi, crc32, FRM_SIGN},
    fourier::{self, profiles::{COMPACT, LOSSLESS}},
    tools::  {asfh::{ASFH, ParseResult::{Complete, Incomplete, ForceFlush}}, ecc},
};

pub struct DecodeResult {
    pub pcm: Vec<Vec<f64>>,
    pub srate: u32,
    pub frames: usize,
    pub crit: bool,
}

/** Decoder
 * Struct for FrAD decoder
 */
pub struct Decoder {
    asfh: ASFH, info: ASFH,
    buffer: Vec<u8>,
    overlap_fragment: Vec<Vec<f64>>,

    fix_error: bool,
    broken_frame: bool,
}

impl Decoder {
    pub fn new(fix_error: bool) -> Decoder {
        return Decoder {
            asfh: ASFH::new(), info: ASFH::new(),
            buffer: Vec::new(),
            overlap_fragment: Vec::new(),

            fix_error,
            broken_frame: false,
        };
    }

    /** overlap
     * Apply overlap to the decoded PCM
     * Parameters: Decoded PCM
     * Returns: PCM with overlap applied
     */
    fn overlap(&mut self, mut frame: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
        // 1. If overlap buffer not empty, apply Forward linear overlap-add
        if !self.overlap_fragment.is_empty() {
            let fade_in = hanning_in_overlap(self.overlap_fragment.len());
            frame.iter_mut().zip(self.overlap_fragment.iter()).zip(fade_in.iter().zip(fade_in.iter().rev()))
            .for_each(|((sample, overlap_sample), (&fade_in, &fade_out))| {
                sample.iter_mut().zip(overlap_sample.iter()).for_each(|(s, &o)| { *s = *s * fade_in + o * fade_out; });
            });
        }

        // 2. if COMPACT profile and overlap is enabled, split this frame
        let mut next_overlap = Vec::new();
        if COMPACT.contains(&self.asfh.profile) && self.asfh.overlap_ratio != 0 {
            let overlap_ratio = self.asfh.overlap_ratio as usize;
            let frame_cutout = frame.len() * (overlap_ratio - 1) / overlap_ratio;
            next_overlap = frame.split_off(frame_cutout); // e.g., ([2048], overlap_ratio=16) -> [1920, 128]
        }
        self.overlap_fragment = next_overlap;
        return frame;
    }

    /** is_empty
     * Check if the buffer is shorter than the frame sign or no more data input while frame is broken
     * Returns: Empty flag
     */
    pub fn is_empty(&self) -> bool { return self.buffer.len() < FRM_SIGN.len() || self.broken_frame; }

    /** get_asfh
     * Get a reference to the ASFH struct
     * Returns: Immutable reference to the ASFH struct
     */
    pub fn get_asfh(&self) -> &ASFH { return &self.asfh; }

    /** process
     * Process the input stream and decode the FrAD frames
     * Parameters: Input stream
     * Returns: Decoded PCM, Sample rate, Critical info modification flag
     */
    pub fn process(&mut self, stream: Vec<u8>) -> DecodeResult {
        let stream_empty = stream.is_empty();
        self.buffer.extend(stream);
        let (mut ret_pcm, mut frames) = (Vec::new(), 0);

        loop {
            // If every parameter in the ASFH struct is set,
            /* 1. Decoding FrAD Frame */
            if self.asfh.all_set {
                // 1.0. If the buffer is not enough to decode the frame, break
                // 1.0.1. If the stream is empty while ASFH is set (which means broken frame), break
                if stream_empty { self.broken_frame = true; break; }
                self.broken_frame = false;
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
                    1 => fourier::profile1::digital(frad, self.asfh.bit_depth_index, self.asfh.channels, self.asfh.srate, self.asfh.fsize),
                    2 => fourier::profile2::digital(frad, self.asfh.bit_depth_index, self.asfh.channels, self.asfh.srate, self.asfh.fsize),
                    4 => fourier::profile4::digital(frad, self.asfh.bit_depth_index, self.asfh.channels, self.asfh.endian),
                    _ => fourier::profile0::digital(frad, self.asfh.bit_depth_index, self.asfh.channels, self.asfh.endian)
                };

                // 1.4. Apply overlap
                pcm = self.overlap(pcm);

                // 1.5. Append the decoded PCM and clear header
                ret_pcm.extend(pcm); frames += 1;
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
                        // 2.1.2. else, Split out the buffer to the last 3 bytes and return
                        None => {
                            self.buffer.split_front(self.buffer.len().saturating_sub(FRM_SIGN.len() - 1));
                            break;
                        }
                    }
                }
                // 2.2. If header buffer found, try parsing the header
                let header_result = self.asfh.read(&mut self.buffer);

                // 2.3. Check header parsing result
                match header_result {
                    // 2.3.1. If header is complete and not forced to flush, continue
                    Complete => {
                        // 2.3.1.1. If any critical parameter has changed, flush the overlap buffer
                        if !self.asfh.criteq(&self.info) {
                            let (srate, chnl) = (self.info.srate, self.info.channels);
                            self.info = self.asfh.clone();
                            if srate != 0 || chnl != 0 { // If the info struct is not empty
                                ret_pcm.extend(self.flush().pcm); // Flush the overlap buffer
                                return DecodeResult { pcm: ret_pcm, srate, frames, crit: true }; // Set the critical flag and break
                            }
                        }
                    },
                    // 2.3.2. If header is complete and forced to flush, flush and return
                    ForceFlush => { ret_pcm.extend(self.flush().pcm); break; },
                    // 2.3.3. If header is incomplete, return
                    Incomplete => break,
                }
            }
        }

        return DecodeResult { pcm: ret_pcm, srate: self.asfh.srate, frames, crit: false };
    }

    /** flush
     * Flush the overlap buffer
     * Returns: Overlap buffer, Sample rate, true(flushed by user)
     */
    pub fn flush(&mut self) -> DecodeResult {
        // 1. Extract the overlap buffer
        // 2. Update stream info
        // 3. Clear the overlap buffer
        // 4. Clear the ASFH struct
        // 5. Return exctacted buffer

        let ret_pcm = self.overlap_fragment.clone();
        self.overlap_fragment.clear();
        self.asfh.clear();
        return DecodeResult {
            pcm: ret_pcm,
            srate: self.asfh.srate,
            frames: 0,
            crit: true,
        };
    }
}