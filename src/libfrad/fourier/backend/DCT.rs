///                      Discrete Cosine Transform - II                      ///
///
/// Copyright 2024 HaמuL
/// Description: DCT-II, Forward normalised

use core::f64::consts::PI;

pub fn dct(x: &[f64]) -> Vec<f64> {
    return (0..x.len()).map(|i| {
        (0..x.len()).map(|j| x[j] * ((PI / x.len() as f64) * (j as f64 + 0.5) * i as f64).cos()).sum::<f64>() / x.len() as f64
    }).collect();
}

pub fn idct(y: &[f64]) -> Vec<f64> {
    return (0..y.len()).map(|i| {
        y[0] + (1..y.len()).map(|j| y[j] * ((PI / y.len() as f64) * (i as f64 + 0.5) * j as f64).cos() * 2.0).sum::<f64>() 
    }).collect();
}