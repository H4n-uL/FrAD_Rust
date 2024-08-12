/**                              Profile 1 Tools                              */
/**
 * Copyright 2024 HaמuL
 * Function: Quantisation and Dequantisation tools for Profile 1
 */

use crate::backend::bitcvt;

pub const ALPHA: f64 = 0.8;
const QUANT_ALPHA: f64 = 0.75;
pub const MOSLEN: usize = MODIFIED_OPUS_SUBBANDS.len() - 1;
const MODIFIED_OPUS_SUBBANDS: [u32; 28] = [
    0,     200,   400,   600,   800,   1000,  1200,  1400,
    1600,  2000,  2400,  2800,  3200,  4000,  4800,  5600,
    6800,  8000,  9600,  12000, 15600, 20000, 24000, 28800,
    34400, 40800, 48000, u32::MAX
];

/** get_bin_range
 * Gets the range of bins for a subband
 * Parameters: Length of the DCT Array, Signal sample rate, Subband index
 * Returns: Range of bins
 */
fn get_bin_range(len: usize, srate: u32, i: usize) -> std::ops::Range<usize> {
    let start = (MODIFIED_OPUS_SUBBANDS[i] as f64 / (srate as f64 / 2.0) * len as f64).round() as usize;
    let end = (MODIFIED_OPUS_SUBBANDS[i + 1] as f64 / (srate as f64 / 2.0) * len as f64).round() as usize;
    return start.min(len)..end.min(len);
}

/** mask_thres_mos
 * Calculates the masking threshold for each subband
 * Parameters: DCT Array, Spread alpha(Constant for now)
 * Returns: Masking threshold array
 */
pub fn mask_thres_mos(freqs: &[f64], alpha: f64) -> Vec<f64> {
    let mut thres = vec![0.0; MOSLEN];

    for i in 0..MOSLEN {
        let f = (MODIFIED_OPUS_SUBBANDS[i] as f64 + MODIFIED_OPUS_SUBBANDS[i + 1] as f64) / 2.0;
        // Absolute Threshold of Hearing(in dB SPL)
        let abs = (3.64 * (f / 1000.0).powf(-0.8) - 6.5 * (-0.6 * (f / 1000.0 - 3.3).powi(2)).exp() + 1e-3 * (f / 1000.0).powi(4)).min(96.0);
        thres[i] = freqs[i].powf(alpha).max(10.0_f64.powf((abs - 96.0) / 20.0));
    }

    return thres;
}

/** mapping_to_opus
 * Maps the frequencies to the modified Opus subbands
 * Parameters: DCT Array, Sample rate
 * Returns: Power-averages of the subbands
 */
pub fn mapping_to_opus(freqs: &[f64], srate: u32) -> Vec<f64> {
    let mut mapped_freqs = [0.0; MOSLEN].to_vec();

    for i in 0..MOSLEN {
        let subfreqs = freqs[get_bin_range(freqs.len(), srate, i)].to_vec();
        if !subfreqs.is_empty() {
            let sfq: f64 = subfreqs.iter().map(|x| x.powi(2)).sum::<f64>() / subfreqs.len() as f64;
            mapped_freqs[i] = sfq.sqrt();
        }
    }

    return mapped_freqs;
}

/** mapping_from_opus
 * Maps the frequencies from the modified Opus subbands
 * Parameters: MOS-Mapped frequencies, Length of the DCT Array, Sample rate
 * Returns: Inverse-mapped frequencies
 */
pub fn mapping_from_opus(mapped_freqs: &[f64], freqs_len: usize, srate: u32) -> Vec<f64> {
    let mut freqs = vec![0.0; freqs_len];

    for i in 0..MOSLEN-1 {
        let start = get_bin_range(freqs_len, srate, i).start;
        let end = get_bin_range(freqs_len, srate, i + 1).start;
        let subfreqs: Vec<f64> = (start..end).map(|x| mapped_freqs[i] + (x - start) as f64 * (mapped_freqs[i + 1] - mapped_freqs[i]) / (end - start) as f64).collect();
        freqs[start..end].copy_from_slice(&subfreqs);
    }

    return freqs;
}

/** quant
 * Non-linear quantisation function
 * Parameters: f64 value to quantise
 * Returns: Quantised value
 */
pub fn quant(x: f64) -> f64 { return x.signum() * x.abs().powf(QUANT_ALPHA); }

/** dequant
 * Non-linear dequantisation function
 * Parameters: f64 value to dequantise
 * Returns: Dequantised value
 */
pub fn dequant(y: f64) -> f64 { return y.signum() * y.abs().powf(1.0 / QUANT_ALPHA); }

/** exp_golomb_rice_encode
 * Encodes any integer array with Exponential Golomb-Rice Encoding
 * Parameters: Integer array
 * Returns: Encoded binary data
 */
pub fn exp_golomb_rice_encode(data: Vec<i64>) -> Vec<u8> {
    let dmax = data.iter().map(|x| x.abs()).max().unwrap();
    let k = if dmax > 0 { (dmax as f64).log2().ceil() as u8 } else { 0 };

    let mut encoded_binary: Vec<bool> = Vec::new();

    for n in data {
        let n = if n > 0 { 2 * n - 1 } else { -2 * n };
        let x = (n + 2_i64.pow(k as u32)).to_be_bytes().to_vec();
        let mut bcode: Vec<bool> = bitcvt::frombytes(x).iter().skip_while(|&x| !x).cloned().collect();
        let m = bcode.len() - (k + 1) as usize;
        let mut un_bin = vec![false; m];
        un_bin.append(&mut bcode);
        encoded_binary.extend(un_bin);
    }
    let mut encoded = vec![k];
    encoded.extend(bitcvt::tobytes(encoded_binary));
    return encoded;
}

/** exp_golomb_rice_decode
 * Decodes any integer array with Exponential Golomb-Rice Encoding
 * Parameters: Binary data
 * Returns: Decoded integer array
 */
pub fn exp_golomb_rice_decode(data: Vec<u8>) -> Vec<i64> {
    let k = data[0];
    let mut decoded: Vec<i64> = Vec::new();
    let mut data = bitcvt::frombytes(data.iter().skip(1).cloned().collect());

    while !data.is_empty() {
        let m = data.iter().position(|&x| x).unwrap_or(data.len());
        if m == data.len() { break; }

        let codeword: Vec<bool> = data.iter().take((m * 2) + k as usize + 1).cloned().collect();
        data = data.iter().skip((m * 2) + k as usize + 1).cloned().collect();

        let mut n = i64::from_be_bytes({
            let mut x = vec![false; 64 - codeword.len()];
            x.extend(codeword);
            bitcvt::tobytes(x).try_into().unwrap()
        }) - 2_i64.pow(k as u32);
        n = if n % 2 == 1 { (n + 1) / 2 } else { -n / 2 };
        decoded.push(n);
    }

    return decoded;
}