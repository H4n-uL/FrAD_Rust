//!                              Library Backend                             !//
//!
//! Copyright 2024-2025 HaƞuL
//! Description: Backend for FrAD Library

pub mod bitcvt;
use core::f64::consts::PI;
use alloc::vec::Vec;

/// linspace
/// Generates a linear spaced vector
/// Parameters: Start value, Stop value, Number of values, Endpoint inclusion
/// Returns: Linear spaced vector
pub fn linspace(start: f64, stop: f64, num: usize, ep: bool) -> Vec<f64> {
    if num == 0 { return alloc::vec![]; }
    if num == 1 { return alloc::vec![(start + stop) / 2.0]; }
    let step = (stop - start) / (num - ep as usize) as f64;

    let mut result = Vec::with_capacity(num);
    for i in 0..num {
        let value = start + step * i as f64;
        result.push(value);
    }
    return result;
}

/// hanning_math
/// Generates a Hanning window (Mathematically precise)
/// Parameters: Length of the window
/// Returns: Hanning window
pub fn _hanning_math(size: usize) -> Vec<f64> {
    return (0..size).map(|n| {
        0.5 * (1.0 - (2.0 * PI * n as f64 / (size - 1) as f64).cos())
    }).collect();
}

/// hanning_in_overlap
/// Generates a fade-in Hanning window (Optimised for overlap-add)
/// Parameters: Length of the window
/// Returns: Fade-in Hanning window
pub fn hanning_in_overlap(olap_len: usize) -> Vec<f64> {
    let res = (((olap_len + 1) >> 1) + 1..=olap_len).map(|i| {
        0.5 * (1.0 - (PI * i as f64 / (olap_len as f64 + 1.0)).cos())
    });
    return res.clone().rev().map(|x| 1.0 - x)
    .chain(if olap_len & 1 == 1 { Some(0.5) } else { None })
    .chain(res).collect();
}

pub trait SplitFront<T> {
    fn split_front(&mut self, n: usize) -> Vec<T>;
}

impl<T> SplitFront<T> for Vec<T> {
    fn split_front(&mut self, at: usize) -> Self {
        let mut other = self.split_off(at.min(self.len()));
        core::mem::swap(self, &mut other);
        return other;
    }
}

pub trait VecPatternFind<T: Eq> {
    fn find_pattern(&self, pattern: &[T]) -> Option<usize>;
}

impl<T: Eq> VecPatternFind<T> for Vec<T> {
    fn find_pattern(&self, pattern: &[T]) -> Option<usize> {
        if self.is_empty() || self.len() < pattern.len() { return None; }
        if pattern.is_empty() { return Some(0); }
        return self.windows(pattern.len()).position(|window| window == pattern);
    }
}

pub trait Prepend<T> {
    fn prepend(&mut self, other: &[T]) where T: Clone;
}

impl<T: Clone> Prepend<T> for Vec<T> {
    fn prepend(&mut self, other: &[T]) {
        self.splice(0..0, other.iter().cloned());
    }
}
