///                            Header Configurator                           ///
///
/// Copyright 2024 HaמuL
/// Description: FrAD Header Builder and Parser

use crate::{backend::SplitFront, common::SIGNATURE};

const COMMENT: [u8; 2] = [0xfa, 0xaa];
const IMAGE: [u8; 1] = [0xf5];

const COMMENT_HEAD_LENGTH: usize = 12;
const IMAGE_HEAD_LENGTH: usize = 10;

/// comment
/// Generates a comment block
/// Parameters: Title, Data
/// Returns: Comment block
fn comment(title: &str, data: &[u8]) -> Vec<u8> {
    let mut data_comb = title.as_bytes().to_vec();
    let title_length = (data_comb.len() as u32).to_be_bytes();
    data_comb.extend(data);
    let block_length = (data_comb.len() + 12).to_be_bytes()[2..].to_vec();

    let mut block = Vec::new();
    block.extend(COMMENT);
    block.extend(block_length);
    block.extend(title_length);
    block.extend(data_comb);

    return block;
}

/// image
/// Generates an image block
/// Parameters: Data, Picture type
/// Returns: Image block
fn image(data: Vec<u8>, itype: Option<u8>) -> Vec<u8> {
    let mut itype = itype.unwrap_or(3);
    itype = if itype > 20 { 3 } else { itype };
    let apictype = [0b01000000 | itype];
    let block_length = (data.len() + 10).to_be_bytes();

    let mut block = Vec::new();
    block.extend(IMAGE);
    block.extend(apictype);
    block.extend(block_length);
    block.extend(data);

    return block;
}

/// builder
/// Builds a header from metadata and image
/// Parameters: Metadata, Image
/// Returns: FrAD Header
pub fn builder(meta: &Vec<(String, Vec<u8>)>, img: Vec<u8>, itype: Option<u8>) -> Vec<u8> {
    let mut blocks = Vec::new();

    if !meta.is_empty() {
        for i in 0..meta.len() {
            blocks.extend(comment(&meta[i].0, &meta[i].1));
        }
    }
    if !img.is_empty() {
        blocks.extend(image(img, itype));
    }

    let length = (64 + blocks.len() as u64).to_be_bytes().to_vec();

    let mut header = Vec::new();
    header.extend(SIGNATURE);
    header.extend(vec![0; 4]);
    header.extend(length);
    header.extend(vec![0; 48]);
    header.extend(blocks);

    return header;
}

/// parser
/// Parses a header into metadata and image
/// Parameters: Header
/// Returns: Metadata in bytes, Image in bytes
pub fn parser(mut header: Vec<u8>) -> (Vec<(String, Vec<u8>)>, Vec<u8>, u8) {
    let mut meta = Vec::new();
    let (mut img, mut itype) = (Vec::new(), 0);
    while header.len() > 1 {
        if header[..2] == COMMENT {
            let block_length = u48be_to_u64(&header[2..8]) as usize;
            let title_length = u32::from_be_bytes(header[8..12].try_into().unwrap()) as usize;
            let mut block = header.split_front(block_length).split_off(COMMENT_HEAD_LENGTH);

            let title = String::from_utf8(block.split_front(title_length)).unwrap();
            meta.push((title, block));
        }
        else if header[..1] == IMAGE {
            itype = header[1] & 0b00011111;
            let block_length = u64::from_be_bytes(header[2..10].try_into().unwrap());

            img = header.split_front(block_length as usize).split_off(IMAGE_HEAD_LENGTH);
        }
        else { header.split_front(1); }
    }
    return (meta, img, itype);
}

/// u48be_to_u64
/// Converts a 48-bit big-endian number to a 64-bit number
/// Parameters: 48-bit / 6-byte number
/// Returns: u64 number
fn u48be_to_u64(data: &[u8]) -> u64 {
    if data.len() != 6 { return 0; }
    return u64::from_be_bytes([vec![0; 2], data.to_vec()].concat().try_into().unwrap());
}