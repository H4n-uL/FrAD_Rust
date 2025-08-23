//!                      Fast Discrete Cosine Transform                      !//
//!
//! Copyright 2024-2025 HaמuL
//! Description: Fast Discrete Cosine Transform core
//! Dependencies: palmfft

use core::f64::consts::PI;
use palmfft::{CfftPlan, Complex};

pub fn dct2_core(x: &[f64], fct: f64) -> Vec<f64> {
    let n = x.len();
    let alpha = (0..n).map(|i| Complex::new(x[i], 0.0));
    let mut beta: Vec<Complex> = alpha.clone().chain(alpha.rev()).collect();
    CfftPlan::new(2 * n).forward(&mut beta, fct);
    return (0..n).map(|k| beta[k].conj().dot(Complex::from_polar(1.0, -PI * k as f64 / (2.0 * n as f64)))).collect();
}

pub fn dct3_core(x: &[f64], fct: f64) -> Vec<f64> {
    let n = x.len();
    let alpha = (0..n).map(|i| Complex::from_polar(x[i], -PI * i as f64 / (2.0 * n as f64)));
    let mut beta: Vec<Complex> = alpha.clone().chain(Some(Complex::zero())).chain(alpha.skip(1).rev().map(|z| z.conj())).collect();
    CfftPlan::new(2 * n).forward(&mut beta, fct);
    return (0..n).map(|k| beta[k].re).collect();
}

// pub fn dct4_core(x: &[f64], fct: f64) -> Vec<f64> {
//     let n = x.len();
//     let alpha = (0..n).map(|i| Complex::from_polar(x[i], -PI * (i as f64 + 0.5) / (2.0 * n as f64)));
//     let mut beta: Vec<Complex> = alpha.clone().chain(alpha.rev().map(|z| z.conj())).collect();
//     CfftPlan::new(2 * n).forward(&mut beta, fct);
//     return (0..n).map(|k| beta[k].conj().dot(Complex::from_polar(1.0, -PI * k as f64 / (2.0 * n as f64)))).collect();
// }

pub fn dct(x: &[f64]) -> Vec<f64> {
    return dct2_core(x, 1.0 / (2.0 * x.len() as f64));
}

pub fn idct(x: &[f64]) -> Vec<f64> {
    return dct3_core(x, 1.0);
}