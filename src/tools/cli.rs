/**                                CLI Parser                                 */
/**
 * Copyright 2024 HaמuL
 * Function: Simple CLI parser for FrAD Library
 */

use std::{collections::VecDeque, env::Args};

// CLI Options
pub const ENCODE_OPT: [&str; 2] = ["encode", "enc"];
pub const DECODE_OPT: [&str; 2] = ["decode", "dec"];
pub const REPAIR_OPT: [&str; 4] = ["reecc", "re-ecc", "repair", "ecc"];
pub const HEADER_OPT: [&str; 2] = ["meta", "metadata"];

pub const META_ADD: &str = "add";
pub const META_REMOVE: &str = "remove";
pub const META_RMIMG: &str = "rm-img";
pub const META_OVERWRITE: &str = "overwrite";

// CLI Parameters
pub struct CliParams {
    pub output: String,
    pub bits: i16,
    pub srate: u32,
    pub channels: i16,
    pub frame_size: u32,
    pub little_endian: bool,
    pub profile: u8,
    pub overlap: u8,
    pub losslevel: u8,
    pub enable_ecc: bool,
    pub ecc_rate: [u8; 2],
    pub overwrite: bool,
    pub meta: Vec<(String, Vec<u8>)>,
    pub image_path: String
}

impl CliParams {
    pub fn new() -> CliParams {
        CliParams {
            output: String::new(),
            bits: 0,
            srate: 0,
            channels: 0,
            frame_size: 2048,
            little_endian: false,
            profile: 4,
            overlap: 16,
            losslevel: 0,
            enable_ecc: false,
            ecc_rate: [96, 24],
            overwrite: false,
            meta: Vec::new(),
            image_path: String::new()
        }
    }// i rly miss lombok
    pub fn set_output(&mut self, output: String) -> () { self.output = output; }
    pub fn set_bits(&mut self, bits: String) -> () { self.bits = bits.parse().unwrap(); }
    pub fn set_srate(&mut self, srate: String) -> () { self.srate = srate.parse().unwrap(); }
    pub fn set_channels(&mut self, channels: String) -> () { self.channels = channels.parse().unwrap(); }
    pub fn set_frame_size(&mut self, frame_size: String) -> () { self.frame_size = frame_size.parse().unwrap(); }
    pub fn set_little_endian(&mut self) -> () { self.little_endian = true; }
    pub fn set_profile(&mut self, profile: String) -> () { self.profile = profile.parse().unwrap(); }
    pub fn set_overlap(&mut self, overlap: String) -> () { self.overlap = overlap.parse().unwrap(); }
    pub fn set_losslevel(&mut self, losslevel: String) -> () { self.losslevel = losslevel.parse().unwrap(); }
    pub fn set_enable_ecc(&mut self) -> () { self.enable_ecc = true; }
    pub fn set_ecc_rate(&mut self, dsize: String, csize: String) -> () { self.ecc_rate = [dsize.parse().unwrap(), csize.parse().unwrap()]; }
    pub fn set_overwrite(&mut self) -> () { self.overwrite = true; }
    pub fn set_meta(&mut self, meta: (String, String)) -> () { self.meta.push((meta.0, meta.1.as_bytes().to_vec())); }
    pub fn set_image(&mut self, image: String) -> () { self.image_path = image; }
}

/** parse
 * Parse CLI arguments and return the action, input file, and parameters
 * Parameters: arguments
 * Returns: Action, Input file name / Pipe, any other parameters
 */
pub fn parse(args: Args) -> (String, String, String, CliParams) {
    let mut args: VecDeque<String> = args.collect();
    let mut params: CliParams = CliParams::new();
    args.pop_front().unwrap();
    if args.len() < 1 { return (String::new(), String::new(), String::new(), params); }

    let action = args.pop_front().unwrap();
    let mut metaaction = String::new();
    if HEADER_OPT.contains(&action.as_str()) {
        metaaction = args.pop_front().unwrap();
    }
    if args.len() < 1 { return (action, String::new(), String::new(), params); }
    let input = args.pop_front().unwrap();

    while args.len() > 0 {
        let key = args.pop_front().unwrap();

        if key.starts_with("-") {
            let key = key.trim_start_matches("-");

            if ["output", "out", "o"].contains(&key) {
                let value = args.pop_front().unwrap();
                params.set_output(value);
            }
            if ["bits", "bit", "b"].contains(&key) {
                let value = args.pop_front().unwrap();
                params.set_bits(value);
            }
            if ["srate", "sample-rate", "sr"].contains(&key) {
                let value = args.pop_front().unwrap();
                params.set_srate(value);
            }
            if ["chnl", "channels", "channel", "ch"].contains(&key) {
                let value = args.pop_front().unwrap();
                params.set_channels(value);
            }
            if ["frame-size", "fsize", "fr"].contains(&key) {
                let value = args.pop_front().unwrap();
                params.set_frame_size(value);
            }
            if ["overlap", "olap"].contains(&key) {
                let value = args.pop_front().unwrap();
                params.set_overlap(value);
            }
            if ["ecc", "enable-ecc", "e"].contains(&key) {
                params.set_enable_ecc();
                if !args.is_empty() {
                    if let Ok(_) = args[0].parse::<f64>() {
                        let v1 = args.pop_front().unwrap();
                        let v2 = args.pop_front().unwrap();
                        params.set_ecc_rate(v1, v2);
                    }
                }
            }
            if ["le", "little-endian"].contains(&key) {
                params.set_little_endian();
            }
            if ["profile", "prf", "p"].contains(&key) {
                let value = args.pop_front().unwrap();
                params.set_profile(value);
            }
            if ["losslevel", "level", "lv"].contains(&key) {
                let value = args.pop_front().unwrap();
                params.set_losslevel(value);
            }
            if ["y", "f"].contains(&key) {
                params.set_overwrite();
            }
            if ["tag", "meta", "m"].contains(&key) {
                let value = args.pop_front().unwrap();
                if metaaction == META_REMOVE { params.set_meta((value, String::new())); }
                else { params.set_meta((value, args.pop_front().unwrap())); }
            }
            if ["img", "image"].contains(&key) {
                let value = args.pop_front().unwrap();
                params.set_image(value);
            }
        }
    }

    return (action, metaaction, input, params);
}