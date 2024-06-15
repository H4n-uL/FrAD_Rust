/**                      Discrete Cosine Transform - II                       */
/**
 * Copyright 2024 HaמuL
 * Function: DCT-II, Backward normalised
*/

use std::f64::consts::PI;

pub fn dct(x: Vec<f64>) -> Vec<f64> {
    let n = x.len();
    let mut y = vec![0.0; n];

    for i in 0..n {
        let mut sum = 0.0;
        for j in 0..n {
            let angle = (PI / n as f64) * (j as f64 + 0.5) * i as f64;
            sum += x[j] * angle.cos();
        }
        y[i] = sum * 2.0;
    }
    return y;
}

pub fn idct(y: Vec<f64>) -> Vec<f64> {
    let n = y.len();
    let mut x = vec![0.0; n];

    for i in 0..n {
        let mut sum = y[0] / 2.0;
        for j in 1..n {
            let angle = (PI / n as f64) * (i as f64 + 0.5) * j as f64;
            sum += y[j] * angle.cos();
        }
        x[i] = sum / n as f64;
    }
    return x;
}