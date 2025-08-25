//!                                  Decoder                                 !//
//!
//! Copyright 2024-2025 HaמuL
//! Description: FrAD decoder

use crate::{
    backend::{hanning_in_overlap, SplitFront, VecPatternFind},
    common:: {crc16_ansi, crc32, FRM_SIGN},
    fourier::{self, profiles::{COMPACT, LOSSLESS}},
    tools::  {asfh::{ASFH, ParseResult::{Complete, Incomplete, ForceFlush}}, ecc},
};

pub struct DecodeResult {
    pcm: Vec<f64>,
    channels: u16,
    srate: u32,
    frames: usize,
    crit: bool,
}

impl DecodeResult {
    pub fn new(pcm: Vec<f64>, channels: u16, srate: u32, frames: usize, crit: bool) -> Self {
        return Self { pcm, channels, srate, frames, crit };
    }

    pub fn is_empty(&self) -> bool { self.pcm.is_empty() || self.channels == 0 || self.srate == 0 }
    pub fn pcm(&self) -> Vec<f64> { self.pcm.clone() }
    pub fn channels(&self) -> u16 { self.channels }
    pub fn samples(&self) -> usize { self.pcm.len() / (self.channels as usize).max(1)}
    pub fn srate(&self) -> u32 { self.srate }
    pub fn frames(&self) -> usize { self.frames }
    pub fn crit(&self) -> bool { self.crit }
}

/// Decoder
/// Struct for FrAD decoder
pub struct Decoder {
    asfh: ASFH, info: ASFH,
    buffer: Vec<u8>,
    overlap_fragment: Vec<f64>,
    overlap_prog: usize,

    fix_error: bool,
    broken_frame: bool
}

impl Decoder {
    pub fn new(fix_error: bool) -> Self {
        return Self {
            asfh: ASFH::new(), info: ASFH::new(),
            buffer: Vec::new(),
            overlap_fragment: Vec::new(),
            overlap_prog: 0,

            fix_error,
            broken_frame: false
        };
    }

    /// overlap
    /// Apply overlap to the decoded PCM
    /// Parameters: Decoded PCM
    /// Returns: PCM with overlap applied
    fn overlap(&mut self, mut frame: Vec<f64>) -> Vec<f64> {
        let channels = (self.asfh.channels as usize).max(1);
        let overlap_len = self.overlap_fragment.len() / channels;
        // 1. If overlap buffer not empty, apply Forward linear overlap-add
        if !self.overlap_fragment.is_empty() {
            let fade_in = hanning_in_overlap(overlap_len);
            let ov_left = (overlap_len - self.overlap_prog).min(frame.len() / channels);
            for i in 0..ov_left {
                let i_ov = i + self.overlap_prog;
                for j in 0..channels {
                    frame[i * channels + j] *= fade_in[i_ov];
                    frame[i * channels + j] += self.overlap_fragment[i_ov * channels + j] * fade_in[fade_in.len() - i_ov - 1];
                }
            }
            self.overlap_prog += ov_left;
        }

        if overlap_len <= self.overlap_prog {
            // 2. if COMPACT profile and overlap is enabled, split this frame
            self.overlap_prog = 0;
            self.overlap_fragment.clear();
            if COMPACT.contains(&self.asfh.profile) && self.asfh.overlap_ratio != 0 {
                let overlap_ratio = self.asfh.overlap_ratio as usize;
                // Samples * (Overlap ratio - 1) / Overlap ratio
                // e.g., ([2048], overlap_ratio=16) -> [1920, 128]
                let frame_cutout = (frame.len() / channels) * (overlap_ratio - 1) / overlap_ratio;
                self.overlap_fragment = frame.split_off(frame_cutout * channels);
            }
        }
        return frame;
    }

    /// is_empty
    /// Check if the buffer is shorter than the frame sign or no more data input while frame is broken
    /// Returns: Empty flag
    pub fn is_empty(&self) -> bool { return self.buffer.len() < FRM_SIGN.len() || self.broken_frame; }

    /// get_asfh
    /// Get a reference to the ASFH struct
    /// Returns: Immutable reference to the ASFH struct
    pub fn get_asfh(&self) -> &ASFH { return &self.asfh; }

    /// process
    /// Process the input stream and decode the FrAD frames
    /// Parameters: Input stream
    /// Returns: Decoded PCM, Sample rate, Critical info modification flag
    pub fn process(&mut self, stream: &[u8]) -> DecodeResult {
        self.buffer.extend(stream);
        let (mut ret_pcm, mut frames) = (Vec::new(), 0);

        loop {
            // If every parameter in the ASFH struct is set,
            /* 1. Decoding FrAD Frame */
            if self.asfh.all_set {
                // 1.0. If the buffer is not enough to decode the frame, break
                // 1.0.1. If the stream is empty while ASFH is set (which means broken frame), break
                self.broken_frame = false;
                if self.buffer.len() < self.asfh.frmbytes as usize {
                    if stream.is_empty() { self.broken_frame = true; }
                    break;
                }

                // 1.1. Split out the frame data
                let mut frad = self.buffer.split_front(self.asfh.frmbytes as usize);

                // 1.2. Correct the error if ECC is enabled
                if self.asfh.ecc {
                    let repair =  self.fix_error && ( // and if the user requested
                        // and if CRC mismatch
                        LOSSLESS.contains(&self.asfh.profile) && crc32(0, &frad) != self.asfh.crc32 ||
                        COMPACT.contains(&self.asfh.profile) && crc16_ansi(0, &frad) != self.asfh.crc16
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
                                return DecodeResult::new(ret_pcm, chnl, srate, frames, true);
                                // Set the critical flag and break
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

        return DecodeResult::new(ret_pcm, self.asfh.channels, self.asfh.srate, frames, false);
    }

    /// flush
    /// Flush the overlap buffer
    /// Returns: Overlap buffer, Sample rate, true(flushed by user)
    pub fn flush(&mut self) -> DecodeResult {
        // 1. Extract the overlap buffer
        // 2. Update stream info
        // 3. Clear the overlap buffer
        // 4. Clear the ASFH struct
        // 5. Return exctacted buffer

        let ret_pcm = self.overlap_fragment.clone();
        self.overlap_fragment.clear();
        self.asfh.clear();
        return DecodeResult::new(ret_pcm, self.asfh.channels, self.asfh.srate, 0, true);
    }
}