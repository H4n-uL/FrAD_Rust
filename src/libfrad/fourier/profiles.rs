//!                        FrAD Profiles configuration                       !//
//!
//! Copyright 2024-2025 HaמuL
//! Description: Configuration for each FrAD profiles and profiles group

// LOSSLESS profiles
pub const LOSSLESS: [u8; 2] = [0, 4];
// Compact profiles
pub const COMPACT: [u8; 2] = [1, 2];

// Compact profiles table
pub mod compact {
    // Sample rate table
    pub const SRATES: &[u32] = &[96000, 88200, 64000, 48000, 44100, 32000, 24000, 22050, 16000, 12000, 11025, 8000];

    // Get valid sample rate eq or larger than given sample rate
    pub fn get_valid_srate(srate: u32) -> u32 {
        let max = SRATES.iter().max().unwrap();
        if srate > *max { return *max; }
        return *SRATES.iter().rev().find(|&&x| x >= srate).unwrap();
    }

    // Get sample rate index of given sample rate
    pub fn get_srate_index(srate: u32) -> u16 {
        return SRATES.iter().enumerate()
            .filter(|&(_, &x)| x >= srate)
            .min_by_key(|&(_, &x)| x)
            .map(|(index, _)| index).unwrap_or(0) as u16;
    }

    // Sample count list
    pub const SAMPLES: [u32; 32] = [
          128,   160,   192,   224,
          256,   320,   384,   448,
          512,   640,   768,   896,
         1024,  1280,  1536,  1792,
         2048,  2560,  3072,  3584,
         4096,  5120,  6144,  7168,
         8192, 10240, 12288, 14336,
        16384, 20480, 24576, 28672
    ];

    // Get minimum sample count greater than or equal to given value
    pub fn get_samples_min_ge(value: u32) -> u32 {
        return *SAMPLES.iter().filter(|&&x| x >= value).min().unwrap_or(&0);
    }

    // Get sample count index of given value
    pub fn get_samples_index(mut value: u32) -> u16 {
        value = get_samples_min_ge(value);
        return SAMPLES.iter().position(|&x| x == value).unwrap_or(0) as u16;
    }

    pub const MAX_SMPL: u32 = max_smpl();
    const fn max_smpl() -> u32 {
        let (mut max, mut i) = (0, 0);
        while i < SAMPLES.len() {
            if SAMPLES[i] > max {
                max = SAMPLES[i];
            } i += 1;
        } return max;
    }
}
