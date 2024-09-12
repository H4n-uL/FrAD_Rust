/**                              FrAD Profile 0                               */
/**
 * Copyright 2024 HaמuL
 * Description: FrAD Profile 0 encoding and decoding core
 */

use crate::backend::Transpose;
use super::super::backend::{u8pack, core::{dct, idct}};
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
* Parameters: f64 PCM, Bit depth, Little endian toggle (and possibly channel count, but it can be extracted from the PCM shape)
* Returns: Encoded audio data, Encoded bit depth index, Encoded channel count
*/
pub fn analogue(pcm: Vec<Vec<f64>>, bit_depth: i16, little_endian: bool) -> (Vec<u8>, i16, i16) {
    let channels = pcm[0].len();
    let freqs: Vec<Vec<f64>> = pcm.trans().iter().map(|x: &Vec<f64>| dct(x.to_vec())).collect();
    let freqs_flat: Vec<f64> = freqs.trans().into_iter().flatten().collect();

    let mut bit_depth_index = DEPTHS.iter().position(|&x| x == bit_depth).unwrap();
    while freqs_flat.iter().fold(0.0, |max: f64, &x| max.max(x.abs())) >= FLOAT_DR_LIMITS[bit_depth_index] {
        // 2^(2^(bit_depth-1)) is the "float limit" before infinity
        if bit_depth_index == DEPTHS.len() || DEPTHS[bit_depth_index + 1] == 0
        { panic!("Overflow with reaching the max bit depth."); }
        bit_depth_index += 1;
    }

    let frad = u8pack::pack(freqs_flat, bit_depth, !little_endian);

    return (frad, bit_depth_index as i16, channels as i16);
}

/** digital
 * Decodes FrAD to PCM
* Parameters: Encoded audio data, Bit depth index, Channel count, Little endian toggle
* Returns: Decoded PCM
*/
pub fn digital(frad: Vec<u8>, bit_depth_index: i16, channels: i16, little_endian: bool) -> Vec<Vec<f64>> {
    let freqs_flat: Vec<f64> = u8pack::unpack(frad, DEPTHS[bit_depth_index as usize], !little_endian);
    let freqs: Vec<Vec<f64>> = freqs_flat.chunks(channels as usize).map(|chunk| chunk.to_vec()).collect();
    return freqs.trans().into_iter().map(idct).collect::<Vec<Vec<f64>>>().trans();
}