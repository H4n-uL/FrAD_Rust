//!                        AAPM@Audio-8151 Executable                        !//
//!
//! Copyright 2024-2025 HaƞuL
//! Description: Fourier Analogue-in-Digital Rust Master

mod tools; mod common;
mod encoder; mod decoder; mod repairer; mod header;

use std::{env, path::Path};

const BANNER: &str =
"                     Fourier Analogue-in-Digital Rust Master
                             Original Author - HaƞuL
";

const GENERAL_HELP:    &str = include_str!("help/general.txt");
const ENCODE_HELP:     &str = include_str!("help/encode.txt");
const DECODE_HELP:     &str = include_str!("help/decode.txt");
const REPAIR_HELP:     &str = include_str!("help/repair.txt");
const PLAY_HELP:       &str = include_str!("help/play.txt");
const METADATA_HELP:   &str = include_str!("help/metadata.txt");
const JSONMETA_HELP:   &str = include_str!("help/jsonmeta.txt");
const VORBISMETA_HELP: &str = include_str!("help/vorbismeta.txt");
const PROFILES_HELP:   &str = include_str!("help/profiles.txt");

/// Main function ///
fn main() {
    let exepath = env::args().next().unwrap();
    let executable = Path::new(&exepath).file_name().unwrap().to_str().unwrap();
    let (action, metaaction, input, params) = tools::cli::parse(env::args());

    if tools::cli::ENCODE_OPT.contains(&action.as_str()) {
        encoder::encode(input, params);
    }
    else if tools::cli::DECODE_OPT.contains(&action.as_str()) {
        decoder::decode(input, params, false);
    }
    else if tools::cli::PLAY_OPT.contains(&action.as_str()) {
        decoder::decode(input, params, true);
    }
    else if tools::cli::REPAIR_OPT.contains(&action.as_str()) {
        repairer::repair(input, params);
    }
    else if tools::cli::METADATA_OPT.contains(&action.as_str()) {
        header::modify(input, metaaction, params);
    }
    else if tools::cli::HELP_OPT.contains(&action.as_str()) {
        println!("{}", BANNER);
        println!("{}",
            if tools::cli::ENCODE_OPT.contains(&input.as_str()) { ENCODE_HELP }
            else if tools::cli::DECODE_OPT.contains(&input.as_str()) { DECODE_HELP }
            else if tools::cli::REPAIR_OPT.contains(&input.as_str()) { REPAIR_HELP }
            else if tools::cli::PLAY_OPT.contains(&input.as_str()) { PLAY_HELP }
            else if tools::cli::METADATA_OPT.contains(&input.as_str()) { METADATA_HELP }
            else if tools::cli::JSONMETA_OPT.contains(&input.as_str()) { JSONMETA_HELP }
            else if tools::cli::VORBISMETA_OPT.contains(&input.as_str()) { VORBISMETA_HELP }
            else if tools::cli::PROFILES_OPT.contains(&input.as_str()) { PROFILES_HELP }
            else { GENERAL_HELP }.replace("{frad}", executable)
        );
    }
    else {
        eprintln!("Fourier Analogue-in-Digital Rust Master");
        eprintln!("Abstract syntax: {executable} [encode|decode|repair] <input> [kwargs...]");
        eprintln!("Type `{executable} help` to get help.");
    }
}
