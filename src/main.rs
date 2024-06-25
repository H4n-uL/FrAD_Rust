mod backend;
mod fourier;
mod tools;
mod common;

mod encode;
mod decode;
mod repair;

use std::env;

const BANNER: &str =
"                    Fourier Analogue-in-Digital Rust Reference
                             Original Author - HaמuL
";

const ENCODE_HELP: &str = "--------------------------------- Description ----------------------------------

Encode
This action will encode your RAW f64be PCM audio file to FrAD format.
This action supports pipe input/output.

------------------------------------ Usage -------------------------------------

frad encode path/to/audio.file -srate [sample rate] -chnl [channels]
    --bits [bit depth] {kwargs...}

----------------------------------- Options ------------------------------------

    --sample-rate | Input sample rate, REQUIRED (alias: sr, srate)
    --channels    | Input hannels, REQUIRED (alias: ch, chnl, channel)
    --bits        | Bit depth, REQUIRED (alias: b, bit)
                  |
    --ecc         | Enable ECC, recommended.
                  | ECC size ratio in --ecc [data size] [ecc code size]
                  | default: 96, 24 (alias: e, enable-ecc)
    --output      | Output file path (alias: o, out)
                  |
    --fsize       | Samples per frame, default: 2048 (alias: fr, frame-size)
    --le          | Little Endian Toggle (alias: little-endian)
    --overlap     | Overlap ratio in 1/{{value}} (alias: olap)
                  |
    --profile     | FrAD Profile from 0 to 7, not recommended (alias: prf)
    --loss-level  | Lossy compression level, default: 0 (alias: lv, level)";

const DECODE_HELP: &str = "--------------------------------- Description ----------------------------------

Decode
This action will decode any supported FrAD audio file to RAW f64be PCM format.
This action supports pipe input/output.

------------------------------------ Usage -------------------------------------

frad decode path/to/audio.frad {kwargs...}

----------------------------------- Options ------------------------------------

    --output      | Output file path (alias: o, out)
    --ecc         | Check and fix errors, default: false (alias: e, enable-ecc)
";

const REPAIR_HELP: &str = "--------------------------------- Description ----------------------------------

Repair
This action will repair any supported FrAD audio file with ECC protection.

------------------------------------ Usage -------------------------------------

frad repair path/to/audio.frad --output path/to/audio_ecc.frad {kwargs...}

----------------------------------- Options ------------------------------------

    --output      | Output file path, REQUIRED (alias: o, out)
    --ecc         | ECC size ratio in --ecc [data size] [ecc code size]
                  | default: 96, 24 (alias: e, enable-ecc)";

/** Main function  */
fn main() {
    let (action, input, params) = tools::cli::parse(env::args());

    if tools::cli::ENCODE_OPT.contains(&action.as_str()) {
        encode::encode(input, params);
    }
    else if tools::cli::DECODE_OPT.contains(&action.as_str()) {
        decode::decode(input, params);
    }
    else if tools::cli::REPAIR_OPT.contains(&action.as_str()) {
        repair::repair(input, params);
    }
    else if &action == &"help".to_string() {
        eprintln!("{}", BANNER);
        if tools::cli::ENCODE_OPT.contains(&input.as_str()) {
            eprintln!("{}", ENCODE_HELP);
        }
        else if tools::cli::DECODE_OPT.contains(&input.as_str()) {
            eprintln!("{}", DECODE_HELP);
        }
        else if tools::cli::REPAIR_OPT.contains(&input.as_str()) {
            eprintln!("{}", REPAIR_HELP);
        }
        else { eprintln!("------------------------------- Available actions ------------------------------

    encode | Encode any audio formats to FrAD (alias: enc)
    decode | Encode FrAD to any audio formats (alias: dec)
    repair | Enable ECC protection / Repair file (alias: ecc, reecc, re-ecc)"
        );}
        eprintln!();
    }
    else {
        eprintln!("Fourier Analogue-in-Digital Rust Reference");
        eprintln!("Abstract syntax: frad [encode|decode|repair] <input> [kwargs...]");
        eprintln!("type 'frad help' to get help.");
    }
}