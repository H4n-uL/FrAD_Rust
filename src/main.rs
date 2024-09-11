mod backend; mod fourier;
mod tools; mod tools_app;
mod common; mod common_app;
mod encode; mod decode; mod repair; mod header;

use std::{env, path::Path};

const BANNER: &str =
"                   Fourier Analogue-in-Digital Rust Reference
                             Original Author - HaמuL
";

const GENERAL_HELP: &str = include_str!("help/general.txt");
const ENCODE_HELP: &str = include_str!("help/encode.txt");
const DECODE_HELP: &str = include_str!("help/decode.txt");
const PLAY_HELP: &str = include_str!("help/play.txt");
const REPAIR_HELP: &str = include_str!("help/repair.txt");
const METADATA_HELP: &str = include_str!("help/metadata.txt");
const JSONMETA_HELP: &str = include_str!("help/jsonmeta.txt");
const PROFILES_HELP: &str = include_str!("help/profiles.txt");

/** Main function  */
fn main() {
    let exepath = env::args().next().unwrap();
    let executable = Path::new(&exepath).file_name().unwrap().to_str().unwrap();
    let (action, metaaction, input, params) = tools_app::cli::parse(env::args());

    let loglevel = params.loglevel;
    if tools_app::cli::ENCODE_OPT.contains(&action.as_str()) {
        encode::encode(input, params, loglevel);
    }
    else if tools_app::cli::DECODE_OPT.contains(&action.as_str())
        || tools_app::cli::PLAY_OPT.contains(&action.as_str()) {
        decode::decode(input, params, loglevel);
    }
    else if tools_app::cli::REPAIR_OPT.contains(&action.as_str()) {
        repair::repair(input, params, loglevel);
    }
    else if tools_app::cli::METADATA_OPT.contains(&action.as_str()) {
        header::modify(input, metaaction, params);
    }
    else if tools_app::cli::HELP_OPT.contains(&action.as_str()) {
        println!("{}", BANNER);
        println!("{}",
            if tools_app::cli::ENCODE_OPT.contains(&input.as_str()) { ENCODE_HELP }
            else if tools_app::cli::DECODE_OPT.contains(&input.as_str()) { DECODE_HELP }
            else if tools_app::cli::REPAIR_OPT.contains(&input.as_str()) { REPAIR_HELP }
            else if tools_app::cli::PLAY_OPT.contains(&input.as_str()) { PLAY_HELP }
            else if tools_app::cli::METADATA_OPT.contains(&input.as_str()) { METADATA_HELP }
            else if tools_app::cli::JSONMETA_OPT.contains(&input.as_str()) { JSONMETA_HELP }
            else if tools_app::cli::PROFILES_OPT.contains(&input.as_str()) { PROFILES_HELP }
            else { GENERAL_HELP }.replace("{frad}", executable)
        );
        println!();
    }
    else {
        eprintln!("Fourier Analogue-in-Digital Rust Reference");
        eprintln!("Abstract syntax: {exepath} [encode|decode|repair] <input> [kwargs...]");
        eprintln!("type '{executable} help' to get help.");
    }
}