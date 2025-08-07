//!                              Profile 1 Tools                             !//
//!
//! Copyright 2024-2025 HaמuL
//! Description: Quantisation and Dequantisation tools for Profile 1

use crate::backend::{bitcvt, linspace};

pub const SPREAD_ALPHA: f64 = 0.8;
const QUANT_ALPHA: f64 = 0.75;
pub const MOSLEN: usize = MODIFIED_OPUS_SUBBANDS.len() - 1;
const MODIFIED_OPUS_SUBBANDS: [u32; 28] = [
    0,     200,   400,   600,   800,   1000,  1200,  1400,
    1600,  2000,  2400,  2800,  3200,  4000,  4800,  5600,
    6800,  8000,  9600,  12000, 15600, 20000, 24000, 28800,
    34400, 40800, 48000, u32::MAX
];

/// get_bin_range
/// Gets the range of bins for a subband
/// Parameters: Length of the DCT Array, Sample rate, Subband index
/// Returns: Range of bins
fn get_bin_range(len: usize, srate: u32, i: usize) -> core::ops::Range<usize> {
    let start = (MODIFIED_OPUS_SUBBANDS[i] as f64 / (srate as f64 / 2.0) * len as f64).round() as usize;
    let end = (MODIFIED_OPUS_SUBBANDS[i + 1] as f64 / (srate as f64 / 2.0) * len as f64).round() as usize;
    return start.min(len)..end.min(len);
}

/// mask_thres_mos
/// Calculates the masking threshold for each subband
/// Parameters: DCT Array, Sample rate, Loss level, Alpha(Constant for now)
/// Returns: Masking threshold array
pub fn mask_thres_mos(mut freqs: Vec<f64>, srate: u32, pcm_scale: f64, loss_level: f64, alpha: f64) -> Vec<f64> {
    freqs = freqs.iter().map(|x| x.abs()).collect();
    let mut thres = vec![0.0; MOSLEN];

    // for each subband
    for i in 0..MOSLEN {
        let subfreqs = freqs[get_bin_range(freqs.len(), srate, i)].to_vec();
        if subfreqs.is_empty() { continue; }
        // Centre frequency of the subband
        let f = (MODIFIED_OPUS_SUBBANDS[i] as f64 + MODIFIED_OPUS_SUBBANDS[i + 1] as f64) / 2.0;
        // Absolute Threshold of Hearing(in dB SPL)
        let absolute_hearing_threshold = 10.0f64.powf(
            (3.64 * (f / 1000.0).powf(-0.8) - 6.5 * (-0.6 * (f / 1000.0 - 3.3).powi(2)).exp() + 1e-3 * (f / 1000.0).powi(4)) / 20.0
        );
        // Root mean square
        let sfq = (subfreqs.iter().map(|x| x.powi(2)).sum::<f64>() / subfreqs.len() as f64).sqrt().powf(alpha) * pcm_scale.sqrt();
        // Larger value between mapped_freq[i]^alpha and ATH in absolute amplitude
        thres[i] = sfq.max(absolute_hearing_threshold) * loss_level;
    }

    return thres;
}

/// mapping_from_opus
/// Maps the thresholds from the modified Opus subbands
/// Parameters: MOS-Mapped thresholds, Length of the DCT Array, Sample rate
/// Returns: Inverse-mapped thresholds
pub fn mapping_from_opus(mapped_thres: &[f64], freqs_len: usize, srate: u32) -> Vec<f64> {
    let mut thres = vec![0.0; freqs_len];

    for i in 0..MOSLEN-1 {
        let range = get_bin_range(freqs_len, srate, i);
        let num = range.end - range.start;
        // Linearly spaced values between the mapped thresholds
        thres[range].copy_from_slice(&linspace(mapped_thres[i], mapped_thres[i + 1], num + 1)[..num]);
    }

    return thres;
}

/// quant
/// Non-linear quantisation function
/// Parameters: f64 value to quantise
/// Returns: Quantised value
pub fn quant(x: f64) -> f64 { return x.signum() * x.abs().powf(QUANT_ALPHA); }

/// dequant
/// Non-linear dequantisation function
/// Parameters: f64 value to dequantise
/// Returns: Dequantised value
pub fn dequant(y: f64) -> f64 { return y.signum() * y.abs().powf(1.0 / QUANT_ALPHA); }

/// exp_golomb_encode
/// Encodes any integer array with Exponential Golomb Encoding
/// Parameters: Integer array
/// Returns: Encoded binary data
pub fn exp_golomb_encode(data: &[i64]) -> Vec<u8> {
    if data.is_empty() { return vec![0]; }
    let dmax = data.iter().map(|x| x.abs()).max().unwrap();
    let k = if dmax > 0 { (dmax as f64).log2().ceil() as u8 } else { 0 };
    let (mut params, mut pos) = (Vec::with_capacity(data.len()), 8);

    for &n in data.iter() {
        let x = if n > 0 { (n << 1) - 1 } else { -n << 1 } + (1 << k);
        let bits = ((x.ilog2() as usize + 1) << 1) - (k + 1) as usize;
        params.push((pos, x, bits));
        pos += bits;
    }

    let mut encoded = vec![0u8; (pos + 7) / 8];
    encoded[0] = k;

    for (pos, x, bits) in params {
        let (byte, bit) = (pos / 8, pos % 8);
        for i in 0..bits {
            if (x & (1 << (bits - 1 - i))) != 0 {
                encoded[byte + (bit + i) / 8] |= 1 << 7 - ((bit + i) % 8);
            }
        }
    }

    return encoded;
}

/// exp_golomb_decode
/// Decodes any integer array with Exponential Golomb Encoding
/// Parameters: Binary data
/// Returns: Decoded integer array
pub fn exp_golomb_decode(data: &[u8]) -> Vec<i64> {
    let k = data[0] as usize;
    let (data, kx, mut decoded, mut idx) =
        (bitcvt::to_bits(&data[1..]), 1 << k, Vec::new(), 0);

    while idx < data.len() {
        let m = data[idx..].iter().position(|&x| x).unwrap_or(data.len());
        if m == data.len() { break; }
        let cwlen = (m * 2) + k + 1;

        let cache = &data[(idx + m)..(idx + cwlen).min(data.len())];
        let n = cache.iter().fold(0, |acc, &bit| { (acc << 1) | (bit as i64) }) - kx;
        decoded.push(if n & 1 == 1 { (n + 1) >> 1 } else { -(n >> 1) });
        idx += cwlen;
    }

    return decoded;
}