///                              FrAD Profile 1                              ///
///
/// Copyright 2024 HaמuL
/// Description: FrAD Profile 1 encoding and decoding core
/// Dependencies: miniz_oxide

use crate::backend::{SplitFront, Transpose};
use super::{
    backend::core::{dct, idct},
    compact::{get_valid_srate, SAMPLES_LI},
    tools::p1tools
};

use miniz_oxide::{deflate, inflate};

// Bit depth table
pub const DEPTHS: [u16; 8] = [8, 12, 16, 24, 32, 48, 64, 0];

/// pad_pcm
/// Pads the PCM to the nearest sample count greater than the original
/// Parameters: f64 PCM
/// Returns: Padded f64 PCM
pub fn pad_pcm(mut pcm: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let len_smpl = pcm.len();
    let chnl = pcm[0].len();
    let pad_len = *SAMPLES_LI.iter().find(|&&x| x as usize >= len_smpl).unwrap_or(&(len_smpl as u32)) as usize - len_smpl;

    pcm.extend(core::iter::repeat(vec![0.0; chnl]).take(pad_len));
    return pcm;
}

/// get_scale_factors
/// Gets the scale factors for PCM and thresholds
/// Parameters: Bit depth
/// Returns: 2.0^(bit_depth - 1) as PCM scale factor,
///          sqrt(3.0)^(16 - bit_depth) as threshold scale factor
pub fn get_scale_factors(bit_depth: u16) -> (f64, f64) {
    let pcm_scale = 2.0_f64.powi(bit_depth as i32 - 1);
    let thres_scale = 3.0_f64.sqrt().powi(16 - bit_depth as i32);
    return (pcm_scale, thres_scale);
}

/// analogue
/// Encodes PCM to FrAD Profile 1
/// Parameters: f64 PCM, Bit depth, Sample rate, Loss level (and channel count, same note as profile 0)
/// Returns: Encoded audio data, Encoded bit depth index, Encoded channel count
pub fn analogue(pcm: Vec<Vec<f64>>, mut bit_depth: u16, mut srate: u32, mut loss_level: f64) -> (Vec<u8>, u16, u16, u32) {
    if !DEPTHS.contains(&bit_depth) || bit_depth == 0 { bit_depth = 16; }
    let (pcm_scale, thres_scale) = get_scale_factors(bit_depth);
    (srate, loss_level) = (get_valid_srate(srate), loss_level.abs().max(0.125));

    // 1. Pad and transform PCM with scaling
    let pcm = pad_pcm(pcm);
    let pcm_trans: Vec<Vec<f64>> = pcm.trans().iter().map(|x| x.iter().map(|y| y * pcm_scale).collect()).collect();

    // 2. DCT
    let freqs: Vec<Vec<f64>> = pcm_trans.iter().map(|x| dct(x.to_vec())).collect();
    let channels = freqs.len();

    // 3. Subband masking and quantisation
    let (freqs_masked, thresholds): (Vec<Vec<f64>>, Vec<Vec<f64>>) = (0..channels)
    .into_iter().map(|c| {
        // 3.1. Masking threshold calculation
        let thres_channel: Vec<f64> = p1tools::mask_thres_mos(
            freqs[c].clone(), srate, bit_depth, loss_level, p1tools::SPREAD_ALPHA
        );

        // 3.2. Remapping thresholds to DCT bins
        // 3.3. Psychoacoustic masking
        let mut div_factor: Vec<f64> = p1tools::mapping_from_opus(&thres_channel, freqs[0].len(), srate);
        div_factor.iter_mut().for_each(|x| if x == &0.0 { *x = core::f64::INFINITY; });
        let chnl_masked: Vec<f64> = freqs[c].iter().zip(&div_factor).map(|(x, y)| x / y).collect();

        (chnl_masked, thres_channel)
    }).unzip();

    // 4. Quantisation and flattening
    let freqs_flat: Vec<i64> = freqs_masked.trans().iter().flat_map(|x| x.iter().map(|y| p1tools::quant(*y).round() as i64)).collect();
    let thres_flat: Vec<i64> = thresholds.trans().iter().flat_map(|x| x.iter().map(|y| (p1tools::quant(y * thres_scale)).round() as i64)).collect();

    // 5. Exponential Golomb-Rice encoding
    let freqs_gol: Vec<u8> = p1tools::exp_golomb_encode(freqs_flat);
    let thres_gol: Vec<u8> = p1tools::exp_golomb_encode(thres_flat);

    // 6. Connecting data
    //    [ Thresholds length in u32be | Thresholds | Frequencies ]
    let frad: Vec<u8> = (thres_gol.len() as u32).to_be_bytes().to_vec().into_iter().chain(thres_gol).chain(freqs_gol).collect();

    // 7. Zlib compression
    let frad = deflate::compress_to_vec_zlib(&frad, 10);

    return (frad, DEPTHS.iter().position(|&x| x == bit_depth).unwrap() as u16, channels as u16, srate);
}

/// digital
/// Decodes FrAD Profile 1 to PCM
/// Parameters: Encoded audio data, Bit depth index, Channel count, Sample rate, Frame size
/// Returns: f64 PCM
pub fn digital(mut frad: Vec<u8>, bit_depth_index: u16, channels: u16, srate: u32, fsize: u32) -> Vec<Vec<f64>> {
    let (bit_depth, channels) = (DEPTHS[bit_depth_index as usize], channels as usize);
    let ((pcm_scale, thres_scale), fsize) = (get_scale_factors(bit_depth), fsize as usize);

    // 1. Zlib decompression
    frad = match inflate::decompress_to_vec_zlib(&frad) {
        Ok(x) => x,
        Err(_) => { return vec![vec![0.0; channels]; fsize]; }
    };

    // 2. Splitting thresholds and frequencies
    let thres_len = u32::from_be_bytes(frad.split_front(4).try_into().unwrap()) as usize;
    let thres_gol = frad.split_front(thres_len).to_vec();

    // 3. Exponential Golomb-Rice decoding
    let mut freqs_flat: Vec<f64> = p1tools::exp_golomb_decode(frad).into_iter().map(|x| p1tools::dequant(x as f64)).collect();
    let mut thres_flat: Vec<f64> = p1tools::exp_golomb_decode(thres_gol).into_iter().map(|x| p1tools::dequant(x as f64) / thres_scale).collect();
    freqs_flat.resize(fsize * channels, 0.0);
    thres_flat.resize(p1tools::MOSLEN * channels, 0.0);

    // 4. Unflattening frequencies and thresholds
    let thresholds: Vec<Vec<f64>> = (0..channels).map(|i| thres_flat.iter().skip(i).step_by(channels).copied().collect()).collect();
    let freqs_masked: Vec<Vec<f64>> = (0..channels).map(|i| freqs_flat.iter().skip(i).step_by(channels).copied().collect()).collect();

    // 5. Dequantisation and inverse masking
    let freqs = (0..channels).into_iter().map(|c| {
        freqs_masked[c].iter().zip(p1tools::mapping_from_opus(&thresholds[c], fsize, srate))
        .map(|(x, y)| x * y).collect()
    }).collect::<Vec<Vec<f64>>>();

    // 6. Inverse DCT and scaling
    return freqs.iter().map(|x|
        idct(x.to_vec()).iter().map(|y| y / pcm_scale).collect()
    ).collect::<Vec<Vec<f64>>>().trans();
}