/**                                CLI Parser                                 */
/**
 * Copyright 2024 HaמuL
 * Description: Simple CLI parser for FrAD Executable
 */

use frad::{Endian::{Big, Little}, PCMFormat};
use std::{collections::VecDeque, env::Args, fs::read_to_string, process::exit};

use base64::{prelude::BASE64_STANDARD, Engine};
use serde_json::{from_str, Value};

// CLI Options
pub const ENCODE_OPT: [&str; 2] = ["encode", "enc"];
pub const DECODE_OPT: [&str; 2] = ["decode", "dec"];
pub const REPAIR_OPT: [&str; 2] = ["repair", "ecc"];
pub const PLAY_OPT: [&str; 2] = ["play", "p"];
pub const METADATA_OPT: [&str; 2] = ["meta", "metadata"];
pub const JSONMETA_OPT: [&str; 2] = ["jsonmeta", "jm"];
pub const VORBISMETA_OPT: [&str; 2] = ["vorbismeta", "vm"];
pub const PROFILES_OPT: [&str; 2] = ["profiles", "prf"];
pub const HELP_OPT: [&str; 3] = ["help", "h", "?"];

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
    pub overlap_ratio: u16,
    pub losslevel: u8,
    pub enable_ecc: bool,
    pub ecc_ratio: [u8; 2],
    pub overwrite: bool,
    pub meta: Vec<(String, Vec<u8>)>,
    pub image_path: String,
    pub loglevel: u8,
    pub speed: f64,
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
            overlap_ratio: 16,
            losslevel: 0,
            enable_ecc: false,
            ecc_ratio: [96, 24],
            overwrite: false,
            meta: Vec::new(),
            image_path: String::new(),
            loglevel: 0,
            speed: 1.0,
        }
    }
    pub fn set_meta_from_json(&mut self, meta_path: String) {
        let contents = match read_to_string(meta_path) { Ok(c) => c, Err(_) => { return; } };
        let json_meta: Vec<Value> = match from_str(&contents) { Ok(m) => m, Err(_) => { return; } };

        for item in json_meta {
            let key = item["key"].as_str();
            let item_type = item["type"].as_str();
            let value_str = item["value"].as_str();

            if key.is_none() && value_str.is_none() { continue; }
            let key = key.unwrap_or_else(|| "");
            let value_str = value_str.unwrap_or_else(|| "");

            let value = if item_type == Some("base64") {
                match BASE64_STANDARD.decode(value_str) {
                    Ok(decoded) => decoded,
                    Err(_) => { continue; }
                }
            }
            else { value_str.as_bytes().to_vec() };
            self.meta.push((key.to_string(), value));
        }
    }
    pub fn set_meta_from_vorbis(&mut self, meta_path: String) {
        let contents = match read_to_string(meta_path) { Ok(c) => c, Err(_) => { return; } };
        let mut meta: Vec<(String, Vec<u8>)> = Vec::new();
        for line in contents.lines() {
            let mut parts = line.splitn(2, '=');
            let key = parts.next().unwrap();
            match parts.next() {
                Some(value) => meta.push((key.to_string(), value.as_bytes().to_vec())),
                None => {
                    if let Some(last) = meta.last_mut() {
                        last.1.extend(format!("\n{}", key).as_str().as_bytes());
                    }
                    else { meta.push(("".to_string(), key.as_bytes().to_vec())); }
                }
            }
        }
        self.meta = meta;
    }
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

            _ => { eprintln!("Invalid format: {fmt}"); exit(1); }
        };
    }
    pub fn set_loglevel(&mut self, loglevel: String) { self.loglevel = loglevel.parse().unwrap(); }
}

/** parse
 * Parse CLI arguments and return the action, input file, and parameters
 * Parameters: arguments
 * Returns: Action, Input file name / Pipe, any other parameters
 */
pub fn parse(args: Args) -> (String, String, String, CliParams) {
    let mut args: VecDeque<String> = args.collect();
    let mut params: CliParams = CliParams::new();
    let executable = args.pop_front().unwrap();
    if args.is_empty() { return (String::new(), String::new(), String::new(), params); }

    let action = args.pop_front().unwrap();
    let mut metaaction = String::new();
    if METADATA_OPT.contains(&action.as_str()) {
        metaaction = args.pop_front().unwrap_or_else(
            || { eprintln!("Metadata action not specified, type `{executable} help meta` for available options."); exit(1); }
        );
    }
    if args.is_empty() { return (action, String::new(), String::new(), params); }
    let input = args.pop_front().unwrap();

    while !args.is_empty() {
        let key = args.pop_front().unwrap();

        if key.starts_with("-") {
            let key = key.trim_start_matches("-");

            match key {
                // universal
                "output" | "out" | "o" => params.output = args.pop_front().unwrap(),
                "pcm" | "format" | "fmt" | "f" => params.set_pcm_format(&args.pop_front().unwrap()),
                "ecc" | "enable-ecc" | "e" => {
                    params.enable_ecc = true;
                    if !args.is_empty() && args[0].parse::<u8>().is_ok() {
                        params.ecc_ratio = [args.pop_front().unwrap().parse().unwrap(), args.pop_front().unwrap().parse().unwrap()];
                    }
                }
                "y" | "force" => params.overwrite = true,

                // encode settings
                "bits" | "bit" | "b" => params.bits = args.pop_front().unwrap().parse().unwrap(),
                "srate" | "sample-rate" | "sr" => params.srate = args.pop_front().unwrap().parse().unwrap(),
                "chnl" | "channels" | "channel" | "ch" => params.channels = args.pop_front().unwrap().parse().unwrap(),
                "frame-size" | "fsize" | "fr" => params.frame_size = args.pop_front().unwrap().parse().unwrap(),
                "overlap-ratio" | "overlap" | "olap" => params.overlap_ratio = args.pop_front().unwrap().parse().unwrap(),
                "le" | "little-endian" => params.little_endian = true,
                "profile" | "prf" | "p" => params.profile = args.pop_front().unwrap().parse().unwrap(),
                "losslevel" | "level" | "lv" => params.losslevel = args.pop_front().unwrap().parse().unwrap(),

                // metadata settings
                "tag" | "meta" | "m" => {
                    let value = args.pop_front().unwrap();
                    if metaaction == META_REMOVE { params.meta.push((value, Vec::new())); }
                    else { params.meta.push((value, args.pop_front().unwrap().as_bytes().to_vec())); }
                }
                "jsonmeta" | "jm" => params.set_meta_from_json(args.pop_front().unwrap()),
                "vorbismeta" | "vm" => params.set_meta_from_vorbis(args.pop_front().unwrap()),
                "img" | "image" => params.image_path = args.pop_front().unwrap(),
                "log" | "v" => {
                    if !args.is_empty() && args[0].parse::<u8>().is_ok() {
                        let value = args.pop_front().unwrap();
                        params.set_loglevel(value);
                    }
                    else { params.set_loglevel("1".to_string()); }
                }
                "speed" | "spd" => params.speed = args.pop_front().unwrap().parse().unwrap(),
                "keys" | "key" | "k" => params.speed = 2.0f64.powf(args.pop_front().unwrap().parse::<f64>().unwrap() / 12.0),
                _ => {}
            }
        }
    }

    return (action, metaaction, input, params);
}