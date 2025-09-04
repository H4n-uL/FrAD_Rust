//!                             Signal Processor                             !//
//!
//! Copyright 2024-2025 HaמuL
//! Description: Library for signal processing
//! Dependencies: palmfft

use alloc::vec::Vec;
use palmfft::{CfftPlan, Complex};

/// impulse_filt
/// Finite/Infinite Impulse Response Filter
/// Parameters: Numerator coefficients, Denominator coefficients, Input signal
/// Returns: Filtered signal
pub fn impulse_filt(b: &[f64], a: &[f64], input: &[f64]) -> Vec<f64> {
    let mut output = alloc::vec![0.0; input.len()];
    let mut x_hist = alloc::vec![0.0; b.len()];
    let mut y_hist = alloc::vec![0.0; a.len()-1];

    for (i, &x) in input.iter().enumerate() {
        for j in (1..x_hist.len()).rev() { x_hist[j] = x_hist[j-1]; }
        x_hist[0] = x;

        let mut y = b[0] * x_hist[0];
        for j in 1..b.len() { y += b[j] * x_hist[j]; }
        for j in 0..a.len()-1 { y -= a[j+1] * y_hist[j]; }
        for j in (1..y_hist.len()).rev() { y_hist[j] = y_hist[j-1]; }
        if !y_hist.is_empty() { y_hist[0] = y; }

        output[i] = y;
    }
    return output;
}

/// correlate_full
/// Full Cross-correlation of two signals
/// Parameters: Two signals
/// Returns: Full Cross-correlated signal
pub fn correlate_full(x: &[f64], y: &[f64]) -> Vec<f64> {
    let n = x.len() + y.len() - 1;
    let size = n.next_power_of_two();

    let mut x: Vec<Complex> = x.iter().map(|&x| Complex::new(x, 0.0))
        .chain(core::iter::repeat(Complex::new(0.0, 0.0))).take(size).collect();

    let mut y: Vec<Complex> = y.iter().rev().map(|&y| Complex::new(y, 0.0))
        .chain(core::iter::repeat(Complex::new(0.0, 0.0))).take(size).collect();

    let plan = CfftPlan::new(size);
    plan.forward(&mut x, 1.0).unwrap();
    plan.forward(&mut y, 1.0).unwrap();

    let mut z: Vec<Complex> = x.iter().zip(y.iter()).map(|(a, b)| *a * *b).collect();
    plan.backward(&mut z, 1.0).unwrap();
    return z.iter().take(n).map(|c| c.re / z.len() as f64).collect();
}