/**                              FrAD Profile 0                               */
/**
 * Copyright 2024 HaמuL
 * Function: FrAD Profile 0 encoding and decoding core
 */

use crate::backend::Transpose;
use super::super::backend::{u8pack, core::{dct, idct}};

// Bit depth table
pub const DEPTHS: [i16; 6] = [12, 16, 24, 32, 48, 64];
// Dynamic ranges for preventing overflow
const FLOAT_DR: [i16; 6] = [5, 5, 8, 8, 11, 11];

/** analogue
 * Encodes PCM to FrAD
* Parameters: f64 PCM, Bit depth, Little endian toggle (and possibly channel count, but it can be extracted from the PCM shape)
* Returns: Encoded audio data, Encoded bit depth index, Encoded channel count
*/
pub fn analogue(pcm: Vec<Vec<f64>>, bit_depth: i16, little_endian: bool) -> (Vec<u8>, i16, i16) {
    let freqs: Vec<Vec<f64>> = pcm.trans().iter().map(|x| dct(x.to_vec())).collect();
    let channels = freqs.len();

    let freqs_flat: Vec<f64> = (0..freqs[0].len()).flat_map(|i| freqs.iter().map(move |inner| inner[i])).collect();

    let mut bit_depth_index = DEPTHS.iter().position(|&x| x == bit_depth).unwrap();
    while freqs_flat.iter().max_by(|x, y| x.abs().partial_cmp(&y.abs()).unwrap()).unwrap().abs()
            >= 2.0f64.powi(2.0f64.powi(FLOAT_DR[bit_depth_index] as i32 - 1) as i32) { // 2^(2^(bit_depth-1)) is the "float limit" before infinity
        if bit_depth_index == DEPTHS.len() { panic!("Overflow with reaching the max bit depth."); }
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
    let channels = channels as usize;

    return freqs_flat.chunks(channels).map(|chunk| chunk.to_vec()).collect::<Vec<Vec<f64>>>().trans()
    .into_iter().map(idct).collect::<Vec<Vec<f64>>>().trans();
}