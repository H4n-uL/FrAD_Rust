//!                              FrAD Profile 2                              !//
//!
//! Copyright 2024-2025 HaמuL
//! Description: FrAD Profile 2 encoding and decoding core
//! Dependencies: miniz_oxide

use crate::backend::SplitFront;
use super::{
    backend::core::{dct, idct},
    compact::get_valid_srate,
    profile1::{get_scale_factor, pad_pcm},
    tools::{p1tools, p2tools}
};

use miniz_oxide::{deflate, inflate};

// Bit depth table
pub const DEPTHS: &[u16] = &[8, 9, 10, 11, 12, 14, 16];

/// analogue
/// Encodes PCM to FrAD Profile 2
/// Parameters: f64 PCM, Bit depth, Channel count, Sample rate
/// Returns: Encoded audio data, Encoded bit depth index, Encoded channel count
pub fn analogue(pcm: Vec<f64>, mut bit_depth: u16, channels: u16, mut srate: u32) -> (Vec<u8>, u16, u16, u32) {
    if !DEPTHS.contains(&bit_depth) || bit_depth == 0 { bit_depth = 16; }
    let pcm_scale = get_scale_factor(bit_depth);
    srate = get_valid_srate(srate);

    // 1. Pad and transform PCM with scaling
    let pcm = pad_pcm(pcm, channels);

    // 2. DCT
    let mut freqs = vec![0.0; pcm.len()];
    for c in 0..channels as usize {
        let pcm_chnl = pcm.iter().skip(c).step_by(channels as usize).cloned().collect::<Vec<f64>>();
        for (i, &s) in dct(&pcm_chnl).iter().enumerate() {
            freqs[i * channels as usize + c] = s;
        }
    }

    // 3. TNS analysis
    let (tns_freqs, lpc) = p2tools::tns_analysis(&freqs, channels as usize);

    // 4. Exponential Golomb-Rice encoding
    let freqs_gol = p1tools::exp_golomb_encode(
        &tns_freqs.iter().map(|x| (x * pcm_scale) as i64).collect::<Vec<i64>>()
    );
    let lpc_gol = p1tools::exp_golomb_encode(&lpc);

    // 5. Connecting data
    //    [ LPC length in u32be | Thresholds | Frequencies ]
    let frad = (lpc_gol.len() as u32).to_be_bytes().to_vec().into_iter().chain(lpc_gol).chain(freqs_gol).collect::<Vec<u8>>();

    // 6. Deflate compression
    let frad = deflate::compress_to_vec(&frad, 10);

    return (frad, DEPTHS.iter().position(|&x| x == bit_depth).unwrap() as u16, channels as u16, srate);
}

/// digital
/// Decodes FrAD Profile 2 to PCM
/// Parameters: Encoded audio data, Bit depth index, Channel count, Sample rate, Frame size
/// Returns: f64 PCM
pub fn digital(mut frad: Vec<u8>, bit_depth_index: u16, channels: u16, _srate: u32, fsize: u32) -> Vec<f64> {
    let (bit_depth, channels) = (DEPTHS[bit_depth_index as usize], channels as usize);
    let (pcm_scale, fsize) = (get_scale_factor(bit_depth), fsize as usize);

    // 1. Deflate decompression
    frad = match inflate::decompress_to_vec(&frad) {
        Ok(x) => x,
        Err(_) => { return vec![0.0; channels * fsize]; }
    };

    // 2. Splitting LPC and frequencies
    let lpc_len = u32::from_be_bytes(frad.split_front(4).try_into().unwrap()) as usize;
    let lpc_gol = frad.split_front(lpc_len).to_vec();

    // 3. Exponential Golomb-Rice decoding
    let mut lpc = p1tools::exp_golomb_decode(&lpc_gol);
    let mut tns_freqs = p1tools::exp_golomb_decode(&frad).into_iter().map(|x|
        x as f64 / pcm_scale
    ).collect::<Vec<f64>>();

    lpc.resize((p2tools::TNS_MAX_ORDER + 1) * channels, 0);
    tns_freqs.resize(fsize * channels, 0.0);

    // 4. TNS synthesis
    let freqs = p2tools::tns_synthesis(&tns_freqs, &lpc, channels as usize);

    // 5. Inverse DCT
    let mut pcm = vec![0.0; freqs.len()];
    for c in 0..channels as usize {
        let freqs_chnl = freqs.iter().skip(c).step_by(channels as usize).cloned().collect::<Vec<f64>>();
        for (i, s) in idct(&freqs_chnl).iter().enumerate() {
            pcm[i * channels as usize + c] = *s;
        }
    }

    return pcm;
}