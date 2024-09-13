/**                              FrAD Profile 1                               */
/**
 * Copyright 2024 HaמuL
 * Description: FrAD Profile 1 encoding and decoding core
 * Dependencies: flate2, half
 */

use crate::backend::Transpose;
use super::{
    super::backend::core::{dct, idct},
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

/** get_quant_factors
 * Gets the quantisation factors for PCM and thresholds
 * Parameters: Bit depth
 * Returns: 2.0^(bit_depth - 1) as PCM quantisation factor,
 *          sqrt(3.0)^(16 - bit_depth) as threshold quantisation factor
 */
fn get_quant_factors(bit_depth: i16) -> (f64, f64) {
    let pcm_quant = 2.0_f64.powi(bit_depth as i32 - 1);
    let thres_quant = 3.0_f64.sqrt().powi(16 - bit_depth as i32);
    return (pcm_quant, thres_quant);
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
    let (pcm_quant, thres_quant) = get_quant_factors(bit_depth);
    srate = get_valid_srate(srate);

    let pcm = pad_pcm(pcm);
    let pcm_trans: Vec<Vec<f64>> = pcm.trans().iter().map(|x| x.iter().map(|y| y * pcm_quant).collect()).collect();

    let freqs: Vec<Vec<f64>> = pcm_trans.iter().map(|x| dct(x.to_vec())).collect();
    let channels = freqs.len();

    // Subband masking and quantisation
    let mut freqs_masked: Vec<Vec<f64>> = vec![vec![0.0; freqs[0].len()]; channels];
    let mut thresholds: Vec<Vec<f64>> = vec![vec![0.0; p1tools::MOSLEN]; channels];

    for c in 0..channels {
        let freqs_map_opus = p1tools::mapping_to_opus(&freqs[c].iter().map(|x| x.abs()).collect::<Vec<f64>>(), srate);
        let thres_channel: Vec<f64> = p1tools::mask_thres_mos(&freqs_map_opus, p1tools::SPREAD_ALPHA).iter().map(|x| x * loss_level).collect();

        let div_factor = p1tools::mapping_from_opus(&thres_channel, freqs[0].len(), srate);
        let chnl_masked: Vec<f64> = freqs[c].iter().zip(div_factor).map(|(x, y)| finite(p1tools::quant(x / y))).collect();

        (freqs_masked[c], thresholds[c]) = (chnl_masked, thres_channel.iter().map(|x| finite(x * thres_quant)).collect());
    }

    let freqs_flat: Vec<i64> = freqs_masked.trans().iter().flat_map(|x| x.iter().map(|y| y.round() as i64)).collect();
    let freqs_gol: Vec<u8> = p1tools::exp_golomb_rice_encode(freqs_flat);

    let thres_flat: Vec<i64> = thresholds.trans().iter().flat_map(|x| x.iter().map(|y| y.round() as i64)).collect();
    let thres_gol: Vec<u8> = p1tools::exp_golomb_rice_encode(thres_flat);

    let frad: Vec<u8> = (thres_gol.len() as u32).to_be_bytes().to_vec().into_iter().chain(thres_gol).chain(freqs_gol).collect();

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(&frad).unwrap();
    let frad = encoder.finish().unwrap();

    return (frad, DEPTHS.iter().position(|&x| x == bit_depth).unwrap() as i16, channels as i16, srate);
}

/** digital
 * Decodes FrAD Profile 1 to PCM
 * Parameters: Encoded audio data, Bit depth index, Channel count, Sample rate, Frame size
 * Returns: f64 PCM
 */
pub fn digital(frad: Vec<u8>, bit_depth_index: i16, channels: i16, srate: u32, fsize: u32) -> Vec<Vec<f64>> {
    let (bit_depth, channels) = (DEPTHS[bit_depth_index as usize], channels as usize);
    let (pcm_quant, thres_quant) = get_quant_factors(bit_depth);

    let mut decoder = ZlibDecoder::new(&frad[..]);
    let frad = {
        let mut buf = Vec::new();
        let _ = decoder.read_to_end(&mut buf)
        .map_err(|_| { return vec![vec![0.0; channels]; fsize as usize]; });
        buf
    };

    let thres_len = u32::from_be_bytes(frad[0..4].try_into().unwrap()) as usize;

    let thres_gol = frad[4..4+thres_len].to_vec();
    let freqs_gol = frad[4+thres_len..].to_vec();

    let thres_flat: Vec<f64> = p1tools::exp_golomb_rice_decode(thres_gol).into_iter()
    .map(|x| x as f64 / thres_quant).collect();
    let freqs_flat: Vec<f64> = p1tools::exp_golomb_rice_decode(freqs_gol).iter().map(|x| *x as f64).collect();

    let thresholds: Vec<Vec<f64>> = (0..channels).map(|i| thres_flat.iter().skip(i).step_by(channels).copied().collect()).collect();
    let freqs_masked: Vec<Vec<f64>> = (0..channels).map(|i| freqs_flat.iter().skip(i).step_by(channels).copied().collect()).collect();

    let mut freqs: Vec<Vec<f64>> = vec![vec![0.0; freqs_masked[0].len()]; channels];

    for c in 0..channels {
        freqs[c] = freqs_masked[c].iter()
            .zip(p1tools::mapping_from_opus(&thresholds[c], freqs_masked[c].len(), srate))
            .map(|(x, y)| p1tools::dequant(*x) * y)
            .collect();
    }

    return freqs.iter().map(|x|
        idct(x.to_vec()).iter().map(|y| y / pcm_quant).collect()
    ).collect::<Vec<Vec<f64>>>().trans();
}