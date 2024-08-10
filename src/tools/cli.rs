/**                                CLI Parser                                 */
/**
 * Copyright 2024 HaמuL
 * Function: Simple CLI parser for FrAD Library
 */

use base64::{prelude::BASE64_STANDARD, Engine as _};
use serde_json::{from_str, Value};
use std::{collections::VecDeque, env::Args, fs::read_to_string};

use crate::common::{Endian::{Big, Little}, PCMFormat};

// CLI Options
pub const ENCODE_OPT: [&str; 2] = ["encode", "enc"];
pub const DECODE_OPT: [&str; 2] = ["decode", "dec"];
pub const REPAIR_OPT: [&str; 4] = ["reecc", "re-ecc", "repair", "ecc"];
pub const METADATA_OPT: [&str; 2] = ["meta", "metadata"];

pub const META_ADD: &str = "add";
pub const META_REMOVE: &str = "remove";
pub const META_RMIMG: &str = "rm-img";
pub const META_OVERWRITE: &str = "overwrite";
pub const META_PARSE: &str = "parse";

// CLI Parameters
pub struct CliParams {
    pub output: String,
    pub pcm: PCMFormat,
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
            pcm: PCMFormat::F64(Big),
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
    pub fn set_output(&mut self, output: String) { self.output = output; }
    pub fn set_bits(&mut self, bits: String) { self.bits = bits.parse().unwrap(); }
    pub fn set_srate(&mut self, srate: String) { self.srate = srate.parse().unwrap(); }
    pub fn set_channels(&mut self, channels: String) { self.channels = channels.parse().unwrap(); }
    pub fn set_frame_size(&mut self, frame_size: String) { self.frame_size = frame_size.parse().unwrap(); }
    pub fn set_little_endian(&mut self) { self.little_endian = true; }
    pub fn set_profile(&mut self, profile: String) { self.profile = profile.parse().unwrap(); }
    pub fn set_overlap(&mut self, overlap: String) { self.overlap = overlap.parse().unwrap(); }
    pub fn set_losslevel(&mut self, losslevel: String) { self.losslevel = losslevel.parse().unwrap(); }
    pub fn set_enable_ecc(&mut self) { self.enable_ecc = true; }
    pub fn set_ecc_rate(&mut self, dsize: String, csize: String) { self.ecc_rate = [dsize.parse().unwrap(), csize.parse().unwrap()]; }
    pub fn set_overwrite(&mut self) { self.overwrite = true; }
    pub fn set_meta(&mut self, meta: (String, String)) { self.meta.push((meta.0, meta.1.as_bytes().to_vec())); }
    pub fn set_meta_from_json(&mut self, meta_path: String) {
        let contents = match read_to_string(meta_path) { Ok(c) => c, Err(_) => { return; } };
        let json_meta: Vec<Value> = match from_str(&contents) { Ok(m) => m, Err(_) => { return; } };

        for item in json_meta {
            let key = item["key"].as_str().map(String::from);
            let item_type = item["type"].as_str();
            let value_str = item["value"].as_str();

            match (key, item_type, value_str) {
                (Some(k), Some(t), Some(v)) => {
                    let value = if t == "base64" {
                        match BASE64_STANDARD.decode(v) {
                            Ok(decoded) => decoded,
                            Err(_) => { continue; }
                        }
                    } else { v.as_bytes().to_vec() };
                    self.meta.push((k, value));
                }
                _ => { continue; }
            }
        }
    }
    pub fn set_image(&mut self, image: String) { self.image_path = image; }
    pub fn set_pcm_format(&mut self, fmt: &str) {
        self.pcm = match fmt {
            "s8" => PCMFormat::I8,
            "u8" => PCMFormat::U8,

            "s16be" => PCMFormat::I16(Big),
            "s16le" => PCMFormat::I16(Little),
            "u16be" => PCMFormat::U16(Big),
            "u16le" => PCMFormat::U16(Little),

            "s24be" => PCMFormat::I24(Big),
            "s24le" => PCMFormat::I24(Little),
            "u24be" => PCMFormat::U24(Big),
            "u24le" => PCMFormat::U24(Little),

            "s32be" => PCMFormat::I32(Big),
            "s32le" => PCMFormat::I32(Little),
            "u32be" => PCMFormat::U32(Big),
            "u32le" => PCMFormat::U32(Little),

            "s64be" => PCMFormat::I64(Big),
            "s64le" => PCMFormat::I64(Little),
            "u64be" => PCMFormat::U64(Big),
            "u64le" => PCMFormat::U64(Little),

            "f16be" => PCMFormat::F16(Big),
            "f16le" => PCMFormat::F16(Little),

            "f32be" => PCMFormat::F32(Big),
            "f32le" => PCMFormat::F32(Little),

            "f64be" => PCMFormat::F64(Big),
            "f64le" => PCMFormat::F64(Little),

            _ => PCMFormat::F64(Big)
        };
    }
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
    if args.is_empty() { return (String::new(), String::new(), String::new(), params); }

    let action = args.pop_front().unwrap();
    let mut metaaction = String::new();
    if METADATA_OPT.contains(&action.as_str()) {
        metaaction = args.pop_front().unwrap();
    }
    if args.is_empty() { return (action, String::new(), String::new(), params); }
    let input = args.pop_front().unwrap();

    while !args.is_empty() {
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
            if ["pcm", "format", "fmt"].contains(&key) {
                let value = args.pop_front().unwrap();
                params.set_pcm_format(&value);
            }
            if ["overlap", "olap"].contains(&key) {
                let value = args.pop_front().unwrap();
                params.set_overlap(value);
            }
            if ["ecc", "enable-ecc", "e"].contains(&key) {
                params.set_enable_ecc();
                if !args.is_empty() && args[0].parse::<u8>().is_ok() {
                    let v1 = args.pop_front().unwrap();
                    let v2 = args.pop_front().unwrap();
                    params.set_ecc_rate(v1, v2);
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
            if ["jsonmeta", "jm"].contains(&key) {
                params.set_meta_from_json(args.pop_front().unwrap());
            }
            if ["img", "image"].contains(&key) {
                let value = args.pop_front().unwrap();
                params.set_image(value);
            }
        }
    }

    return (action, metaaction, input, params);
}