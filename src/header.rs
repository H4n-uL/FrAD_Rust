/**                                  Header                                   */
/**
 * Copyright 2024 HaמuL
 * Description: Metadata modificator for FrAD
 */

use frad::{common::SIGNATURE, head};
use crate::{
    common::move_all,
    tools::cli::{CliParams, META_ADD, META_OVERWRITE, META_PARSE, META_REMOVE, META_RMIMG}
};
use std::{fs::File, io::{Read, Seek, SeekFrom, Write}, path::Path, process::exit};

use base64::{prelude::BASE64_STANDARD, Engine};
use infer;
use serde_json::{json, Value};
use tempfile::NamedTempFile;

/** modify
 * Modify the metadata of a FrAD file
 * Parameters: File path, Modification type, Metadata, Image path
 * Returns: FrAD file with modified metadata
 */
pub fn modify(file_name: String, modtype: String, params: CliParams) {
    if file_name.is_empty() { eprintln!("Input file must be given"); exit(1); }
    else if !Path::new(&file_name).exists() { eprintln!("Input file does not exist"); exit(1); }

    let mut head = vec![0u8; 64];

    let mut rfile = File::open(&file_name).unwrap();
    rfile.read_exact(&mut head).unwrap();

    let head_len = if head[0..4] == SIGNATURE { u64::from_be_bytes(head[8..16].try_into().unwrap()) } else { 0 };
    rfile.seek(SeekFrom::Start(0)).unwrap();
    let mut head_old = vec![0u8; head_len as usize];
    rfile.read_exact(&mut head_old).unwrap();

    let (mut meta_old, img_old, _itype) = head::parser(head_old);
    let (mut meta_new, mut img_new) = (Vec::new(), Vec::new());

    if modtype == META_PARSE {
        let mut json: Vec<Value> = Vec::new();
        for (key, data) in meta_old {
            let (data, itype) = match String::from_utf8(data.clone()) {
                Ok(data_str) => (data_str.to_string(), "string".to_string()),
                Err(_) => (BASE64_STANDARD.encode(data).to_string(), "base64".to_string())
            };
            json.push(json!({"key": key, "type": itype, "value": data}));
        }
        let mut wfile = params.output;

        if wfile.is_empty() {
            let wfrf = Path::new(&file_name).file_name().unwrap().to_str().unwrap().to_string();
            wfile = wfrf.split(".").collect::<Vec<&str>>()[..wfrf.split(".").count() - 1].join(".");
        }
        File::create(format!("{}.json", wfile)).unwrap().write_all(serde_json::to_string_pretty(&json).unwrap().as_bytes()).unwrap();
        if !img_old.is_empty() {
            let img_suffix = if let Some(imgtype) = infer::get(&img_old) { imgtype.extension() } else { "img" };
            File::create(format!("{}.{}", wfile, img_suffix)).unwrap().write_all(&img_old).unwrap();
        }

        return;
    }

    let mut temp = NamedTempFile::new().unwrap();
    move_all(&mut rfile, temp.as_file_mut(), 16777216);

    let mut img = Vec::new();
    if !params.image_path.is_empty() {
        match File::open(&params.image_path) {
            Ok(mut imgfile) => { imgfile.read_to_end(&mut img).unwrap(); },
            Err(_) => { eprintln!("Image not found"); }
        }
    }

    match modtype.as_str() {
        META_ADD => {
            if !meta_old.is_empty() { meta_new.append(&mut meta_old); }
            meta_new.extend(params.meta);
            if !img_old.is_empty() { img_new = img_old; }
            if !img.is_empty() { img_new = img; }
        }
        META_REMOVE => {
            meta_new = meta_old.into_iter().filter(|(title, _)| !params.meta.iter().any(|(t, _)| t == title)).collect();
            img_new = img_old;
        }
        META_RMIMG => {
            meta_new = meta_old;
            img_new = Vec::new();
        }
        META_OVERWRITE => {
            meta_new = params.meta;
            img_new = img;
        }
        _ => { eprintln!("Invalid modification type."); std::process::exit(1); }
    }

    let head_new = head::builder(&meta_new, img_new, None);

    let mut wfile = File::create(&file_name).unwrap();
    wfile.write_all(&head_new).unwrap();

    temp.seek(SeekFrom::Start(0)).unwrap();

    move_all(temp.as_file_mut(), &mut wfile, 16777216);
}