//!                                Bitconvert                                !//
//!
//! Copyright 2024-2025 HaמuL
//! Description: Bitstream <-> Byte array converter

/// to_bits
/// Converts byte array to bitstream
/// Parameters: Byte array
/// Returns: Bitstream
pub fn to_bits(bytes: &[u8]) -> Vec<bool> {
    return bytes.into_iter().flat_map(|byte| {
        (0..8).rev().map(move |i| (byte & (1 << i)) != 0)
    }).collect();
}

/// to_bytes
/// Converts bitstream to byte array
/// Parameters: Bitstream
/// Returns: Byte array
pub fn to_bytes(bitstream: &[bool]) -> Vec<u8> {
    return bitstream.chunks(8).map(|byte| {
        byte.iter().enumerate().fold(0u8, |acc, (i, &bit)| {
            acc | ((bit as u8) << (7 - i))
        })
    }).collect();
}