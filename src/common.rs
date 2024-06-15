const fn gen_crc32t() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut crc = i as u32;
        let mut j = 0;
        while j < 8 {
            if crc & 1 == 1 { crc = (crc >> 1) ^ 0xedb88320; }
            else            { crc >>= 1; }
            j += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}

const CRC32T: [u32; 256] = gen_crc32t();

pub fn crc32(data: &[u8]) -> Vec<u8> {
    let mut crc = 0xffffffff;
    for &byte in data {
        crc = (crc >> 8) ^ CRC32T[((crc & 0xff) ^ byte as u32) as usize];
    }

    return (crc ^ 0xffffffff).to_be_bytes().to_vec();
}