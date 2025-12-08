//!                                  Encoder                                 !//
//!
//! Copyright 2024-2025 HaƞuL
//! Description: FrAD encoder

use crate::{
    backend::{Prepend, SplitFront},
    fourier::{self, profiles::{compact, COMPACT}, AVAILABLE, BIT_DEPTHS, SEGMAX},
    tools::  {asfh::ASFH, ecc},
};

use alloc::{format, string::{String, ToString}, vec::Vec};

// use rand::prelude::*;

pub struct EncodeResult {
    buf: Vec<u8>,
    samples: usize
}

impl EncodeResult {
    pub fn new(buf: Vec<u8>, samples: usize) -> Self {
        return Self { buf, samples };
    }

    pub fn is_empty(&self) -> bool { self.buf.is_empty() || self.samples == 0 }
    pub fn buf(&self) -> Vec<u8> { self.buf.clone() }
    pub fn samples(&self) -> usize { self.samples }
}

/// Encoder
/// Struct for FrAD encoder
pub struct Encoder {
    asfh: ASFH, buffer: Vec<f64>,
    bit_depth: u16, channels: u16,
    fsize: u32, srate: u32,
    overlap_fragment: Vec<f64>,

    loss_level: f64,
    init: bool
}

pub struct EncoderParams {
    pub profile: u8,
    pub srate: u32,
    pub channels: u16,
    pub bit_depth: u16,
    pub frame_size: u32
}

impl Encoder {
    pub fn new(args: EncoderParams) -> Result<Self, String> {
        let mut encoder = Self {
            asfh: ASFH::new(), buffer: Vec::new(),
            bit_depth: 0, channels: 0,
            fsize: 0, srate: 0,
            overlap_fragment: Vec::new(),

            loss_level: 0.5,
            init: false
        };
        encoder.set_profile(args)?;
        return Ok(encoder);
    }

    /// overlap
    /// Overlaps the current frame with the overlap fragment from the previous frame
    /// Parameters: Current frame, Overlap read length, Flush flag
    /// Returns: Overlapped frame
    fn overlap(&mut self, mut frame: Vec<f64>, overlap_read: usize, flush: bool) -> Vec<f64> {
        let channels = self.channels as usize;
        // 1. If overlap fragment is not empty,
        if !self.overlap_fragment.is_empty() {
            // prepent the fragment to the frame
            frame.prepend(&self.overlap_fragment.split_front(overlap_read));
        }

        // 2. If overlap is enabled and profile uses overlap
        let next_flag = {
            !flush &&
            COMPACT.contains(&self.asfh.profile) &&
            self.asfh.overlap_ratio > 1 &&
            self.overlap_fragment.is_empty()
        };

        if next_flag {
            // Copy the last olap samples to the next overlap fragment
            let overlap_ratio = self.asfh.overlap_ratio as usize;
            // Samples * (Overlap ratio - 1) / Overlap ratio
            // e.g., ([2048], overlap_ratio=16) -> [1920, 128]
            let cutoff = (frame.len() / channels) * (overlap_ratio - 1) / overlap_ratio;
            self.overlap_fragment = frame[cutoff * channels..].to_vec();
        }
        return frame;
    }

    /// inner
    /// Inner encoder loop
    /// Parameters: PCM stream, Flush flag
    /// Returns: Encoded audio data
    fn inner(&mut self, stream: &[f64], flush: bool) -> EncodeResult {
        self.buffer.extend(stream);
        let (mut ret, mut samples) = (Vec::new(), 0);

        if !self.init {
            return EncodeResult::new(ret, samples);
        }

        loop {
            // let rng = &mut rand::rng();
            // let prf = *AVAILABLE.choose(rng).unwrap();
            // let prm = EncoderParams {
            //     profile: prf, srate: self.srate,
            //     channels: self.channels, bit_depth: *BIT_DEPTHS[prf as usize].iter().filter(|&&x| x != 0).choose(rng).unwrap(),
            //     frame_size: if COMPACT.contains(&prf) { *compact::SAMPLES.choose(rng).unwrap() } else { rng.random_range(128..32768) }
            // };
            // self.set_profile(prm).unwrap();
            // self.set_loss_level(rng.random_range(0.125..10.0));
            // let ecc_data = rng.random_range(1..255);
            // self.set_ecc(rng.random_bool(0.5), [ecc_data, rng.random_range(0..(255 - ecc_data))]);
            // self.set_overlap_ratio(rng.random_range(2..256));

            // 0. Set read length in samples
            let overlap_len = self.overlap_fragment.len() / self.channels as usize;
            let mut read_len = self.fsize as usize;
            if COMPACT.contains(&self.asfh.profile) {
                read_len = compact::get_samples_min_ge(read_len as u32) as usize;
            }
            let overlap_read = overlap_len.min(read_len);
            read_len -= overlap_read;
            read_len *= self.channels as usize;
            if self.buffer.len() < read_len && !flush { break; }

            // 1. Cut out the frame from the buffer
            let mut frame = self.buffer.split_front(read_len);
            let samples_in_frame = frame.len() / self.channels as usize;

            // 2. Overlap the frame with the previous overlap fragment
            frame = self.overlap(frame, overlap_read, flush);
            if frame.is_empty() {
                // If this frame is empty, break
                ret.extend(self.asfh.force_flush());
                break;
            }
            samples += samples_in_frame;
            let fsize = (frame.len() / self.channels as usize) as u32;

            // 3. Encode the frame
            if !BIT_DEPTHS[self.asfh.profile as usize].contains(&self.bit_depth) { panic!("Invalid bit depth"); }
            let (mut frad, bit_depth_index, channels, srate) = match self.asfh.profile {
                1 => fourier::profile1::analogue(frame, self.bit_depth, self.channels, self.srate, self.loss_level),
                2 => fourier::profile2::analogue(frame, self.bit_depth, self.channels, self.srate, self.loss_level),
                4 => fourier::profile4::analogue(frame, self.bit_depth, self.channels, self.srate, self.asfh.endian),
                _ => fourier::profile0::analogue(frame, self.bit_depth, self.channels, self.srate, self.asfh.endian)
            };

            // 4. Create Reed-Solomon error correction code
            if self.asfh.ecc { frad = ecc::encode(frad, self.asfh.ecc_ratio); }

            // 5. Write the frame to the buffer
            (self.asfh.bit_depth_index, self.asfh.channels, self.asfh.fsize, self.asfh.srate) = (bit_depth_index, channels, fsize, srate);
            ret.extend(self.asfh.write(frad));
            if flush { ret.extend(self.asfh.force_flush()); }
        }

        return EncodeResult::new(ret, samples);
    }

    /// process
    /// Processes the input stream
    /// Parameters: Input stream
    /// Returns: Encoded audio data
    pub fn process(&mut self, stream: &[f64]) -> EncodeResult {
        return self.inner(stream, false);
    }

    /// flush
    /// Encodes the remaining data in the buffer and flush
    /// Returns: Encoded audio data
    pub fn flush(&mut self) -> EncodeResult {
        return self.inner(&[], true);
    }
}

// Getters and Setters
impl Encoder {
    fn verify_profile(profile: u8) -> Result<(), String> {
        if !AVAILABLE.contains(&profile) {
            return Err(format!("Invalid profile! Available: {:?}", AVAILABLE));
        }
        return Ok(());
    }

    fn verify_srate(profile: u8, srate: u32) -> Result<(), String> {
        if COMPACT.contains(&profile) && !compact::SRATES.contains(&srate) {
            return Err(format!(
                "Invalid sample rate! Valid rates for profile {}: {:?}",
                profile, compact::SRATES.iter().rev().collect::<Vec<&u32>>()
            ));
        }
        return Ok(());
    }

    fn verify_channels(_profile: u8, channels: u16) -> Result<(), String> {
        if channels == 0 { return Err("Channel count cannot be zero".to_string()); }
        return Ok(());
    }

    fn verify_bit_depth(profile: u8, bit_depth: u16) -> Result<(), String> {
        if bit_depth == 0 { return Err("Bit depth cannot be zero".to_string()); }
        if !BIT_DEPTHS[profile as usize].contains(&bit_depth) {
            return Err(format!(
                "Invalid bit depth! Valid depths for profile {}: {:?}",
                profile, BIT_DEPTHS[profile as usize]
            ));
        }
        return Ok(());
    }

    fn verify_frame_size(profile: u8, frame_size: u32) -> Result<(), String> {
        if frame_size == 0 { return Err("Frame size cannot be zero".to_string()); }
        if frame_size > SEGMAX[profile as usize] {
            return Err(format!("Samples per frame cannot exceed {}", SEGMAX[profile as usize]));
        }
        return Ok(());
    }

    // Critical info
    pub fn get_profile(&self) -> u8 { self.asfh.profile }
    pub fn set_profile(&mut self, args: EncoderParams) -> Result<EncodeResult, String> {
        Self::verify_profile(args.profile)?;
        Self::verify_srate(args.profile, args.srate)?;
        Self::verify_channels(args.profile, args.channels)?;
        Self::verify_bit_depth(args.profile, args.bit_depth)?;
        Self::verify_frame_size(args.profile, args.frame_size)?;

        let mut res = EncodeResult::new(Vec::new(), 0);
        if {
            self.channels != 0 && self.channels != args.channels
            || self.srate != 0 && self.srate != args.srate
        } {
            res = self.flush();
        }

        self.asfh.profile = args.profile;
        self.srate = args.srate;
        self.channels = args.channels;
        self.bit_depth = args.bit_depth;
        self.fsize = args.frame_size;
        self.init = true;
        return Ok(res);
    }

    pub fn get_channels(&self) -> u16 { self.channels }
    pub fn set_channels(&mut self, channels: u16) -> Result<EncodeResult, String> {
        Self::verify_channels(self.get_profile(), channels)?;
        let mut res = EncodeResult::new(Vec::new(), 0);
        if self.channels != 0 && self.channels != channels {
            res = self.flush();
        }
        self.channels = channels;
        return Ok(res);
    }
    pub fn get_srate(&self) -> u32 { self.srate }
    pub fn set_srate(&mut self, srate: u32) -> Result<EncodeResult, String> {
        Self::verify_srate(self.get_profile(), srate)?;
        let mut res = EncodeResult::new(Vec::new(), 0);
        if self.srate != 0 && self.srate != srate { res = self.flush(); }
        self.srate = srate;
        return Ok(res);
    }

    // Semi-critical info
    pub fn get_frame_size(&self) -> u32 { self.fsize }
    pub fn set_frame_size(&mut self, frame_size: u32) -> Result<(), String> {
        Self::verify_frame_size(self.get_profile(), frame_size)?;
        self.fsize = frame_size;
        return Ok(());
    }
    pub fn get_bit_depth(&self) -> u16 { self.bit_depth }
    pub fn set_bit_depth(&mut self, bit_depth: u16) -> Result<(), String> {
        Self::verify_bit_depth(self.get_profile(), bit_depth)?;
        self.bit_depth = bit_depth;
        return Ok(());
    }

    // Non-critical info
    pub fn set_ecc(&mut self, ecc: bool, mut ecc_ratio: [u8; 2]) -> String {
        self.asfh.ecc = ecc;
        let (dsize_zero, exceed_255) = (ecc_ratio[0] == 0, ecc_ratio[0] as u16 + ecc_ratio[1] as u16 > 255);
        let mut warn = String::new();
        if dsize_zero || exceed_255 {
            if dsize_zero { warn.push_str("ECC data size must not be zero"); }
            if exceed_255 {
                warn.push_str(format!(
                    "ECC data size and check size must not exceed 255, given: {} and {}",
                    ecc_ratio[0], ecc_ratio[1]
                ).as_str());
            }
            warn.push_str("\nSetting ECC to default 96/24");
            ecc_ratio = [96, 24];
        }
        self.asfh.ecc_ratio = ecc_ratio;
        return warn;
    }
    pub fn set_little_endian(&mut self, little_endian: bool) { self.asfh.endian = little_endian; }
    pub fn set_loss_level(&mut self, loss_level: f64) { self.loss_level = loss_level.abs().max(0.125); }
    pub fn set_overlap_ratio(&mut self, mut overlap_ratio: u16) {
        if overlap_ratio != 0 { overlap_ratio = overlap_ratio.max(2).min(256); }
        self.asfh.overlap_ratio = overlap_ratio;
    }
    pub fn get_asfh(&self) -> &ASFH { return &self.asfh; }
}
