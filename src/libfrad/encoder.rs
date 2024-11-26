/**                                  Encoder                                  */
/**
 * Copyright 2024 HaמuL
 * Description: FrAD encoder
 */

use crate::{
    PCMFormat, f64cvt::any_to_f64,
    backend::{Prepend, SplitFront},
    fourier::{self, profiles::{compact, COMPACT}, AVAILABLE, BIT_DEPTHS, SEGMAX},
    tools::  {asfh::ASFH, ecc},
};

use std::process::exit;
// use rand::{seq::{IteratorRandom, SliceRandom}, Rng};

pub struct EncodeResult {
    pub buf: Vec<u8>,
    pub samples: usize
}

/** Encoder
 * Struct for FrAD encoder
 */
pub struct Encoder {
    asfh: ASFH, buffer: Vec<u8>,
    bit_depth: i16, channels: i16,
    fsize: u32, srate: u32,
    overlap_fragment: Vec<Vec<f64>>,

    pcm_format: PCMFormat,
    loss_level: f64,
}

impl Encoder {
    pub fn new(profile: u8, pcm_format: PCMFormat) -> Encoder {
        if !AVAILABLE.contains(&profile) { eprintln!("Invalid profile! Available: {:?}", AVAILABLE); exit(1); }
        let mut asfh = ASFH::new();
        asfh.profile = profile;
        Encoder {
            asfh, buffer: Vec::new(),
            bit_depth: 0, channels: 0,
            fsize: 0, srate: 0,
            overlap_fragment: Vec::new(),

            pcm_format,
            loss_level: 0.5,
        }
    }

    // true dynamic info - set every frame
    pub fn get_channels(&self) -> i16 { self.channels }
    pub fn set_channels(&mut self, channels: i16) {
        if channels == 0 { eprintln!("Channel count cannot be zero"); exit(1); }
        self.channels = channels;
    }
    pub fn get_frame_size(&self) -> u32 { self.fsize }
    pub fn set_frame_size(&mut self, frame_size: u32) {
        if frame_size == 0 { eprintln!("Frame size cannot be zero"); exit(1); }
        if frame_size > SEGMAX[self.asfh.profile as usize] { eprintln!("Samples per frame cannot exceed {}", SEGMAX[self.asfh.profile as usize]); exit(1); }
        self.fsize = frame_size;
    }
    pub fn get_srate(&self) -> u32 { self.srate }
    pub fn set_srate(&mut self, mut srate: u32) {
        if srate == 0 { eprintln!("Sample rate cannot be zero"); exit(1); }
        if COMPACT.contains(&self.asfh.profile) {
            let x = compact::get_valid_srate(srate);
            if x != srate {
                eprintln!("Invalid sample rate! Valid rates for profile {}: {:?}",
                self.asfh.profile, compact::SRATES.iter().rev().filter(|&&x| x != 0).cloned().collect::<Vec<u32>>());
                eprintln!("Auto-adjusting to: {}", x);
            }
            srate = x;
        }
        self.srate = srate;
    }

    // half-dynamic info - This will be conveted to bit depth index for ASFH on encoding each frame
    pub fn get_bit_depth(&self) -> i16 { self.bit_depth }
    pub fn set_bit_depth(&mut self, bit_depth: i16) {
        if bit_depth == 0 { eprintln!("Bit depth cannot be zero"); exit(1); }
        if !BIT_DEPTHS[self.asfh.profile as usize].contains(&bit_depth) {
            eprintln!("Invalid bit depth! Valid depths for profile {}: {:?}",
            self.asfh.profile, BIT_DEPTHS[self.asfh.profile as usize].iter().filter(|&&x| x != 0).cloned().collect::<Vec<i16>>());
            exit(1);
        }
        self.bit_depth = bit_depth;
    }

    // static info - set once before encoding
    pub fn set_ecc(&mut self, ecc: bool, ecc_ratio: [u8; 2]) {
        self.asfh.ecc = ecc;
        if ecc_ratio[0] == 0 {
            eprintln!("ECC data size must not be zero");
            eprintln!("Setting ECC to default 96 24");
            self.asfh.ecc_ratio = [96, 24];
        }
        if ecc_ratio[0] as i16 + ecc_ratio[1] as i16 > 255 {
            eprintln!("ECC data size and check size must not exceed 255, given: {} and {}",
                ecc_ratio[0], ecc_ratio[1]);
            eprintln!("Setting ECC to default 96 24");
            self.asfh.ecc_ratio = [96, 24];
        }
        self.asfh.ecc_ratio = ecc_ratio;
    }
    pub fn set_little_endian(&mut self, little_endian: bool) { self.asfh.endian = little_endian; }
    // pub fn set_profile(&mut self, profile: u8) { self.asfh.profile = profile; }
    pub fn set_loss_level(&mut self, loss_level: f64) {
        self.loss_level = loss_level.abs().max(0.125);
    }
    pub fn set_overlap_ratio(&mut self, mut overlap_ratio: u16) {
        if overlap_ratio != 0 { overlap_ratio = overlap_ratio.max(2).min(256); }
        self.asfh.overlap_ratio = overlap_ratio;
    }

    /** get_asfh
     * Get a reference to the ASFH struct
     * Returns: Immutable reference to the ASFH struct
     */
    pub fn get_asfh(&self) -> &ASFH { return &self.asfh; }

    /** overlap
     * Overlaps the current frame with the overlap fragment
     * Parameters: Current frame, Overlap fragment, Overlap rate, Profile
     * Returns: Overlapped frame, Next overlap fragment
     */
    fn overlap(&mut self, mut frame: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
        // 1. If overlap fragment is not empty,
        if !self.overlap_fragment.is_empty() {
            // prepent the fragment to the frame
            frame.prepend(&self.overlap_fragment);
        }

        // 2. If overlap is enabled and profile uses overlap
        let mut next_overlap = Vec::new();
        if COMPACT.contains(&self.asfh.profile) && self.asfh.overlap_ratio > 1 {
            // Copy the last olap samples to the next overlap fragment
            let cutoff = (frame.len() * (self.asfh.overlap_ratio as usize - 1)) / self.asfh.overlap_ratio as usize;
            next_overlap = frame[cutoff..].to_vec();
        }
        self.overlap_fragment = next_overlap;
        return frame;
    }

    /** inner
     * Inner encoder loop
     * Parameters: PCM stream, Flush flag
     * Returns: Encoded audio data
     */
    fn inner(&mut self, stream: Vec<u8>, flush: bool) -> EncodeResult {
        self.buffer.extend(stream);
        let (mut ret, mut samples) = (Vec::new(), 0);

        loop {
            // let rng = &mut rand::thread_rng();
            // self.asfh.profile = *AVAILABLE.choose(rng).unwrap();
            // self.bit_depth = *BIT_DEPTHS[self.asfh.profile as usize].iter().filter(|&&x| x != 0).choose(rng).unwrap();
            // self.set_frame_size(
            //     if COMPACT.contains(&self.asfh.profile)
            //     { *compact::SAMPLES_LI.choose(rng).unwrap() }
            //     else { rng.gen_range(128..32768) }
            // );
            // self.set_loss_level(rng.gen_range(0.125..10.0));
            // let ecc_data = rng.gen_range(1..255);
            // self.set_ecc(rng.gen_bool(0.5), [ecc_data, rng.gen_range(0..(255 - ecc_data))]);
            // self.set_overlap_ratio(rng.gen_range(2..256));

            // 0. Set read length in samples
            let mut rlen = self.fsize as usize;
            if COMPACT.contains(&self.asfh.profile) {
                // Read length = smallest value in SMPLS_LI bigger than frame size and overlap fragment size
                let li_val = *compact::SAMPLES_LI.iter().filter(|&x| *x >= self.fsize as u32).min().unwrap() as usize;
                if li_val < self.overlap_fragment.len() // if overlap fragment is bigger than frame size
                { // find the smallest value in SMPLS_LI bigger than fragment and subtract fragment size
                    rlen = *compact::SAMPLES_LI.iter().filter(|&x| *x >= self.overlap_fragment.len() as u32).min().unwrap() as usize - self.overlap_fragment.len();
                }
                else { // else, just subtract fragment size
                    rlen = li_val - self.overlap_fragment.len();
                };
            }
            let bytes_per_sample = self.pcm_format.bit_depth() / 8;
            let read_bytes = rlen * self.channels as usize * bytes_per_sample;
            if self.buffer.len() < read_bytes && !flush { break; }

            // 1. Cut out the frame from the buffer
            let pcm_bytes: Vec<u8> = self.buffer.split_front(read_bytes);
            let pcm_flat: Vec<f64> = pcm_bytes.chunks(bytes_per_sample).map(|bytes| any_to_f64(bytes, &self.pcm_format)).collect();

            // Unravel flat PCM to 2D PCM array
            let mut frame: Vec<Vec<f64>> = pcm_flat.chunks(self.channels as usize).map(Vec::from).collect();
            if frame.is_empty() { ret.extend(self.asfh.force_flush()); break; } // If frame is empty, break
            samples += frame.len();

            // 2. Overlap the frame with the previous overlap fragment
            frame = self.overlap(frame);
            let fsize: u32 = frame.len() as u32;

            // 3. Encode the frame
            if !BIT_DEPTHS[self.asfh.profile as usize].contains(&self.bit_depth) { panic!("Invalid bit depth"); }
            let (mut frad, bit_depth_index, channels, srate) = match self.asfh.profile {
                1 => fourier::profile1::analogue(frame, self.bit_depth, self.srate, self.loss_level),
                4 => fourier::profile4::analogue(frame, self.bit_depth, self.srate, self.asfh.endian),
                _ => fourier::profile0::analogue(frame, self.bit_depth, self.srate, self.asfh.endian)
            };

            // 4. Create Reed-Solomon error correction code
            if self.asfh.ecc {
                frad = ecc::encode(frad, self.asfh.ecc_ratio);
            }

            // 5. Write the frame to the buffer
            (self.asfh.bit_depth_index, self.asfh.channels, self.asfh.fsize, self.asfh.srate) = (bit_depth_index, channels, fsize, srate);
            ret.extend(self.asfh.write(frad));
            if flush { ret.extend(self.asfh.force_flush()); }
        }

        return EncodeResult { buf: ret, samples };
    }

    /** process
     * Processes the input stream
     * Parameters: Input stream
     * Returns: Encoded audio data
     */
    pub fn process(&mut self, stream: Vec<u8>) -> EncodeResult {
        return self.inner(stream, false);
    }

    /** flush
     * Encodes the remaining data in the buffer and flush
     * Returns: Encoded audio data
     */
    pub fn flush(&mut self) -> EncodeResult {
        return self.inner(Vec::new(), true);
    }
}