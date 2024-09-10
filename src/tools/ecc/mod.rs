/**                             Error Correction                              */
/**
 * Copyright 2024 HaמuL
 * Function: Error correction tools
 */

mod reedsolo;
pub use reedsolo::RSCodec;

/** encode_rs
 * Encodes data w. Reed-Solomon ECC
 * Parameters: Data, data length, code length
 * Returns: Encoded data
 */
pub fn encode_rs(data: Vec<u8>, dlen: usize, codelen: usize) -> Vec<u8> {
    let block_sz: usize = dlen + codelen;
    let rs = RSCodec::new_default(codelen, block_sz);

    let encoded_chunks = data.chunks(dlen).map(|chunk| {
        rs.encode(chunk)
    });

    return encoded_chunks.flatten().collect();
}

/** decode_rs
 * Decodes data and corrects errors w. Reed-Solomon ECC
 * Parameters: Data, data length, code length
 * Returns: Decoded data
 */
pub fn decode_rs(data: Vec<u8>, dlen: usize, codelen: usize) -> Vec<u8> {
    let block_sz: usize = dlen + codelen;
    let rs = RSCodec::new_default(codelen, block_sz);

    let decoded_chunks = data.chunks(block_sz).map(|chunk| {
        rs.decode(chunk, None)
    });

    let mut decoded = Vec::new();
    for chunk in decoded_chunks {
        match chunk {
            Ok(chunk) => decoded.extend(chunk),
            Err(_e) => decoded.extend(vec![0; dlen])
        }
    }

    return decoded;
}

/** unecc
 * Removes error correction code from data
 * Parameters: Data, data length, code length
 * Returns: Data without error correction code
 */
pub fn unecc(data: Vec<u8>, dlen: usize, codelen: usize) -> Vec<u8> {
    let block_sz: usize = dlen + codelen;
    let mut decoded: Vec<u8> = Vec::new();

    for chunk in data.chunks(block_sz) {
        decoded.extend(chunk.iter().take(chunk.len() - codelen).cloned());
    }

    return decoded;
}