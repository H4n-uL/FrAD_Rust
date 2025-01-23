/**                                  Fourier                                  */
/**
 * Copyright 2024 HaמuL
 * Description: Main Fourier tools
 */

pub mod profile0;
pub mod profile1;
pub mod profile2;
// pub mod profile3;
pub mod profile4;
// pub mod profile5;
// pub mod profile6;
// pub mod profile7;

pub mod backend;
pub mod profiles;
pub mod tools;

use profiles::compact;

pub const AVAILABLE: [u8; 3] = [0, 1, 4];

pub const SEGMAX: [u32; 8] =
[
    u32::MAX, // Profile 0
    compact::MAX_SMPL, // Profile 1
    compact::MAX_SMPL, // Profile 2
    0, // Profile 3
    u32::MAX, // Profile 4
    0, // Profile 5
    0, // Profile 6
    0, // Profile 7
];

pub const BIT_DEPTHS: [[u16; 8]; 8] = [
    profile0::DEPTHS,
    profile1::DEPTHS,
    profile2::DEPTHS,
    [0; 8],
    profile4::DEPTHS,
    [0; 8],
    [0; 8],
    [0; 8],
];