/**                    Fast Discrete Cosine Transform - II                    */
/**
 * Copyright 2024 HaמuL
 * Function: FCT-II, Backward normalised
 * Dependencies: rustfft
*/

use std::f64::consts::PI;
use rustfft::{FftPlanner, num_complex::Complex};

pub fn dct(x: Vec<f64>) -> Vec<f64> {
    let n = x.len();
    let mut beta = vec![Complex::new(0.0, 0.0); 2 * n];

    for i in 0..n {
        beta[i] = Complex::new(x[i], 0.0);
        beta[2 * n - 1 - i] = Complex::new(x[i], 0.0);
    }

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(2 * n);
    fft.process(&mut beta);

    let mut y = vec![0.0; n];
    for k in 0..n {
        let angle = -PI * k as f64 / (2.0 * n as f64);
        y[k] = beta[k].re * angle.cos() - beta[k].im * angle.sin();
    }

    return y;
}

pub fn idct(y: Vec<f64>) -> Vec<f64> {
    let n = y.len();
    let mut beta = vec![Complex::new(0.0, 0.0); 2 * n];

    for i in 0..n {
        beta[i] = Complex::new(y[i], 0.0);
        beta[2 * n - 1 - i] = Complex::new(y[i], 0.0);
    }

    let mut planner = FftPlanner::new();
    let ifft = planner.plan_fft_inverse(2 * n);
    ifft.process(&mut beta);

    let mut x = vec![0.0; n];
    for i in 0..n {
        x[i] = beta[i].re / (2.0 * n as f64);
    }

    return x;
}