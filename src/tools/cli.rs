use std::env::Args;

pub const ENCODE_OPT: [&str; 2] = ["encode", "enc"];
pub const DECODE_OPT: [&str; 2] = ["decode", "dec"];
pub const REPAIR_OPT: [&str; 4] = ["reecc", "re-ecc", "repair", "ecc"];

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
    pub overwrite: bool
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
            profile: 0,
            overlap: 16,
            losslevel: 0,
            enable_ecc: false,
            ecc_rate: [96, 24],
            overwrite: false
        }
    }
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
}

pub fn parse(args: Args) -> (String, String, CliParams) {
    let mut args: Vec<String> = args.collect();
    let mut params: CliParams = CliParams::new();
    args.remove(0);
    if args.len() < 1 { return (String::new(), String::new(), params); }

    let action = args.remove(0);
    if args.len() < 1 { return (action.to_string(), String::new(), params); }
    let input = args.remove(0);

    while args.len() > 0 {
        let key = args.remove(0);

        if key.starts_with("-") {
            let key = key.trim_start_matches("-");

            if ["output", "out", "o"].contains(&key) {
                let value = args.remove(0);
                params.set_output(value.to_string());
            }
            if ["bits", "bit", "b"].contains(&key) {
                let value = args.remove(0);
                params.set_bits(value.to_string());
            }
            if ["srate", "sample-rate", "sr"].contains(&key) {
                let value = args.remove(0);
                params.set_srate(value.to_string());
            }
            if ["chnl", "channels", "channel", "ch"].contains(&key) {
                let value = args.remove(0);
                params.set_channels(value.to_string());
            }
            if ["frame-size", "fsize", "fr"].contains(&key) {
                let value = args.remove(0);
                params.set_frame_size(value.to_string());
            }
            if ["overlap", "olap"].contains(&key) {
                let value = args.remove(0);
                params.set_overlap(value.to_string());
            }
            if ["ecc", "enable-ecc", "e"].contains(&key) {
                params.set_enable_ecc();
                if args.len() > 0 {
                    let v1 = args.remove(0);
                    let v2 = args.remove(0);
                    params.set_ecc_rate(v1, v2);
                }
            }
            if ["le", "little-endian"].contains(&key) {
                params.set_little_endian();
            }
            if ["profile", "prf", "p"].contains(&key) {
                let value = args.remove(0);
                params.set_profile(value.to_string());
            }
            if ["losslevel", "level", "lv"].contains(&key) {
                let value = args.remove(0);
                params.set_losslevel(value.to_string());
            }
            if ["y"].contains(&key) {
                params.set_overwrite();
            }
        }
    }

    return (action.to_string(), input.to_string(), params);
}