pub fn fromvec(bytes: Vec<u8>) -> Vec<bool> {
    let mut bitstream: Vec<bool> = Vec::new();

    for byte in bytes {
        for i in 0..8 { bitstream.push((byte >> (7 - i)) & 1 == 1); }
    }
    return bitstream;
}

pub fn tovec(bitstream: Vec<bool>) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    let mut byte: u8 = 0u8;

    for (i, bit) in bitstream.iter().enumerate() {
        if *bit { byte |= 1 << (7 - (i % 8)); }
        if (i + 1) % 8 == 0 { bytes.push(byte); byte = 0; }
    }
    if bitstream.len() % 8 != 0 { bytes.push(byte); }
    return bytes;
}