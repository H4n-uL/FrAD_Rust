/**                            Header Configurator                            */
/**
 * Copyright 2024 HaמuL
 * Description: FrAD Header Builder and Parser
 */

use crate::common::SIGNATURE;

const COMMENT: [u8; 2] = [0xfa, 0xaa];
const IMAGE: [u8; 1] = [0xf5];

/** comment
 * Generates a comment block
 * Parameters: Title, Data
 * Returns: Comment block
 */
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

/** image
 * Generates an image block
 * Parameters: Data, Picture type
 * Returns: Image block
 */
fn image(data: Vec<u8>, pictype: Option<u8>) -> Vec<u8> {
    let mut pictype = pictype.unwrap_or(3);
    pictype = if pictype > 20 { 3 } else { pictype };
    let apictype = [0b01000000 | pictype];
    let block_length = (data.len() + 10).to_be_bytes();

    let mut block = Vec::new();
    block.extend(IMAGE);
    block.extend(apictype);
    block.extend(block_length);
    block.extend(data);

    return block;
}

/** builder
 * Builds a header from metadata and image
 * Parameters: Metadata, Image
 * Returns: FrAD Header
 */
pub fn builder(meta: &Vec<(String, Vec<u8>)>, img: Vec<u8>) -> Vec<u8> {
    let mut blocks = Vec::new();

    if !meta.is_empty() {
        for i in 0..meta.len() {
            blocks.extend(comment(&meta[i].0, &meta[i].1));
        }
    }
    if !img.is_empty() {
        blocks.extend(image(img, None));
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

/** parser
 * Parses a header into metadata and image
 * Parameters: Header
 * Returns: Metadata in bytes, Image in bytes
 */
pub fn parser(mut header: Vec<u8>) -> (Vec<(String, Vec<u8>)>, Vec<u8>) {
    let mut meta = Vec::new();
    let mut img = Vec::new();
    loop {
        if header.len() < 2 { break; }
        let block_type = &header[0..2];
        if block_type == COMMENT {
            let block_length = u64::from_be_bytes([0, 0, header[2], header[3], header[4], header[5], header[6], header[7]].try_into().unwrap());
            let title_length = u32::from_be_bytes(header[8..12].try_into().unwrap());

            let title = String::from_utf8(header[12..12 + title_length as usize].to_vec()).unwrap();
            let data = header[12 + title_length as usize..block_length as usize].to_vec();
            meta.push((title, data));
            header = header[block_length as usize..].to_vec();
        }
        else if block_type[0] == IMAGE[0] {
            let block_length = u64::from_be_bytes(header[2..10].try_into().unwrap());
            img = header[10..block_length as usize].to_vec();
            header = header[block_length as usize..].to_vec();
        }
        else { header = header[1..].to_vec(); }
    }
    return (meta, img);
}