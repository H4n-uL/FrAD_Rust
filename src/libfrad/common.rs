/**                               Common tools                                */
/**
 * Copyright 2024 HaמuL
 * Description: Common tools for FrAD
 */

// signatures
pub const SIGNATURE: [u8; 4] = [0x66, 0x52, 0x61, 0x64];
pub const FRM_SIGN: [u8; 4] = [0xff, 0xd0, 0xd2, 0x97];

// CRC-32 Table generator
const fn gcrc32t() -> [[u32; 256]; 4] {
    let mut tables = [[0u32; 256]; 4];

    let mut i = 0; while i < 256 {
        let (mut crc, mut j) = (i as u32, 0);
        while j < 8 {
            if crc & 1 == 1 { crc = (crc >> 1) ^ 0xedb88320; } else { crc >>= 1; }
        j += 1; }
        tables[0][i] = crc;
    i += 1; }

    let mut i = 0; while i < 256 {
        let mut j = 1; while j < 4 { // table count
            tables[j][i] = (tables[j-1][i] >> 8) ^ tables[0][tables[j-1][i] as u8 as usize];
        j += 1; }
    i += 1; }

    return tables;
}

// CRC-32 Table
const CRC32T: [[u32; 256]; 4] = gcrc32t();

/** crc32
 * Calculates CRC-32 checksum of a byte array
 * Parameters: Byte array
 * Returns: CRC-32 checksum in byte array
 */
pub fn crc32(data: &[u8]) -> Vec<u8> {
    let mut crc = u32::MAX;
    let chunks = data.chunks_exact(4);
    let rem = chunks.remainder();

    chunks.for_each(|chunk| {
        crc ^= u32::from_le_bytes(chunk.try_into().unwrap());
        crc = CRC32T[3][( crc        & 0xff) as usize] ^
              CRC32T[2][((crc >>  8) & 0xff) as usize] ^
              CRC32T[1][((crc >> 16) & 0xff) as usize] ^
              CRC32T[0][((crc >> 24) & 0xff) as usize];
    });

    rem.iter().for_each(|&byte| { crc = (crc >> 8) ^ CRC32T[0][(crc ^ byte as u32) as usize]; });

    return (!crc).to_be_bytes().to_vec();
}

// CRC-16 ANSI Table generator
const fn gcrc16t_ansi() -> [[u16; 256]; 2] {
    let mut tables = [[0u16; 256]; 2];
    
    // 기본 테이블 생성
    let mut i = 0; while i < 256 {
        let mut crc = i as u16;
        let mut j = 0; while j < 8 {
            crc = if crc & 1 == 1 { (crc >> 1) ^ 0xA001 } else { crc >> 1 };
        j += 1; }
        tables[0][i] = crc;
    i += 1; }
    
    // 두 번째 테이블 생성
    let mut i = 0; while i < 256 {
        tables[1][i] = (tables[0][i] >> 8) ^ tables[0][tables[0][i] as u8 as usize];
    i += 1; }
    
    return tables;
}

// CRC-16 ANSI Table
const CRC16T_ANSI: [[u16; 256]; 2] = gcrc16t_ansi();

/** crc16_ansi
 * Calculates CRC-16 ANSI checksum of a byte array
 * Parameters: Byte array
 * Returns: CRC-16 ANSI checksum in byte array
 */
pub fn crc16_ansi(data: &[u8]) -> Vec<u8> {
    let mut crc = 0u16;
    let chunks = data.chunks_exact(2);
    let rem = chunks.remainder();

    chunks.for_each(|chunk| {
        crc ^= u16::from_le_bytes(chunk.try_into().unwrap());
        crc = CRC16T_ANSI[1][(crc & 0xff) as usize] ^
              CRC16T_ANSI[0][((crc >> 8) & 0xff) as usize];
    });

    rem.iter().for_each(|&byte| { crc = (crc >> 8) ^ CRC16T_ANSI[0][((crc ^ byte as u16) & 0xff) as usize]; });
    return crc.to_be_bytes().to_vec();
}