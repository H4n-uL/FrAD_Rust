///                    Fast Discrete Cosine Transform - II                   ///
///
/// Copyright 2024 HaמuL
/// Description: FCT-II, Forward normalised
/// Dependencies: palmfft

use core::f64::consts::PI;
use palmfft::{CfftPlan, Complex};

pub fn dct(x: Vec<f64>) -> Vec<f64> {
    let n = x.len();

    let alpha = (0..n).map(|i| Complex::new(x[i], 0.0));
    let mut beta: Vec<Complex> = alpha.clone().chain(alpha.rev()).collect();
    CfftPlan::new(2 * n).forward(&mut beta, 1.0 / (2.0 * n as f64));

    return (0..n).map(|k| beta[k].conj().dot(Complex::from_polar(1.0, -PI * k as f64 / (2.0 * n as f64)))).collect();
}

pub fn idct(y: Vec<f64>) -> Vec<f64> {
    let n = y.len();

    let alpha: Vec<Complex> = (0..n).map(|k| Complex::from_polar(y[k], -PI * k as f64 / (2.0 * n as f64))).collect();
    let mut beta: Vec<Complex> = alpha.iter().map(|&z| z.conj())
    .chain([Complex::new(0.0, 0.0)]).chain(alpha[1..].iter().rev().cloned()).collect();

    CfftPlan::new(2 * n).backward(&mut beta, 1.0);
    return beta[..n].iter().map(|c| c.r).collect();
}