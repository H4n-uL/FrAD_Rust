/**                                Stream Info                                */
/**
 * Copyright 2024 HaמuL
 * Description: Stream information container
 */

use std::{collections::HashMap, time::Instant};

/** StreamInfo
 * Struct for stream information
 */
pub struct StreamInfo {
    pub start_time: Instant,
    pub total_size: u128,
    duration: HashMap<u32, u128>,
    bitrate: HashMap<u32, u128>,
}

impl StreamInfo {
    pub fn new() -> StreamInfo {
        StreamInfo {
            start_time: Instant::now(),
            duration: HashMap::new(),
            total_size: 0,
            bitrate: HashMap::new(),
        }
    }

    pub fn update(&mut self, size: &u128, samples: usize, srate: &u32) {
        self.total_size += size;
        self.duration.insert(*srate, if self.duration.contains_key(&srate) { self.duration[&srate] } else { 0 } + samples as u128);
        self.bitrate.insert(*srate, if self.bitrate.contains_key(&srate) { self.bitrate[&srate] } else { 0 } + *size as u128);
    }

    pub fn get_duration(&self) -> f64 {
        return self.duration.iter().map(|(k, v)| *v as f64 / *k as f64).sum();
    }

    pub fn get_bitrate(&self) -> f64 {
        let total_bits: f64 = self.bitrate.values().sum::<u128>() as f64 * 8.0;
        let total_duration: f64 = self.duration.iter().map(|(k, v)| *v as f64 / *k as f64).sum();
        return if total_duration > 0.0 { total_bits / total_duration } else { 0.0 };
    }

    pub fn get_speed(&self) -> f64 {
        let encoding_time = self.start_time.elapsed().as_secs_f64();
        let total_duration: f64 = self.duration.iter().map(|(k, v)| *v as f64 / *k as f64).sum();
        return if encoding_time > 0.0 { total_duration / encoding_time } else { 0.0 };
    }
}