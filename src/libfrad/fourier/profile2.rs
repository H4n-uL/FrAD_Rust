//!                              FrAD Profile 2                              !//
//!
//! Copyright 2024-2025 HaמuL
//! Description: FrAD Profile 2 encoding and decoding core
//! Dependencies: miniz_oxide

use crate::backend::{SplitFront, Transpose};
use super::{
    backend::core::{dct, idct},
    compact::get_valid_srate,
    profile1::{get_scale_factors, pad_pcm},
    tools::{p1tools, p2tools}
};

use miniz_oxide::{deflate, inflate};

// Bit depth table
pub const DEPTHS: [u16; 8] = [8, 9, 10, 11, 12, 14, 16, 0];

/// analogue
/// Encodes PCM to FrAD Profile 2
/// Parameters: f64 PCM, Bit depth, Sample rate (and channel count, same note as profile 0)
/// Returns: Encoded audio data, Encoded bit depth index, Encoded channel count
pub fn analogue(pcm: Vec<Vec<f64>>, mut bit_depth: u16, mut srate: u32) -> (Vec<u8>, u16, u16, u32) {
    if !DEPTHS.contains(&bit_depth) || bit_depth == 0 { bit_depth = 16; }
    let (pcm_scale, _) = get_scale_factors(bit_depth);
    srate = get_valid_srate(srate);

    // 1. Pad and transform PCM with scaling
    let pcm = pad_pcm(pcm);

    // 2. DCT
    let freqs: Vec<Vec<f64>> = pcm.trans().iter().map(|x| dct(x)).collect();
    let channels = freqs.len();

    // 3. TNS analysis
    let (tns_freqs, lpc) = p2tools::tns_analysis(&freqs);

    // 4. Flattening frequencies and thresholds
    let freqs_flat: Vec<i64> = tns_freqs.trans().iter().flat_map(|x| x.iter().map(|y| (y * pcm_scale) as i64)).collect();
    let lpc_flat: Vec<i64> = lpc.trans().iter().flat_map(|x| x.iter().map(|y| *y)).collect();

    // 5. Exponential Golomb-Rice encoding
    let freqs_gol: Vec<u8> = p1tools::exp_golomb_encode(&freqs_flat);
    let lpc_gol: Vec<u8> = p1tools::exp_golomb_encode(&lpc_flat);

    // 6. Connecting data
    //    [ LPC length in u32be | Thresholds | Frequencies ]
    let frad: Vec<u8> = (lpc_gol.len() as u32).to_be_bytes().to_vec().into_iter().chain(lpc_gol).chain(freqs_gol).collect();

    // 7. Zlib compression
    let frad = deflate::compress_to_vec(&frad, 10);

    return (frad, DEPTHS.iter().position(|&x| x == bit_depth).unwrap() as u16, channels as u16, srate);
}

/// digital
/// Decodes FrAD Profile 2 to PCM
/// Parameters: Encoded audio data, Bit depth index, Channel count, Sample rate, Frame size
/// Returns: f64 PCM
pub fn digital(mut frad: Vec<u8>, bit_depth_index: u16, channels: u16, _srate: u32, fsize: u32) -> Vec<Vec<f64>> {
    let (bit_depth, channels) = (DEPTHS[bit_depth_index as usize], channels as usize);
    let ((pcm_scale, _), fsize) = (get_scale_factors(bit_depth), fsize as usize);

    // 1. Zlib decompression
    frad = match inflate::decompress_to_vec(&frad) {
        Ok(x) => x,
        Err(_) => { return vec![vec![0.0; channels]; fsize]; }
    };

    // 2. Splitting LPC and frequencies
    let lpc_len = u32::from_be_bytes(frad.split_front(4).try_into().unwrap()) as usize;
    let lpc_gol = frad.split_front(lpc_len).to_vec();

    // 3. Exponential Golomb-Rice decoding
    let mut lpc_flat: Vec<i64> = p1tools::exp_golomb_decode(&lpc_gol);
    let mut freqs_flat: Vec<f64> = p1tools::exp_golomb_decode(&frad).into_iter().map(|x| x as f64 / pcm_scale).collect();
    lpc_flat.resize((p2tools::TNS_MAX_ORDER + 1) * channels, 0);
    freqs_flat.resize(fsize * channels, 0.0);

    // 4. Unflattening frequencies and LPC
    let lpc: Vec<Vec<i64>> = (0..channels).map(|i| lpc_flat.iter().skip(i).step_by(channels).copied().collect()).collect();
    let tns_freqs: Vec<Vec<f64>> = (0..channels).map(|i| freqs_flat.iter().skip(i).step_by(channels).copied().collect()).collect();

    // 5. TNS synthesis
    let freqs = p2tools::tns_synthesis(&tns_freqs, &lpc);

    // 6. Inverse DCT and scaling
    return freqs.iter().map(|x| idct(x)).collect::<Vec<Vec<f64>>>().trans();
}