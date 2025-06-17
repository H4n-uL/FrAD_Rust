//!                              FrAD Profile 0                              !//
//!
//! Copyright 2024-2025 HaמuL
//! Description: FrAD Profile 0 encoding and decoding core
/// Dependencies: half

use super::backend::{u8pack, core::{dct, idct}};
use half::f16;

// Bit depth table
pub const DEPTHS: [u16; 8] = [12, 16, 24, 32, 48, 64, 0, 0];
// Dynamic ranges for preventing overflow
const FLOAT_DR_LIMITS: [f64; 8] = [
    // 12, 16, 24, 32
    // 48, 64, 128, 256
    f16::MAX.to_f64_const(), f16::MAX.to_f64_const(), f32::MAX as f64, f32::MAX as f64,
    f64::MAX, f64::MAX, f64::INFINITY, f64::INFINITY
];

/// analogue
/// Encodes PCM to FrAD
/// Parameters: f64 PCM, Bit depth, Channel count, Sample rate, Little endian toggle
/// Returns: Encoded audio data, Encoded bit depth index, Encoded channel count
pub fn analogue(pcm: Vec<f64>, mut bit_depth: u16, channels: u16, srate: u32, little_endian: bool) -> (Vec<u8>, u16, u16, u32) {
    if !DEPTHS.contains(&bit_depth) || bit_depth == 0 { bit_depth = 16; }

    let mut freqs = vec![0.0; pcm.len()];
    for c in 0..channels as usize {
        let pcm_chnl: Vec<f64> = pcm.iter().skip(c).step_by(channels as usize).cloned().collect();
        for (i, s) in dct(&pcm_chnl).iter().enumerate() {
            freqs[i * channels as usize + c] = *s;
        }
    }

    let max_abs = freqs.iter().map(|&x| x.abs()).fold(0.0f64, f64::max);

    let bit_depth_index = DEPTHS.iter().zip(FLOAT_DR_LIMITS.iter())
        .enumerate().find(|(_, (value, limit))| **value >= bit_depth && **value > 0 && max_abs < **limit)
        .map(|(i, _)| i).unwrap_or_else(|| panic!("Overflow with reaching the max bit depth."));

    let frad = u8pack::pack(freqs, DEPTHS[bit_depth_index], little_endian);

    return (frad, bit_depth_index as u16, channels, srate);
}

/// digital
/// Decodes FrAD to PCM
/// Parameters: Encoded audio data, Bit depth index, Channel count, Little endian toggle
/// Returns: Decoded PCM
pub fn digital(frad: Vec<u8>, bit_depth_index: u16, channels: u16, little_endian: bool) -> Vec<f64> {
    let freqs = u8pack::unpack(frad, DEPTHS[bit_depth_index as usize], little_endian);

    let mut pcm = vec![0.0; freqs.len()];
    for c in 0..channels as usize {
        let freqs_chnl: Vec<f64> = freqs.iter().skip(c).step_by(channels as usize).cloned().collect();
        for (i, s) in idct(&freqs_chnl).iter().enumerate() {
            pcm[i * channels as usize + c] = *s;
        }
    }

    return pcm;
}