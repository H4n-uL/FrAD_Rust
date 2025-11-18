//!                              Profile 2 Tools                             !//
//!
//! Copyright 2024-2025 HaƞuL
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
    let freq_mean = freq.iter().sum::<f64>() / freq.len() as f64;
    let mut sig = freq.iter().map(|&x| x - freq_mean).collect::<Vec<f64>>();
    let norm = sig.iter().sum::<f64>();
    if norm > 1e-6 {
        sig.iter_mut().for_each(|x| *x /= norm);
    }

    let full_corr = correlate_full(&sig, &sig);
    let autocorr = &full_corr[freq.len() - 1..freq.len() + TNS_MAX_ORDER];
    let window = (0..=TNS_MAX_ORDER).map(|i| (-0.5 * (i as f64 * 0.01).powi(2)).exp()).collect::<Vec<f64>>();
    return window.iter().zip(autocorr.iter()).map(|(&w, &c)| w * c).collect();
}

/// levinson_durbin
/// Calculates the LPC coefficients using the Levinson-Durbin algorithm
/// Parameters: Auto-correlation array
/// Returns: LPC coefficients
fn levinson_durbin(autocorr: &[f64]) -> Vec<f64> {
    let mut lpc = alloc::vec![0.0; TNS_MAX_ORDER + 1];
    lpc[0] = 1.0;
    let mut error = autocorr[0];
    if error <= 1e-10 { return lpc; }

    for i in 1..=TNS_MAX_ORDER {
        let mut reflection = -(0..i).map(|j| lpc[j] * autocorr[i - j]).sum::<f64>() / error;
        if reflection.abs() >= 0.96 { reflection = 0.96 * reflection.signum(); }

        let lpc_old = lpc.clone();
        lpc[i] = reflection;
        for j in 1..i {
            lpc[j] += reflection * lpc_old[i - j];
        }

        error *= 1.0 - reflection * reflection;
        if error <= 1e-12 { break; }
    }

    return lpc;
}

/// quantise_lpc
/// Quantises the LPC coefficients to integers
/// Parameters: LPC coefficients
/// Returns: Quantised LPC coefficients
fn quantise_lpc(lpc: &[f64]) -> Vec<i64> {
    let scale = (1 << (TNS_COEF_RES - 1)) as f64 - 1.0;

    let mut lpc_quant = alloc::vec![0; lpc.len()];
    if lpc.len() > 1 {
        lpc_quant[1..].iter_mut().zip(lpc[1..].iter()).for_each(|(q, &x)| {
            *q = (x * scale).clamp(-scale, scale - 1.0).round() as i64;
        });
    }

    return lpc_quant;
}

/// dequantise_lpc
/// Dequantises the LPC coefficients to floats
/// Parameters: Quantised LPC coefficients
/// Returns: LPC coefficients
fn dequantise_lpc(lpc_quant: &[i64]) -> Vec<f64> {
    if lpc_quant.iter().all(|&x| x == 0) {
        return alloc::vec![1.0];
    }
    let scale = (1 << (TNS_COEF_RES - 1)) as f64 - 1.0;
    let mut lpc_deq = alloc::vec![0.0; lpc_quant.len()];
    lpc_deq[0] = 1.0;
    if lpc_quant.len() > 1 {
        lpc_deq[1..].iter_mut().zip(lpc_quant[1..].iter()).for_each(|(d, &x)| {
            *d = x as f64 / scale;
        });
    }
    return lpc_deq;
}

/// predgain
/// Calculates the prediction gain of a signal
/// Parameters: Original signal, Residual signal
/// Returns: Prediction gain in dB SPL
fn predgain(orig: &[f64], resid: &[f64]) -> f64 {
    let orig_mean = orig.iter().sum::<f64>() / orig.len() as f64;
    let orig_centred = orig.iter().map(|&x| x - orig_mean).collect::<Vec<f64>>();
    let resid_mean = resid.iter().sum::<f64>() / resid.len() as f64;
    let resid_centred = resid.iter().map(|&x| x - resid_mean).collect::<Vec<f64>>();

    let orig_energy = orig_centred.iter().map(|&x| x * x).sum::<f64>();
    let resid_energy = resid_centred.iter().map(|&x| x * x).sum::<f64>();

    if orig_energy < 1e-10 || resid_energy < 1e-10 || resid_energy >= orig_energy {
        return 0.0;
    }

    return 20.0 * (orig_energy / resid_energy).log10();
}

/// tns_analysis
/// Performs TNS analysis on Frequency-domain signals
/// Parameters: DCT Array
/// Returns: TNS frequencies(mutated) and LPC coefficients
pub fn tns_analysis(freqs: &mut [f64]) -> Vec<i64> {
    let lpc_zero = alloc::vec![0; TNS_MAX_ORDER + 1];
    if freqs.iter().map(|x| x * x).sum::<f64>() < 1e-10 || !lpc_cond(&freqs) {
        return lpc_zero;
    }

    let autocorr = calc_autocorr(&freqs);
    let lpc = levinson_durbin(&autocorr);

    if lpc.iter().map(|x| x.abs()).sum::<f64>() < 0.01 {
        return lpc_zero;
    }

    let lpc_quant = quantise_lpc(&lpc);
    if lpc_quant.iter().all(|&x| x == 0) { return lpc_zero; }
    let lpc_deq = dequantise_lpc(&lpc_quant);

    let residual = impulse_filt(&lpc_deq, &[1.0], &freqs);
    let max = residual.iter().map(|x| x.abs()).fold(0.0, f64::max);
    if max > 1e6 || residual.iter().any(|&x| !x.is_finite()) { return lpc_zero; }
    let gain = predgain(&freqs, &residual);
    if gain < TNS_MIN_PRED { return lpc_zero; }

    for (i, &s) in residual.iter().enumerate() {
        freqs[i] = s;
    }
    return lpc_quant;
}

/// tns_synthesis
/// Performs TNS synthesis on Frequency-domain signals
/// Parameters: TNS frequencies, LPC coefficients
/// Returns: Synthesised DCT Array(mutated)
pub fn tns_synthesis(tns_freqs: &mut [f64], lpc_quant: &[i64]) {
    let mut freqs = alloc::vec![0.0; tns_freqs.len()];
    let lpc_deq = dequantise_lpc(&lpc_quant);
    let filtered = impulse_filt(&[1.0], &lpc_deq, &tns_freqs);

    let max = filtered.iter().map(|x| x.abs()).fold(0.0, f64::max);
    if max > 1e6 || filtered.iter().any(|&x| !x.is_finite()) { return; }
    for (i, &s) in filtered.iter().enumerate() {
        freqs[i] = s;
    }
}

/// lpc_cond
/// Checks if the LPC will be effective
/// Parameters: LPC coefficients
/// Returns: true if effective, false otherwise
fn lpc_cond(freqs: &[f64]) -> bool {
    let geo_mean = (freqs.iter().map(|&x| (x.abs() + 1e-10).ln()).sum::<f64>() / freqs.len() as f64).exp();
    let arith_mean = freqs.iter().map(|&x| x.abs()).sum::<f64>() / freqs.len() as f64;
    return geo_mean / (arith_mean + 1e-10) < 0.5;
}
