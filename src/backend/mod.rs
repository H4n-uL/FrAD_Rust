/**                              Library Backend                              */
/**
 * Copyright 2024 HaמuL
 * Function: Backend for FrAD Library
 */

pub mod bitcvt;

/** linspace
 * Generates a linear spaced vector
 * Parameters: Start value, Stop value, Number of values
 * Returns: Linear spaced vector
 */
pub fn linspace(start: f64, stop: f64, num: usize) -> Vec<f64> {
    if num == 0 { return vec![]; }
    if num == 1 { return vec![start]; }
    let step = (stop - start) / (num - 1) as f64;

    let mut result = Vec::with_capacity(num);
    for i in 0..num {
        let value = if i == num - 1 { stop }
        else { start + step * i as f64 };
        result.push(value);
    }
    return result;
}