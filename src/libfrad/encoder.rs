//!                                  Encoder                                 !//
//!
//! Copyright 2024-2025 HaמuL
//! Description: FrAD encoder

use crate::{
    PCMFormat, f64cvt::any_to_f64,
    backend::{Prepend, SplitFront},
    fourier::{self, profiles::{compact, COMPACT}, AVAILABLE, BIT_DEPTHS, SEGMAX},
    tools::  {asfh::ASFH, ecc},
};

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
    asfh: ASFH, buffer: Vec<u8>,
    bit_depth: u16, channels: u16,
    fsize: u32, srate: u32,
    overlap_fragment: Vec<f64>,

    pcm_format: PCMFormat,
    loss_level: f64
}

pub struct EncoderParams {
    pub profile: u8,
    pub srate: u32,
    pub channels: u16,
    pub bit_depth: u16,
    pub frame_size: u32
}

impl Encoder {
    pub fn new(args: EncoderParams, pcm_format: PCMFormat) -> Result<Self, String> {
        let mut encoder = Self {
            asfh: ASFH::new(), buffer: Vec::new(),
            bit_depth: 0, channels: 0,
            fsize: 0, srate: 0,
            overlap_fragment: Vec::new(),

            pcm_format,
            loss_level: 0.5
        };
        encoder.set_profile(args)?;

        return Ok(encoder);
    }

    /// set_profile
    /// Modify the profile while running
    /// Parameters: Profile, Sample rate, Channel count, Bit depth, Frame size
    pub fn set_profile(&mut self, args: EncoderParams) -> Result<(), String> {
        if !AVAILABLE.contains(&args.profile) { return Err(format!("Invalid profile! Available: {:?}", AVAILABLE)); }


        self.asfh.profile = args.profile;
        self.set_srate(args.srate)?;
        self.set_channels(args.channels)?;
        self.set_bit_depth(args.bit_depth)?;
        self.set_frame_size(args.frame_size)?;
        return Ok(());
    }

    // Critical info - set after initialising, before processing (Global)
    pub fn get_channels(&self) -> u16 { self.channels }
    pub fn set_channels(&mut self, channels: u16) -> Result<(), String> {
        if channels == 0 { return Err("Channel count cannot be zero".to_string()); }
        self.channels = channels;
        return Ok(());
    }
    pub fn get_srate(&self) -> u32 { self.srate }
    pub fn set_srate(&mut self, srate: u32) -> Result<(), String> {
        if srate == 0 { return Err("Sample rate cannot be zero".to_string()); }
        if COMPACT.contains(&self.asfh.profile) && !compact::SRATES.contains(&srate) {
            return Err(
                format!("Invalid sample rate! Valid rates for profile {}: {:?}",
                self.asfh.profile, compact::SRATES.iter().rev().collect::<Vec<&u32>>())
            );
        }
        self.srate = srate;
        return Ok(());
    }

    // Semi-critical info - set after resetting profile
    pub fn get_frame_size(&self) -> u32 { self.fsize }
    pub fn set_frame_size(&mut self, frame_size: u32) -> Result<(), String> {
        if frame_size == 0 { return Err("Frame size cannot be zero".to_string()); }
        if frame_size > SEGMAX[self.asfh.profile as usize] {
            return Err(format!("Samples per frame cannot exceed {}", SEGMAX[self.asfh.profile as usize]));
        }
        self.fsize = frame_size;
        return Ok(());
    }
    pub fn get_bit_depth(&self) -> u16 { self.bit_depth }
    pub fn set_bit_depth(&mut self, bit_depth: u16) -> Result<(), String> {
        if bit_depth == 0 { return Err("Bit depth cannot be zero".to_string()); }
        if !BIT_DEPTHS[self.asfh.profile as usize].contains(&bit_depth) {
            return Err(
                format!("Invalid bit depth! Valid depths for profile {}: {:?}",
                self.asfh.profile, BIT_DEPTHS[self.asfh.profile as usize])
            );
        }
        self.bit_depth = bit_depth;
        return Ok(());
    }

    // Non-critical info - can be set anytime
    pub fn set_ecc(&mut self, ecc: bool, mut ecc_ratio: [u8; 2]) -> String {
        self.asfh.ecc = ecc;
        let (dsize_zero, exceed_255) = (ecc_ratio[0] == 0, ecc_ratio[0] as u16 + ecc_ratio[1] as u16 > 255);
        let mut result = String::new();
        if dsize_zero || exceed_255 {
            if dsize_zero { result = "ECC data size must not be zero".to_string(); }
            if exceed_255 {
                result = format!(
                    "ECC data size and check size must not exceed 255, given: {} and {}",
                    ecc_ratio[0], ecc_ratio[1]
                );
            }
            result.push_str("\nSetting ECC to default 96/24");
            ecc_ratio = [96, 24];
        }
        self.asfh.ecc_ratio = ecc_ratio;
        return result;
    }
    pub fn set_little_endian(&mut self, little_endian: bool) { self.asfh.endian = little_endian; }
    pub fn set_loss_level(&mut self, loss_level: f64) { self.loss_level = loss_level.abs().max(0.125); }
    pub fn set_pcm_format(&mut self, pcm_format: PCMFormat) { self.pcm_format = pcm_format; }
    pub fn set_overlap_ratio(&mut self, mut overlap_ratio: u16) {
        if overlap_ratio != 0 { overlap_ratio = overlap_ratio.max(2).min(256); }
        self.asfh.overlap_ratio = overlap_ratio;
    }

    /// get_asfh
    /// Get a reference to the ASFH struct
    /// Returns: Immutable reference to the ASFH struct
    pub fn get_asfh(&self) -> &ASFH { return &self.asfh; }

    /// overlap
    /// Overlaps the current frame with the overlap fragment
    /// Parameters: Current frame, Overlap fragment, Overlap rate, Profile
    /// Returns: Overlapped frame, Next overlap fragment
    fn overlap(&mut self, mut frame: Vec<f64>) -> Vec<f64> {
        let channels = self.channels as usize;
        // 1. If overlap fragment is not empty,
        if !self.overlap_fragment.is_empty() {
            // prepent the fragment to the frame
            frame.prepend(&self.overlap_fragment);
        }

        // 2. If overlap is enabled and profile uses overlap
        let mut next_overlap = Vec::new();
        if COMPACT.contains(&self.asfh.profile) && self.asfh.overlap_ratio > 1 {
            // Copy the last olap samples to the next overlap fragment
            let overlap_ratio = self.asfh.overlap_ratio as usize;
            // Samples * (Overlap ratio - 1) / Overlap ratio
            // e.g., ([2048], overlap_ratio=16) -> [1920, 128]
            let cutoff = (frame.len() / channels) * (overlap_ratio - 1) / overlap_ratio;
            next_overlap = frame[cutoff * channels..].to_vec();
        }
        self.overlap_fragment = next_overlap;
        return frame;
    }

    /// inner
    /// Inner encoder loop
    /// Parameters: PCM stream, Flush flag
    /// Returns: Encoded audio data
    fn inner(&mut self, stream: &[u8], flush: bool) -> EncodeResult {
        self.buffer.extend(stream);
        let (mut ret, mut samples) = (Vec::new(), 0);

        if self.srate == 0 || self.channels == 0 || self.fsize == 0 {
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
            let mut rlen = self.fsize as usize;
            if COMPACT.contains(&self.asfh.profile) {
                // Read length = smallest value in SMPLS_LI bigger than frame size and overlap fragment size
                let li_val = *compact::SAMPLES.iter().filter(|&x| *x >= self.fsize as u32).min().unwrap() as usize;
                let overlap_len = self.overlap_fragment.len() / self.channels as usize;
                if li_val <= overlap_len {
                    // if overlap fragment is equal or bigger than frame size
                    // find the smallest value in SMPLS_LI bigger than fragment and subtract fragment size
                    rlen = *compact::SAMPLES.iter().filter(|&x| *x > overlap_len as u32).min().unwrap() as usize - overlap_len;
                }
                else { // else, just subtract fragment size
                    rlen = li_val - overlap_len;
                };
            }
            let bytes_per_sample = self.pcm_format.bit_depth() / 8;
            let read_bytes = rlen * self.channels as usize * bytes_per_sample;
            if self.buffer.len() < read_bytes && !flush { break; }

            // 1. Cut out the frame from the buffer
            let pcm_bytes = self.buffer.split_front(read_bytes);
            let mut frame = pcm_bytes.chunks(bytes_per_sample)
                .map(|bytes| any_to_f64(bytes, &self.pcm_format))
                .collect::<Vec<f64>>();
            if frame.is_empty() { ret.extend(self.asfh.force_flush()); break; } // If frame is empty, break
            samples += frame.len() / self.channels as usize;

            // 2. Overlap the frame with the previous overlap fragment
            frame = self.overlap(frame);
            let fsize = (frame.len() / self.channels as usize) as u32;

            // 3. Encode the frame
            if !BIT_DEPTHS[self.asfh.profile as usize].contains(&self.bit_depth) { panic!("Invalid bit depth"); }
            let (mut frad, bit_depth_index, channels, srate) = match self.asfh.profile {
                1 => fourier::profile1::analogue(frame, self.bit_depth, self.channels, self.srate, self.loss_level),
                2 => fourier::profile2::analogue(frame, self.bit_depth, self.channels, self.srate),
                4 => fourier::profile4::analogue(frame, self.bit_depth, self.channels, self.srate, self.asfh.endian),
                _ => fourier::profile0::analogue(frame, self.bit_depth, self.channels, self.srate, self.asfh.endian)
            };

            // 4. Create Reed-Solomon error correction code
            if self.asfh.ecc { frad = ecc::encode(frad, self.asfh.ecc_ratio); }

            // 5. Write the frame to the buffer
            (self.asfh.bit_depth_index, self.asfh.channels, self.asfh.fsize, self.asfh.srate) = (bit_depth_index, channels, fsize, srate);
            // eprintln!("Encoding frame: {} samples, {} bytes, profile: {}, bit depth: {}, channels: {}, srate: {}",
                // fsize, frad.len(), self.asfh.profile, self.bit_depth, self.channels, self.srate);
            ret.extend(self.asfh.write(frad));
            if flush { ret.extend(self.asfh.force_flush()); }
        }

        return EncodeResult::new(ret, samples);
    }

    /// process
    /// Processes the input stream
    /// Parameters: Input stream
    /// Returns: Encoded audio data
    pub fn process(&mut self, stream: &[u8]) -> EncodeResult {
        return self.inner(stream, false);
    }

    /// flush
    /// Encodes the remaining data in the buffer and flush
    /// Returns: Encoded audio data
    pub fn flush(&mut self) -> EncodeResult {
        return self.inner(b"", true);
    }
}