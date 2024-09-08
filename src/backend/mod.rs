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
 
 /** Transpose
  * Trait for 2D vector transposition
  */
 pub trait Transpose<T> {
     fn trans(&self) -> Vec<Vec<T>> where T: Clone;
 }
 
 impl<T: Clone> Transpose<T> for Vec<Vec<T>> {
     fn trans(&self) -> Vec<Vec<T>> {
         if self.is_empty() || self[0].is_empty() { return Vec::new(); }
         return (0..self[0].len()).map(|i| self.iter().map(|inner| inner[i].clone()).collect()).collect();
     }
 }
 
 pub trait SplitOut<T> {
     fn _split_out(&mut self, n: usize) -> Vec<T> where T: Clone;
 }
 
 impl<T: Clone> SplitOut<T> for Vec<T> {
     fn _split_out(&mut self, at: usize) -> Self {
         let mut other = self.split_off(at);
         std::mem::swap(self, &mut other);
         return other;
     }
 }
 
 pub trait VecPatternFind<T: PartialEq> {
     fn _find_pattern(&self, pattern: &[T]) -> Option<usize>;
 }
 
 impl<T: PartialEq> VecPatternFind<T> for Vec<T> {
     fn _find_pattern(&self, pattern: &[T]) -> Option<usize> {
         if self.is_empty() { return Some(0); }
         return self.windows(pattern.len()).position(|window| window == pattern);
     }
 }