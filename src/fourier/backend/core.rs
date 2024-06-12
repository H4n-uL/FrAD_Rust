/**                      Discrete Cosine Transform - II                       */
/**
 * Copyright 2024 HaמuL
 * Function: DCT-II, Backward normalised
*/

use std::f64::consts::PI;

pub fn dct(input: Vec<f64>) -> Vec<f64> {
    let slen = input.len();
    let mut output = vec![0.0; slen];

    for i in 0..slen {
        let mut sum = 0.0;
        for j in 0..slen {
            let angle = (2 * j + 1) as f64 * i as f64 * PI / (2 * slen) as f64;
            sum += input[j] * angle.cos();
        }
        output[i] = sum * 2.0;
    }
    return output;
}

pub fn idct(input: Vec<f64>) -> Vec<f64> {
    let slen = input.len();
    let mut output = vec![0.0; slen];

    for i in 0..slen {
        let mut sum = input[0] / 2.0;
        for j in 1..slen {
            let angle = j as f64 * (2 * i + 1) as f64 * PI / (2 * slen) as f64;
            sum += input[j] * angle.cos();
        }
        output[i] = sum / slen as f64;
    }
    return output;
}