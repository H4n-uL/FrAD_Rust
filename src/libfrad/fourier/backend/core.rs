/**                    Fast Discrete Cosine Transform - II                    */
/**
 * Copyright 2024 HaמuL
 * Description: FCT-II, Forward normalised
 * Dependencies: rustfft
 */

use core::f64::consts::PI;
use rustfft::{FftPlanner, num_complex::Complex};

pub fn dct(x: Vec<f64>) -> Vec<f64> {
    let n = x.len();

    let alpha = (0..n).map(|i| Complex::new(x[i] / (2.0 * n as f64), 0.0));
    let mut beta: Vec<Complex<f64>> = alpha.clone().chain(alpha.rev()).collect();
    FftPlanner::new().plan_fft_forward(2 * n).process(&mut beta);

    let y = (0..n).map(|k| {
        let angle = -PI * k as f64 / (2.0 * n as f64);
        beta[k].re * angle.cos() - beta[k].im * angle.sin()
    }).collect();
    return y;
}

pub fn idct(y: Vec<f64>) -> Vec<f64> {
    let n = y.len();

    let alpha: Vec<Complex<f64>> = (0..n).map(|k| {
        let angle = -PI * k as f64 / (2.0 * n as f64);
        Complex::new(y[k] * angle.cos(), y[k] * angle.sin())
    }).collect();

    let mut beta: Vec<Complex<f64>> = alpha.iter().map(|&z| Complex::new(z.re, -z.im))
    .chain([Complex::new(0.0, 0.0)]).chain(alpha[1..].iter().rev().cloned()).collect();

    FftPlanner::new().plan_fft_inverse(2 * n).process(&mut beta);
    let x = beta[..n].iter().map(|c| c.re).collect();
    return x;
}