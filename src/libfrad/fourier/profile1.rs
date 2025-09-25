//!                              FrAD Profile 1                              !//
//!
//! Copyright 2024-2025 HaמuL
//! Description: FrAD Profile 1 encoding and decoding core
//! Dependencies: miniz_oxide

use crate::backend::SplitFront;
use super::{
    backend::core::{dct, idct},
    compact::{get_samples_min_ge, get_valid_srate},
    tools::p1tools
};

use core::{f64::consts::E, iter::repeat};
use alloc::vec::Vec;
use miniz_oxide::{deflate, inflate};

// Bit depth table
pub const DEPTHS: &[u16] = &[8, 12, 16, 24, 32, 48, 64];

/// pad_pcm
/// Pads the PCM to the nearest sample count greater than the original
/// Parameters: f64 PCM
/// Returns: Padded f64 PCM
pub fn pad_pcm(mut pcm: Vec<f64>, channels: u16) -> Vec<f64> {
    let len_smpl = pcm.len() / channels as usize;
    let pad_len = get_samples_min_ge(len_smpl as u32) as usize - len_smpl;
    pcm.extend(repeat(0.0).take(pad_len * channels as usize));
    return pcm;
}

/// get_scale_factor
/// Gets the scale factor from bit depth
/// Parameters: Bit depth
/// Returns: 2.0 ^ (bit_depth - 1)
pub fn get_scale_factor(bit_depth: u16) -> f64 {
    return 2.0_f64.powi(bit_depth as i32 - 1);
}

/// analogue
/// Encodes PCM to FrAD Profile 1
/// Parameters: f64 PCM, Bit depth, Channel count, Sample rate, Loss level
/// Returns: Encoded audio data, Encoded bit depth index, Encoded channel count
pub fn analogue(pcm: Vec<f64>, mut bit_depth: u16, channels: u16, mut srate: u32, mut loss_level: f64) -> (Vec<u8>, u16, u16, u32) {
    if !DEPTHS.contains(&bit_depth) || bit_depth == 0 { bit_depth = 16; }
    let pcm_scale = get_scale_factor(bit_depth);
    (srate, loss_level) = (get_valid_srate(srate), loss_level.abs().max(0.125));

    // 1. Pad and transform PCM with scaling
    let pcm = pad_pcm(pcm, channels);

    let mut freqs_masked = alloc::vec![0; pcm.len()];
    let mut thres = alloc::vec![0; p1tools::MOSLEN * channels as usize];
    for c in 0..channels as usize {
        // 2. DCT
        let freqs_chnl = dct(&pcm.iter().skip(c).step_by(channels as usize).cloned().collect::<Vec<f64>>());

        // 3. Subband masking and quantisation
        // 3.1. Masking threshold calculation
        let thres_chnl = p1tools::mask_thres_mos(
            freqs_chnl.iter().map(|&x| x * pcm_scale).collect::<Vec<f64>>(),
            srate, loss_level, p1tools::SPREAD_ALPHA
        );

        // 3.2. Remapping thresholds to DCT bins
        // 3.3. Psychoacoustic masking
        let mut div_factor = p1tools::mapping_from_opus(&thres_chnl, freqs_chnl.len(), srate);
        div_factor.iter_mut().for_each(|x| if *x == 0.0 { *x = core::f64::INFINITY; });
        let freqs_masked_chnl = freqs_chnl.iter().zip(&div_factor).map(|(x, y)| x / y).collect::<Vec<f64>>();

        // 4. Quantisation
        for (i, &s) in freqs_masked_chnl.iter().enumerate() {
            freqs_masked[i * channels as usize + c] = p1tools::quant(s * pcm_scale).round() as i64;
        }
        for (i, &m) in thres_chnl.iter().enumerate() {
            thres[i * channels as usize + c] = p1tools::dequant(m.max(1.0).log(E / 2.0)).round() as i64;
        }
    }

    // 5. Exponential Golomb-Rice encoding
    let freqs_gol = p1tools::exp_golomb_encode(&freqs_masked);
    let thres_gol = p1tools::exp_golomb_encode(&thres);

    // 6. Connecting data
    //    [ Thresholds length in u32be | Thresholds | Frequencies ]
    let frad = (thres_gol.len() as u32).to_be_bytes().to_vec().into_iter().chain(thres_gol).chain(freqs_gol).collect::<Vec<u8>>();

    // 7. Deflate compression
    let frad = deflate::compress_to_vec(&frad, 10);

    return (frad, DEPTHS.iter().position(|&x| x == bit_depth).unwrap() as u16, channels as u16, srate);
}

/// digital
/// Decodes FrAD Profile 1 to PCM
/// Parameters: Encoded audio data, Bit depth index, Channel count, Sample rate, Frame size
/// Returns: f64 PCM
pub fn digital(mut frad: Vec<u8>, bit_depth_index: u16, channels: u16, srate: u32, fsize: u32) -> Vec<f64> {
    let (bit_depth, channels) = (DEPTHS[bit_depth_index as usize], channels as usize);
    let (pcm_scale, fsize) = (get_scale_factor(bit_depth), fsize as usize);

    // 1. Deflate decompression
    frad = match inflate::decompress_to_vec(&frad) {
        Ok(x) => x,
        Err(_) => { return alloc::vec![0.0; channels * fsize]; }
    };

    // 2. Splitting thresholds and frequencies
    let thres_len = u32::from_be_bytes(frad.split_front(4).try_into().unwrap()) as usize;
    let thres_gol = frad.split_front(thres_len).to_vec();

    // 3. Exponential Golomb-Rice decoding
    let mut freqs_masked = p1tools::exp_golomb_decode(&frad).into_iter().map(|x|
        p1tools::dequant(x as f64) / pcm_scale
    ).collect::<Vec<f64>>();

    let mut thres = p1tools::exp_golomb_decode(&thres_gol).into_iter().map(|x|
        (E / 2.0).powf(p1tools::quant(x as f64))
    ).collect::<Vec<f64>>();

    freqs_masked.resize(fsize * channels, 0.0);
    thres.resize(p1tools::MOSLEN * channels, 0.0);

    // 4. Dequantisation and inverse masking
    let mut pcm = alloc::vec![0.0; fsize * channels];
    for c in 0..channels {
        let freqs_masked_chnl = freqs_masked.iter().skip(c).step_by(channels as usize).cloned().collect::<Vec<f64>>();
        let thres_chnl = thres.iter().skip(c).step_by(channels).cloned().collect::<Vec<f64>>();

        // 4.1. Inverse masking
        let freqs_chnl = freqs_masked_chnl.iter().zip(
            p1tools::mapping_from_opus(&thres_chnl, fsize, srate)
        ).map(|(x, y)| x * y).collect::<Vec<f64>>();

        // 4.2. Inverse DCT and scaling
        let pcm_chnl = idct(&freqs_chnl);
        for (i, &val) in pcm_chnl.iter().enumerate() {
            pcm[i * channels + c] = val;
        }
    }

    return pcm;
}
