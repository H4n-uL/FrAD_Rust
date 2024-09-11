/**                                  Fourier                                  */
/**
 * Copyright 2024 HaמuL
 * Description: Main Fourier tools
 */

pub mod backend; pub mod profiles;
use profiles::{compact, profile0, profile1, profile4};

pub const SEGMAX: [u32; 8] =
[
    u32::MAX, // Profile 0
    compact::MAX_SMPL, // Profile 1
    0, // Profile 2
    0, // Profile 3
    u32::MAX, // Profile 4
    0, // Profile 5
    0, // Profile 6
    0, // Profile 7
];

pub const BIT_DEPTHS: [[i16; 8]; 8] = [
    profile0::DEPTHS,
    profile1::DEPTHS,
    [0; 8],
    [0; 8],
    profile4::DEPTHS,
    [0; 8],
    [0; 8],
    [0; 8],
];