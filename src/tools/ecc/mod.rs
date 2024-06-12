mod reedsolo;

pub use reedsolo::RSCodec;

pub fn encode_rs(data: Vec<u8>, dlen: usize, codelen: usize) -> Vec<u8> {
    let block_sz = dlen + codelen;
    let rs = RSCodec::new(codelen, block_sz, 0, 0x11d, 2, 8);

    let encoded_chunks = data.chunks(block_sz).map(|chunk| {
        rs.encode(chunk)
    });

    encoded_chunks.flatten().collect()
}

pub fn decode_rs(data: Vec<u8>, dlen: usize, codelen: usize) -> Vec<u8> {
    let block_sz = dlen + codelen;
    let rs = RSCodec::new(codelen, block_sz, 0, 0x11d, 2, 8);

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

    decoded
}