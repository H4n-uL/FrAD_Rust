mod backend; mod fourier; mod tools; mod common;
mod encode; mod decode; mod repair; mod header;

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

{frad} encode path/to/audio.file
    --sample-rate [sample rate]
    --channels [channels]
    --bits [bit depth]
    {{kwargs...}}

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
    --profile     | FrAD Profile from 0 to 7 (alias: prf)
    --loss-level  | Lossy compression level, default: 0 (alias: lv, level)
                  |
    --meta        | Metadata in [key] [value] (alias: m, tag)
    --image       | Image file path to embed (alias: img)";

const DECODE_HELP: &str = "--------------------------------- Description ----------------------------------

Decode
This action will decode any supported FrAD audio file to RAW f64be PCM format.
This action supports pipe input/output.

------------------------------------ Usage -------------------------------------

{frad} decode path/to/audio.frad
    {{kwargs...}}

----------------------------------- Options ------------------------------------

    --output      | Output file path (alias: o, out)
    --ecc         | Check and fix errors, default: false (alias: e, enable-ecc)
";

const REPAIR_HELP: &str = "--------------------------------- Description ----------------------------------

Repair
This action will repair any supported FrAD audio file with ECC protection.

------------------------------------ Usage -------------------------------------

{frad} repair path/to/audio.frad
    --output path/to/audio_ecc.frad
    {{kwargs...}}

----------------------------------- Options ------------------------------------

    --output      | Output file path, REQUIRED (alias: o, out)
    --ecc         | ECC size ratio in --ecc [data size] [ecc code size]
                  | default: 96, 24 (alias: e, enable-ecc)";

const HEADER_HELP: &str = "--------------------------------- Description ----------------------------------

Header
This action will modify the metadata of the FrAD audio file.

------------------------------------ Usage -------------------------------------

{frad} meta [meta-action] path/to/audio.frad
    {{kwargs...}}
    
----------------------------------- Options ------------------------------------

    add -           Add metadata and image to the FrAD file
    --meta        | Metadata in [key] [value] (alias: m, tag)
    --image       | Image file path to embed, replace if exists (alias: img)

    remove -        Remove metadata from the FrAD file
    --meta        | Metadata key to remove (alias: m, tag)

    rm-img -        Remove image from the FrAD file
    No option for this action.

    overwrite -     Remove all metadata and rewrite whole header
    --meta        | Metadata in [key] [value] (alias: m, tag)
    --image       | Image file path to embed (alias: img)";

/** Main function  */
fn main() {
    let executable = env::args().next().unwrap();
    let (action, metaaction, input, params) = tools::cli::parse(env::args());

    if tools::cli::ENCODE_OPT.contains(&action.as_str()) {
        encode::encode(input, params);
    }
    else if tools::cli::DECODE_OPT.contains(&action.as_str()) {
        decode::decode(input, params);
    }
    else if tools::cli::REPAIR_OPT.contains(&action.as_str()) {
        repair::repair(input, params);
    }
    else if tools::cli::HEADER_OPT.contains(&action.as_str()) {
        header::modify(input, metaaction, params.meta, params.image_path);
    }
    else if &action == &"help".to_string() {
        println!("{}", BANNER);
        println!("{}",
                 if tools::cli::ENCODE_OPT.contains(&input.as_str()) { ENCODE_HELP }
            else if tools::cli::DECODE_OPT.contains(&input.as_str()) { DECODE_HELP }
            else if tools::cli::REPAIR_OPT.contains(&input.as_str()) { REPAIR_HELP }
            else if tools::cli::HEADER_OPT.contains(&input.as_str()) { HEADER_HELP }
            else { "------------------------------- Available actions ------------------------------

    encode | Encode any audio formats to FrAD    (alias: enc)
    decode | Encode FrAD to any audio formats    (alias: dec)
    repair | Enable ECC protection / Repair file (alias: ecc, reecc, re-ecc)
    meta   | Edit metadata on FrAD               (alias: metadata)

------------------------------ Available profiles ------------------------------

    Profile 0 - DCT Archiving, Recommended for extreme environments
    Profile 1 - Compact file size, Low complexity
    Profile 2 - In development
    Profile 3 - (Reserved)
    Profile 4 - PCM Archiving, Recommended for general use
    Profile 5 - (Reserved)
    Profile 6 - (Reserved)
    Profile 7 - (Reserved)
    
Type `{frad} help [action]` to get help for specific action." }.replace("{frad}", executable.as_str())
        );
        eprintln!();
    }
    else {
        eprintln!("Fourier Analogue-in-Digital Rust Reference");
        eprintln!("Abstract syntax: {executable} [encode|decode|repair] <input> [kwargs...]");
        eprintln!("type '{executable} help' to get help.");
    }
}