/**                                  Header                                   */
/**
 * Copyright 2024 HaמuL
 * Function: Add or Remove metadata from FrAD file
 */

use std::{fs::File, io::{Read, Seek, SeekFrom, Write}};
use tempfile::NamedTempFile;
use crate::{common::SIGNATURE, tools::head};

/** modify
 * Modify the metadata of a FrAD file
 * Parameters: File path, Modification type, Metadata, Image path
 * Returns: FrAD file with modified metadata
 */
pub fn modify(file_name: String, modtype: String, mut meta: Vec<(String, Vec<u8>)>, img_path: String) {
    if file_name.len() == 0 { eprintln!("File path is required."); std::process::exit(1); }

    let add = modtype == "add";
    let remove_img = modtype == "remove-img";
    let overwrite = modtype == "overwrite";

    let mut img = Vec::new();
    if img_path.len() > 0 {
        match File::open(&img_path) {
            Ok(mut imgfile) => { imgfile.read_to_end(&mut img).unwrap(); },
            Err(_) => { eprintln!("Image not found"); }
        }
    }

    let mut head = vec![0u8; 64];

    let mut fread = File::open(&file_name).unwrap();
    fread.read_exact(&mut head).unwrap();

    let head_len = if head[0..4] == SIGNATURE { u64::from_be_bytes(head[8..16].try_into().unwrap()) } else { 0 };
    fread.seek(SeekFrom::Start(0)).unwrap();
    let mut head_original = vec![0u8; head_len as usize];
    fread.read_exact(&mut head_original).unwrap();

    let mut temp = NamedTempFile::new().unwrap();
    loop {
        let mut buf: Vec<u8> = vec![0; 1048576];
        let mut total_read = 0;

        while total_read < buf.len() {
            let read_size = fread.read(&mut buf[total_read..]).unwrap();
            if read_size == 0 { break; }
            total_read += read_size;
        }
        if total_read == 0 { break; }
        temp.write_all(&buf[..total_read]).unwrap();
    }

    let (mut meta_old, img_old) = head::parser(head_original);
    let (mut meta_new, mut img_new) = (Vec::new(), Vec::new());

    if add {
        if meta_old.len() > 0 { meta_new.append(&mut meta_old); }
        meta_new.append(&mut meta);
        if img_old.len() > 0 { img_new = img_old; }
        if img.len() > 0 { img_new = img; }
    }
    else if remove_img {
        meta_new = meta_old;
        img_new = Vec::new();
    }
    else if overwrite {
        meta_new = meta;
        img_new = img;
    }
    else {
        eprintln!("Invalid modification type.");
        std::process::exit(1);
    }

    let head_new = head::builder(&meta_new, img_new);

    let mut fwrite = File::create(&file_name).unwrap();
    fwrite.write_all(&head_new).unwrap();

    temp.seek(SeekFrom::Start(0)).unwrap();
    loop {
        let mut buf: Vec<u8> = vec![0; 1048576];
        let mut total_read = 0;

        while total_read < buf.len() {
            let read_size = temp.read(&mut buf[total_read..]).unwrap();
            if read_size == 0 { break; }
            total_read += read_size;
        }
        if total_read == 0 { break; }
        fwrite.write_all(&buf[..total_read]).unwrap();
    }
}