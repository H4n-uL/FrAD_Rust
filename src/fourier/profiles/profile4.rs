/**                              FrAD Profile 4                               */
/**
 * Copyright 2024 HaמuL
 * Function: FrAD Profile 4 encoding and decoding core
 */

use super::super::backend::u8pack;

// Bit depth table
pub const DEPTHS: [i16; 8] = [12, 16, 24, 32, 48, 64, 0, 0];
// Dynamic ranges for preventing overflow
const FLOAT_DR: [i16; 8] = [5, 5, 8, 8, 11, 11, 15, 0];

/** analogue
 * Encodes PCM to FrAD
 * Parameters: f64 PCM, Bit depth, Little endian toggle (and channel count, same note as profile 0)
 * Returns: Encoded audio data, Encoded bit depth index, Encoded channel count
 */
pub fn analogue(pcm: Vec<Vec<f64>>, bits: i16, little_endian: bool) -> (Vec<u8>, i16, i16) {
    let channels = pcm[0].len();

    let pcm_flat = pcm.iter().flat_map(|x| x.iter()).cloned().collect::<Vec<f64>>();

    let mut bit_depth_index = DEPTHS.iter().position(|&x| x == bits).unwrap();
    while pcm_flat.iter().max_by(|x, y| x.abs().partial_cmp(&y.abs()).unwrap()).unwrap().abs()
            >= 2.0f64.powi(2.0f64.powi(FLOAT_DR[bit_depth_index] as i32 - 1) as i32) { // 2^(2^(bit_depth-1)) is the "float limit" before infinity
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
    let channels = channels as usize;

    return pcm_flat.chunks(channels).map(|chunk| chunk.to_vec()).collect();
}