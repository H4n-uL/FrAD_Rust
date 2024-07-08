/**                              Profile 1 Tools                              */
/**
 * Copyright 2024 HaמuL
 * Function: Quantisation and Dequantisation tools for Profile 1
 */

use crate::backend::bitcvt;

const ALPHA: f64 = 0.8;
const MODIFIED_OPUS_SUBBANDS: [u32; 28] = [
    0,     200,   400,   600,   800,   1000,  1200,  1400,
    1600,  2000,  2400,  2800,  3200,  4000,  4800,  5600,
    6800,  8000,  9600,  12000, 15600, 20000, 24000, 28800,
    34400, 40800, 48000, u32::MAX
];
const MOSLEN: usize = MODIFIED_OPUS_SUBBANDS.len() - 1;

/** getbinrng
 * Gets the range of bins for a subband
 * Parameters: Length of the DCT Array, Signal sample rate, Subband index
 * Returns: Range of bins
 */
fn getbinrng(len: usize, srate: u32, i: usize) -> std::ops::Range<usize> {
    let start = (MODIFIED_OPUS_SUBBANDS[i] as f64 / (srate as f64 / 2.0) * len as f64).round() as usize;
    let end = (MODIFIED_OPUS_SUBBANDS[i + 1] as f64 / (srate as f64 / 2.0) * len as f64).round() as usize;
    return start.min(len)..end.min(len);
}

/** mask_thres_mos
 * Calculates the masking threshold for each subband
 * Parameters: DCT Array, Alpha value(constant... for now)
 * Returns: Masking threshold array
 */
fn mask_thres_mos(freqs: &[f64], alpha: f64) -> Vec<f64> {
    let mut thres = vec![0.0; MOSLEN];

    for i in 0..MOSLEN {
        if MODIFIED_OPUS_SUBBANDS[i+1] == u32::MAX { thres[i] = f64::INFINITY; break; }
        let f = (MODIFIED_OPUS_SUBBANDS[i] as f64 + MODIFIED_OPUS_SUBBANDS[i + 1] as f64) / 2.0;
        let abs = 3.64 * (f / 1000.0).powf(-0.8) - 6.5 * (-0.6 * (f / 1000.0 - 3.3).powi(2)).exp() + 1e-3 * (f / 1000.0).powi(4);
        let abs = abs.min(96.0);
        thres[i] = freqs[i].powf(alpha).max(10.0_f64.powf((abs - 96.0) / 20.0));
    }

    return thres;
}

/** mapping_to_opus
 * Maps the frequencies to the modified Opus subbands
 * Parameters: DCT Array, Sample rate
 * Returns: Power-averages of the subbands
 */
fn mapping_to_opus(freqs: &[f64], srate: u32) -> Vec<f64> {
    let mut mapped_freqs = [0.0; MOSLEN].to_vec();

    for i in 0..MOSLEN {
        let subfreqs = freqs[getbinrng(freqs.len(), srate, i)].to_vec();
        if subfreqs.len() > 0 {
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
fn mapping_from_opus(freqs: &[f64], freqs_len: usize, srate: u32) -> Vec<f64> {
    let mut mapped_freqs = vec![0.0; freqs_len];

    for i in 0..MOSLEN {
        let subfreqs = &mut mapped_freqs[getbinrng(freqs_len, srate, i)];
        for j in 0..subfreqs.len() { subfreqs[j] = freqs[i]; }
    }

    return mapped_freqs;
}

/** quant
 * Quantises the frequencies
 * Parameters: DCT Array, Number of channels, Sample rate, Quantisation level
 * Returns: Quantised frequencies and Masking thresholds
 */
pub fn quant(freqs: Vec<Vec<f64>>, channels: i16, srate: u32, level: u8) -> (Vec<Vec<i64>>, Vec<Vec<f64>>) {
    let const_factor = 1.25_f64.powi(level as i32) / 19.0 + 0.5;

    let mut pns_sgnl: Vec<Vec<i64>> = vec![vec![0; freqs[0].len()]; channels as usize];
    let mut mask: Vec<Vec<f64>> = vec![vec![0.0; MOSLEN]; channels as usize];

    for c in 0..channels as usize {
        let absfreqs = freqs[c].iter().map(|x| x.abs()).collect::<Vec<f64>>();
        let thres: Vec<f64> = mask_thres_mos(
            &mapping_to_opus(&absfreqs, srate), ALPHA
        ).iter().map(|x| x * const_factor).collect();
        mask[c] = thres.clone();
        let thres = mapping_from_opus(&thres, freqs[0].len(), srate);

        for i in 0..freqs[c].len() {
            pns_sgnl[c][i] = (freqs[c][i] / thres[i]).round() as i64;
        }
    }

    return (pns_sgnl, mask);
}

/** dequant
 * Dequantises the frequencies
 * Parameters: Quantised frequencies, Masking thresholds, Number of channels, Sample rate
 * Returns: Dequantised frequencies
 */
pub fn dequant(pns_sgnl: Vec<Vec<f64>>, mut masks: Vec<Vec<f64>>, channels: i16, srate: u32) -> Vec<Vec<f64>> {
    let mut freqs: Vec<Vec<f64>> = vec![vec![0.0; pns_sgnl[0].len()]; channels as usize];
    masks = masks.iter().map(|x| x.iter().map(|y| y.max(0.0)).collect()).collect();

    for c in 0..channels as usize {
        freqs[c] = pns_sgnl[c].iter().zip(mapping_from_opus(&masks[c], pns_sgnl[c].len(), srate)).map(|(x, y)| *x as f64 * y).collect();
    }

    return freqs;
}

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

    while data.len() > 0 {
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