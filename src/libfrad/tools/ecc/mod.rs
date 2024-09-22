/**                             Error Correction                              */
/**
 * Copyright 2024 HaמuL
 * Description: Error correction tools
 */

mod reedsolo;
pub use reedsolo::RSCodec;

/** encode_rs
 * Encodes data w. Reed-Solomon ECC
 * Parameters: Data, ECC ratio
 * Returns: Encoded data
 */
pub fn encode(data: Vec<u8>, ratio: [u8; 2]) -> Vec<u8> {
    let (data_size, parity_size) = (ratio[0] as usize, ratio[1] as usize);
    let rs: RSCodec = RSCodec::new_default(data_size, parity_size);

    return data.chunks(data_size).map(|chunk| {
        rs.encode(chunk)
    }).flatten().collect();
}

/** decode_rs
 * Decodes data and corrects errors w. Reed-Solomon ECC
 * Parameters: Data, ECC ratio
 * Returns: Decoded data
 */
pub fn decode(data: Vec<u8>, ratio: [u8; 2], repair: bool) -> Vec<u8> {
    let (data_size, parity_size) = (ratio[0] as usize, ratio[1] as usize);
    let block_size = data_size + parity_size;
    let rs: RSCodec = RSCodec::new_default(data_size, parity_size);

    return data.chunks(block_size).map(|chunk| {
        if repair {
            match rs.decode(chunk, None) {
                Ok(chunk) => chunk,
                Err(_) => vec![0; data_size]
            }
        } else { chunk.iter().take(chunk.len() - parity_size).cloned().collect() }
    }).flatten().collect();
}