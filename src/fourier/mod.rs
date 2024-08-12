/**                                  Fourier                                  */
/**
 * Copyright 2024 HaמuL
 * Function: Main Fourier tools
 */

pub mod backend; pub mod profiles;
use profiles::compact;

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