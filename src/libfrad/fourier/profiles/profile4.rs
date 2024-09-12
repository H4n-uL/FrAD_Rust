/**                              FrAD Profile 4                               */
/**
 * Copyright 2024 HaמuL
 * Description: FrAD Profile 4 encoding and decoding core
 */

use super::super::backend::u8pack;
use half::f16;

// Bit depth table
pub const DEPTHS: [i16; 8] = [12, 16, 24, 32, 48, 64, 0, 0];
// Dynamic ranges for preventing overflow
const FLOAT_DR_LIMITS: [f64; 8] = [
    // 12, 16, 24, 32
    // 48, 64, 128, 256
    f16::MAX.to_f64_const(), f16::MAX.to_f64_const(), f32::MAX as f64, f32::MAX as f64,
    f64::MAX, f64::MAX, f64::INFINITY, f64::INFINITY
];

/** analogue
 * Encodes PCM to FrAD
 * Parameters: f64 PCM, Bit depth, Little endian toggle (and channel count, same note as profile 0)
 * Returns: Encoded audio data, Encoded bit depth index, Encoded channel count
 */
pub fn analogue(pcm: Vec<Vec<f64>>, bits: i16, little_endian: bool) -> (Vec<u8>, i16, i16) {
    let channels = pcm[0].len();
    let pcm_flat: Vec<f64> = pcm.into_iter().flatten().collect();

    let mut bit_depth_index = DEPTHS.iter().position(|&x| x == bits).unwrap();
    while pcm_flat.iter().fold(0.0, |max: f64, &x| max.max(x.abs())) >= FLOAT_DR_LIMITS[bit_depth_index] {
        // 2^(2^(bit_depth-1)) is the "float limit" before infinity
        if bit_depth_index == DEPTHS.len() { panic!("Overflow with reaching the max bit depth."); }
        bit_depth_index += 1;
    }

    let frad = u8pack::pack(pcm_flat, bits, !little_endian);
    return (frad, bit_depth_index as i16, channels as i16);
}

/** digital
 * Decodes FrAD to PCM
 * Parameters: Encoded audio data, Bit depth index, Channel count, Little endian toggle
 * Returns: Decoded PCM
 */
pub fn digital(frad: Vec<u8>, bit_depth_index: i16, channels: i16, little_endian: bool) -> Vec<Vec<f64>> {
    let pcm_flat: Vec<f64> = u8pack::unpack(frad, DEPTHS[bit_depth_index as usize], !little_endian);
    return pcm_flat.chunks(channels as usize).map(|chunk| chunk.to_vec()).collect();
}