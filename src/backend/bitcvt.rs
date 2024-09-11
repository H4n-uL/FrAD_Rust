/**                                Bitconvert                                 */
/**
 * Copyright 2024 HaמuL
 * Function: Convert byte array to bitstream and vice versa
 */

/** to_bits
 * Converts byte array to bitstream
 * Parameters: Byte array
 * Returns: Bitstream
 */
pub fn to_bits(bytes: Vec<u8>) -> Vec<bool> {
    let mut bitstream: Vec<bool> = Vec::new();

    for byte in bytes {
        for i in 0..8 { bitstream.push((byte >> (7 - i)) & 1 == 1); }
    }
    return bitstream;
}

/** to_bytes
 * Converts bitstream to byte array
 * Parameters: Bitstream
 * Returns: Byte array
 */
pub fn to_bytes(bitstream: Vec<bool>) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    let mut byte: u8 = 0u8;

    for (i, bit) in bitstream.iter().enumerate() {
        if *bit { byte |= 1 << (7 - (i % 8)); }
        if (i + 1) % 8 == 0 { bytes.push(byte); byte = 0; }
    }
    if bitstream.len() % 8 != 0 { bytes.push(byte); } // flushing last bits with zero padding
    return bytes;
}