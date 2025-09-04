//!                              Profile 2 Tools                             !//
//!
//! Copyright 2024-2025 HaמuL
//! Description: TNS analysis and synthesis tools for Profile 2

use crate::fourier::backend::signal::{correlate_full, impulse_filt};
use alloc::vec::Vec;

pub const TNS_MAX_ORDER: usize = 12;
pub const TNS_COEF_RES: usize = 4;
pub const TNS_MIN_PRED: f64 = 3.01029995663981195213738894724493027;

/// calc_autocorr
/// Calculates the auto-correlation of a frequency-domain signal
/// Parameters: Frequency-domain signal
/// Returns: Auto-correlation array of the signal
fn calc_autocorr(freq: &[f64]) -> Vec<f64> {
    let window: Vec<f64> = (0..=TNS_MAX_ORDER).map(|i| (-0.5 * (i as f64 * 0.4).powi(2)).exp()).collect();
    let corr = correlate_full(freq, freq);
    return (0..=TNS_MAX_ORDER).map(|i| corr[freq.len() - 1 + i] * window[i]).collect();
}

/// levinson_durbin
/// Calculates the LPC coefficients using the Levinson-Durbin algorithm
/// Parameters: Auto-correlation array
/// Returns: LPC coefficients
fn levinson_durbin(autocorr: &[f64]) -> Vec<f64> {
    let mut lpc = alloc::vec![0.0; TNS_MAX_ORDER + 1];
    lpc[0] = 1.0;
    let mut error = autocorr[0];

    if error <= 0.0 { return lpc; }

    for i in 1..=TNS_MAX_ORDER {
        let mut reflection = -(0..i).map(|j| lpc[j] * autocorr[i - j]).sum::<f64>();
        if error < 1e-9 { break; }

        reflection /= error;
        if reflection.abs() >= 1.0 { break; }

        lpc[i] = reflection;
        for j in 1..i {
            lpc[j] += reflection * lpc[i - j];
        }

        error *= 1.0 - reflection * reflection;
        if error <= 0.0 { break; }
    }

    return lpc;
}

/// quantise_lpc
/// Quantises the LPC coefficients to integers
/// Parameters: LPC coefficients
/// Returns: Quantised LPC coefficients
fn quantise_lpc(lpc: &[f64]) -> Vec<i64> {
    let scale = (1 << (TNS_COEF_RES - 1)) as f64 - 1.0;
    let eps = 1e-6;

    return lpc.iter().map(|&coef| {
        let scaled = (coef * scale).clamp(
            -(1 << (TNS_COEF_RES - 1)) as f64 + eps,
            (1 << (TNS_COEF_RES - 1)) as f64 - 1.0 - eps
        );
        scaled.round() as i64
    }).collect();
}

/// dequantise_lpc
/// Dequantises the LPC coefficients to floats
/// Parameters: Quantised LPC coefficients
/// Returns: LPC coefficients
fn dequantise_lpc(lpcq: &[i64]) -> Vec<f64> {
    let scale = (1 << (TNS_COEF_RES - 1)) as f64 - 1.0;
    return lpcq.iter().map(|&x| x as f64 / scale).collect();
}

/// predgain
/// Calculates the prediction gain of a signal
/// Parameters: Original signal, Predicted signal
/// Returns: Prediction gain in dB SPL
fn predgain(orig: &[f64], prc: &[f64]) -> f64 {
    let orig_energy: f64 = orig.iter().map(|x| x * x).sum();
    if orig_energy < 1e-9 { return 0.0; }

    let error: f64 = orig.iter().zip(prc.iter()).map(|(x, y)| (x - y) * (x - y)).sum();
    if error < 1e-9 { return 0.0; }

    return 20.0 * (orig_energy / error).log10();
}

/// tns_analysis
/// Performs TNS analysis on Frequency-domain signals
/// Parameters: DCT Array and Channel count
/// Returns: TNS frequencies and LPC coefficients
pub fn tns_analysis(freqs: &[f64], channels: usize) -> (Vec<f64>, Vec<i64>) {
    let mut tns_freqs = alloc::vec![0.0; freqs.len()];
    let mut lpcqs = alloc::vec![0; TNS_MAX_ORDER + 1];

    for c in 0..channels {
        let freqs_chnl = freqs.iter().skip(c).step_by(channels).cloned().collect::<Vec<_>>();
        let autocorr = calc_autocorr(&freqs_chnl);
        let lpc = levinson_durbin(&autocorr);

        if lpc.iter().any(|&x| x.abs() >= 1.0) {
            for (i, &s) in freqs_chnl.iter().enumerate() {
                tns_freqs[i * channels + c] = s;
            }
            continue;
        }

        let lpcq = quantise_lpc(&lpc);
        let lpcdeq = dequantise_lpc(&lpcq);

        let filtered = impulse_filt(&lpcdeq, &[1.0], &freqs_chnl);
        let filtered_infinite = filtered.iter().any(|x| !x.is_finite());
        let predgain_too_low = predgain(&freqs_chnl, &filtered) < TNS_MIN_PRED;
        if filtered_infinite || predgain_too_low {
            for (i, &s) in freqs_chnl.iter().enumerate() {
                tns_freqs[i * channels + c] = s;
            }
        }
        else {
            for (i, &s) in filtered.iter().enumerate() {
                tns_freqs[i * channels + c] = s;
            }

            for (i, &l) in lpcq.iter().enumerate() {
                lpcqs[i * channels + c] = l;
            }
        }
    }

    return (tns_freqs, lpcqs);
}

/// tns_synthesis
/// Performs TNS synthesis on Frequency-domain signals
/// Parameters: TNS frequencies, LPC coefficients, and Channel count
/// Returns: Synthesised DCT Array
pub fn tns_synthesis(tns_freqs: &[f64], lpcqs: &[i64], channels: usize) -> Vec<f64> {
    let mut freqs = alloc::vec![0.0; tns_freqs.len()];

    for c in 0..channels {
        let tns_freq_chnl = tns_freqs.iter().skip(c).step_by(channels).cloned().collect::<Vec<_>>();
        let lpcq_chnl = lpcqs.iter().skip(c).step_by(channels).cloned().collect::<Vec<_>>();

        let lpcdeq = dequantise_lpc(&lpcq_chnl);
        let filtered = impulse_filt(&[1.0], &lpcdeq, &tns_freq_chnl);

        let filt = if filtered.iter().any(|x| !x.is_finite()) { tns_freq_chnl } else { filtered };
        for (i, &s) in filt.iter().enumerate() {
            freqs[i * channels + c] = s;
        }
    }

    return freqs;
}