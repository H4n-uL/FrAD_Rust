///                        FrAD Profiles configuration                       ///
///
/// Copyright 2024 HaמuL
/// Description: Configuration for each FrAD profiles and profiles group

// LOSSLESS profiles
pub const LOSSLESS: [u8; 2] = [0, 4];
// Compact profiles
pub const COMPACT: [u8; 2] = [1, 2];

// Compact profiles table
pub mod compact {
    // Sample rate table
    pub const SRATES: [u32; 12] = [96000, 88200, 64000, 48000, 44100, 32000, 24000, 22050, 16000, 12000, 11025, 8000];

    // Get valid sample rate eq or larger than given sample rate
    pub fn get_valid_srate(srate: u32) -> u32 {
        return SRATES.iter().rev().find(|&&x| x >= srate).unwrap_or(SRATES.iter().max().unwrap()).to_owned();
    }

    // Get sample rate index of given sample rate
    pub fn get_srate_index(srate: u32) -> u16 {
        return SRATES.iter().enumerate()
            .filter(|&(_, &x)| x >= srate)
            .min_by_key(|&(_, &x)| x)
            .map(|(index, _)| index).unwrap_or(0) as u16;
    }

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
    pub const SAMPLES_LI: [u32; 24] = [
          128,   144,   192,
          256,   288,   384,
          512,   576,   768,
         1024,  1152,  1536,
         2048,  2304,  3072,
         4096,  4608,  6144,
         8192,  9216, 12288,
        16384, 18432, 24576
    ];

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