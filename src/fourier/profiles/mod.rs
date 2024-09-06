/**                        FrAD Profiles configuration                        */
/**
 * Copyright 2024 HaמuL
 * Function: Configuration for each FrAD profiles and profiles group
 */

pub mod tools;
pub mod profile0;
pub mod profile1;
pub mod profile4;

// LOSSLESS profiles
pub const LOSSLESS: [u8; 2] = [0, 4];
// Compact profiles
pub const COMPACT: [u8; 1] = [1];

// Compact profiles table
pub mod compact {
    // Sample rate table
    pub const SRATES: [u32; 12] = [96000, 88200, 64000, 48000, 44100, 32000, 24000, 22050, 16000, 12000, 11025, 8000];
    // Sample count table
    pub const SAMPLES: [(u32, [u32; 8]); 3] = [
        (128, [   128,   256,   512,  1024,  2048,  4096,  8192, 16384]),
        (144, [   144,   288,   576,  1152,  2304,  4608,  9216, 18432]),
        (192, [   192,   384,   768,  1536,  3072,  6144, 12288, 24576]),
    ];

    // Get sample count multiplier from value
    pub fn get_samples_from_value(key: &u32) -> u32 {
        return SAMPLES.iter().find(|&(_, v)| v.iter().any(|&x| x == *key)).unwrap().0;
    }

    // Sample count list
    pub const SAMPLES_LI: [u32; 24] = samples_li();
    const fn samples_li() -> [u32; 24] {
        let mut result = [0; 24];
        let (mut index, mut s) = (0, 0);
        while s < 8 {
            let mut i = 0;
            while i < 3 {
                result[index] = SAMPLES[i].1[s];
                (index, i) = (index+1, i+1);
            } s += 1;
        } return result;
    }

    pub const MAX_SMPL: u32 = max_smpl();
    const fn max_smpl() -> u32 {
        let (mut max, mut i) = (0, 0);
        while i < SAMPLES_LI.len() {
            if SAMPLES_LI[i] > max {
                max = SAMPLES_LI[i];
            } i += 1;
        } return max;
    }
}