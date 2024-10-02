/**                              FrAD Profile 1                               */
/**
 * Copyright 2024 HaמuL
 * Description: FrAD Profile 1 encoding and decoding core
 * Dependencies: flate2, half
 */

use crate::backend::{SplitFront, Transpose};
use super::{
    backend::core::{dct, idct},
    compact::{get_valid_srate, SAMPLES_LI},
    tools::p1tools
};

use flate2::{write::ZlibEncoder, read::ZlibDecoder, Compression};
use std::io::prelude::*;

// Bit depth table
pub const DEPTHS: [i16; 8] = [8, 12, 16, 24, 32, 48, 64, 0];

/** pad_pcm
 * Pads the PCM to the nearest sample count greater than the original
 * Parameters: f64 PCM
 * Returns: Padded f64 PCM
 */
fn pad_pcm(mut pcm: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let len_smpl = pcm.len();
    let chnl = pcm[0].len();
    let pad_len = *SAMPLES_LI.iter().find(|&&x| x as usize >= len_smpl).unwrap_or(&(len_smpl as u32)) as usize - len_smpl;

    pcm.extend(std::iter::repeat(vec![0.0; chnl]).take(pad_len));
    return pcm;
}

/** get_scale_factors
 * Gets the scale factors for PCM and thresholds
 * Parameters: Bit depth
 * Returns: 2.0^(bit_depth - 1) as PCM scale factor,
 *          sqrt(3.0)^(16 - bit_depth) as threshold scale factor
 */
fn get_scale_factors(bit_depth: i16) -> (f64, f64) {
    let pcm_scale = 2.0_f64.powi(bit_depth as i32 - 1);
    let thres_scale = 3.0_f64.sqrt().powi(16 - bit_depth as i32);
    return (pcm_scale, thres_scale);
}

fn finite(x: f64) -> f64 {
    return if x.is_finite() { x } else { 0.0 };
}

/** analogue
 * Encodes PCM to FrAD Profile 1
 * Parameters: f64 PCM, Bit depth, Sample rate, Loss level (and channel count, same note as profile 0)
 * Returns: Encoded audio data, Encoded bit depth index, Encoded channel count
 */
pub fn analogue(pcm: Vec<Vec<f64>>, bit_depth: i16, mut srate: u32, loss_level: f64) -> (Vec<u8>, i16, i16, u32) {
    let (pcm_scale, thres_scale) = get_scale_factors(bit_depth);
    srate = get_valid_srate(srate);

    // 1. Pad and transform PCM with scaling
    let pcm = pad_pcm(pcm);
    let pcm_trans: Vec<Vec<f64>> = pcm.trans().iter().map(|x| x.iter().map(|y| y * pcm_scale).collect()).collect();

    // 2. DCT
    let freqs: Vec<Vec<f64>> = pcm_trans.iter().map(|x| dct(x.to_vec())).collect();
    let channels = freqs.len();

    // 3. Subband masking and quantisation
    let mut freqs_masked: Vec<Vec<f64>> = Vec::new();
    let mut thresholds: Vec<Vec<f64>> = Vec::new();

    for c in 0..channels {
        // 3.1. Mapping frequencies to Modified Opus Subbands
        // 3.2. Masking threshold calculation
        let freqs_map_opus: Vec<f64> = p1tools::mapping_to_opus(&freqs[c].iter().map(|x| x.abs()).collect::<Vec<f64>>(), srate);
        let thres_channel: Vec<f64> = p1tools::mask_thres_mos(&freqs_map_opus, p1tools::SPREAD_ALPHA).iter().map(|x| x * loss_level).collect();

        // 3.3. Remapping thresholds to DCT bins
        // 3.4. Masking and quantisation with remapped thresholds
        let div_factor: Vec<f64> = p1tools::mapping_from_opus(&thres_channel, freqs[0].len(), srate);
        let chnl_masked: Vec<f64> = freqs[c].iter().zip(div_factor).map(|(x, y)| finite(p1tools::quant(x / y))).collect();

        // 3.5. Multiplying thresholds by threshold scale factor
        freqs_masked.push(chnl_masked);
        thresholds.push(thres_channel.iter().map(|x| finite(x * thres_scale)).collect());
    }

    // 4. Flattening frequencies and thresholds
    let freqs_flat: Vec<i64> = freqs_masked.trans().iter().flat_map(|x| x.iter().map(|y| y.round() as i64)).collect();
    let thres_flat: Vec<i64> = thresholds.trans().iter().flat_map(|x| x.iter().map(|y| y.round() as i64)).collect();

    // 5. Exponential Golomb-Rice encoding
    let freqs_gol: Vec<u8> = p1tools::exp_golomb_encode(freqs_flat);
    let thres_gol: Vec<u8> = p1tools::exp_golomb_encode(thres_flat);

    // 6. Connecting data
    //    [ Thresholds length in u32be | Thresholds | Frequencies ]
    let frad: Vec<u8> = (thres_gol.len() as u32).to_be_bytes().to_vec().into_iter().chain(thres_gol).chain(freqs_gol).collect();

    // 7. Zlib compression
    let mut compressor = ZlibEncoder::new(Vec::new(), Compression::best());
    compressor.write_all(&frad).unwrap();
    let frad = compressor.finish().unwrap();

    return (frad, DEPTHS.iter().position(|&x| x == bit_depth).unwrap() as i16, channels as i16, srate);
}

/** digital
 * Decodes FrAD Profile 1 to PCM
 * Parameters: Encoded audio data, Bit depth index, Channel count, Sample rate, Frame size
 * Returns: f64 PCM
 */
pub fn digital(frad: Vec<u8>, bit_depth_index: i16, channels: i16, srate: u32, fsize: u32) -> Vec<Vec<f64>> {
    let (bit_depth, channels) = (DEPTHS[bit_depth_index as usize], channels as usize);
    let ((pcm_scale, thres_scale), fsize) = (get_scale_factors(bit_depth), fsize as usize);

    // 1. Zlib decompression
    let mut decompressor = ZlibDecoder::new(&frad[..]);
    let mut frad = {
        let mut buf = Vec::new();
        // If decompression fails, return silence
        let _ = decompressor.read_to_end(&mut buf).map_err(|_| { return vec![vec![0.0; channels]; fsize as usize]; });
        buf
    };

    // 2. Splitting thresholds and frequencies
    let thres_len = u32::from_be_bytes(frad.split_front(4).try_into().unwrap()) as usize;
    let thres_gol = frad.split_front(thres_len).to_vec();

    // 3. Exponential Golomb-Rice decoding
    let mut thres_flat: Vec<f64> = p1tools::exp_golomb_decode(thres_gol).into_iter().map(|x| x as f64 / thres_scale).collect();
    let mut freqs_flat: Vec<f64> = p1tools::exp_golomb_decode(frad).into_iter().map(|x| x as f64).collect();
    thres_flat.resize(p1tools::MOSLEN * channels, 0.0);
    freqs_flat.resize(fsize * channels, 0.0);

    // 4. Unflattening frequencies and thresholds
    let thresholds: Vec<Vec<f64>> = (0..channels).map(|i| thres_flat.iter().skip(i).step_by(channels).copied().collect()).collect();
    let freqs_masked: Vec<Vec<f64>> = (0..channels).map(|i| freqs_flat.iter().skip(i).step_by(channels).copied().collect()).collect();

    // 5. Dequantisation and inverse masking
    let mut freqs: Vec<Vec<f64>> = Vec::new();
    for c in 0..channels {
        freqs.push(freqs_masked[c].iter()
        .zip(p1tools::mapping_from_opus(&thresholds[c], fsize, srate))
        .map(|(x, y)| p1tools::dequant(*x) * y).collect());
    }

    // 6. Inverse DCT and scaling
    return freqs.iter().map(|x|
        idct(x.to_vec()).iter().map(|y| y / pcm_scale).collect()
    ).collect::<Vec<Vec<f64>>>().trans();
}