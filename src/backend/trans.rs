/**                                 Transpose                                 */
/**
 * Copyright 2024 HaמuL
 * Function: Transpose a 2D vector
 */

/** trans
 * Transposes a 2D vector
 * Parameters: 2D vector
 * Returns: Transposed 2D vector
 */
pub fn trans<T: Clone>(vector: Vec<Vec<T>>) -> Vec<Vec<T>> {
    return (0..vector[0].len())
    .map(|i| vector.iter().map(|inner| inner[i].clone()).collect())
    .collect();
}