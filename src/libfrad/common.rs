//!                               Common tools                               !//
//!
//! Copyright 2024-2025 HaמuL
//! Description: Common tools for FrAD

// signatures
pub const SIGNATURE: [u8; 4] = [0x66, 0x52, 0x61, 0x64];
pub const FRM_SIGN: [u8; 4] = [0xff, 0xd0, 0xd2, 0x98];

// CRC Table sizes
const TABLE_SIZE_CRC32: usize = 16;
const TABLE_SIZE_CRC16_ANSI: usize = 16;

// CRC-32 Table generator
const fn gcrc32t() -> [[u32; 256]; TABLE_SIZE_CRC32] {
    let mut tables = [[0u32; 256]; TABLE_SIZE_CRC32];

    let mut i = 0; while i < 256 {
        let (mut crc, mut j) = (i as u32, 0);
        while j < 8 {
            crc = if crc & 1 == 1 { (crc >> 1) ^ 0xedb88320 } else { crc >> 1 };
        j += 1; }
        tables[0][i] = crc;
    i += 1; }

    let mut i = 0; while i < 256 {
        let mut j = 1; while j < TABLE_SIZE_CRC32 {
            tables[j][i] = (tables[j-1][i] >> 8) ^ tables[0][tables[j-1][i] as u8 as usize];
        j += 1; }
    i += 1; }

    return tables;
}

// CRC-16 ANSI Table generator
const fn gcrc16t_ansi() -> [[u16; 256]; TABLE_SIZE_CRC16_ANSI] {
    let mut tables = [[0u16; 256]; TABLE_SIZE_CRC16_ANSI];

    let mut i = 0; while i < 256 {
        let (mut crc, mut j) = (i as u16, 0);
        while j < 8 {
            crc = if crc & 1 == 1 { (crc >> 1) ^ 0xA001 } else { crc >> 1 };
        j += 1; }
        tables[0][i] = crc;
    i += 1; }

    let mut i = 0; while i < 256 {
        let mut j = 1; while j < TABLE_SIZE_CRC16_ANSI {
            tables[j][i] = (tables[j-1][i] >> 8) ^ tables[0][tables[j-1][i] as u8 as usize];
        j += 1; }
    i += 1; }

    return tables;
}

// CRC Tables
const CRC32_TABLE: [[u32; 256]; TABLE_SIZE_CRC32] = gcrc32t();
const CRC16T_ANSI: [[u16; 256]; TABLE_SIZE_CRC16_ANSI] = gcrc16t_ansi();

/// crc32_slow
/// Calculates CRC-32 checksum of a byte array
/// Parameters: Byte array
/// Returns: CRC-32 checksum
fn crc32_slow(mut crc: u32, buf: &[u8]) -> u32 {
    crc = !crc;
    buf.iter().for_each(|&byte| { crc = (crc >> 8) ^ CRC32_TABLE[0][((crc as u8) ^ byte) as usize]; });
    return !crc;
}

/// crc16_ansi_slow
/// Calculates CRC-16 ANSI checksum of a byte array
/// Parameters: Byte array
/// Returns: CRC-16 ANSI checksum
fn crc16_ansi_slow(mut crc: u16, buf: &[u8]) -> u16 {
    buf.iter().for_each(|&byte| { crc = (crc >> 8) ^ CRC16T_ANSI[0][((crc as u8) ^ byte) as usize]; });
    return crc;
}

/// crc32
/// Accelerated CRC-32 checksum calculation
/// Parameters: Byte array
/// Returns: CRC-32 checksum
pub fn crc32(mut crc: u32, buf: &[u8]) -> u32 {
    if TABLE_SIZE_CRC32 < 4 { return crc32_slow(crc, buf); }
    crc = !crc;

    buf.chunks(TABLE_SIZE_CRC32).for_each(|chunk| {
        if chunk.len() < TABLE_SIZE_CRC32 { crc = !crc32_slow(!crc, chunk); return; }
        let mut crc_temp = 0u32;
        for i in (0..TABLE_SIZE_CRC32).rev() {
            if i < 4 { crc_temp ^= CRC32_TABLE[TABLE_SIZE_CRC32 - i - 1][chunk[i] as usize ^ ((crc >> (i * 8)) & 0xFF) as usize]; }
            else { crc_temp ^= CRC32_TABLE[TABLE_SIZE_CRC32 - i - 1][chunk[i] as usize]; }
        }
        crc = crc_temp;
    });

    return !crc;
}

/// crc16_ansi
/// Accelerated CRC-16 ANSI checksum calculation
/// Parameters: Byte array
/// Returns: CRC-16 ANSI checksum
pub fn crc16_ansi(mut crc: u16, buf: &[u8]) -> u16 {
    if TABLE_SIZE_CRC16_ANSI < 2 { return crc16_ansi_slow(crc, buf); }

    buf.chunks(TABLE_SIZE_CRC16_ANSI).for_each(|chunk| {
        if chunk.len() < TABLE_SIZE_CRC16_ANSI { crc = crc16_ansi_slow(crc, chunk); return; }
        let mut crc_temp = 0u16;
        for i in (0..TABLE_SIZE_CRC16_ANSI).rev() {
            if i < 2 { crc_temp ^= CRC16T_ANSI[TABLE_SIZE_CRC16_ANSI - i - 1][chunk[i] as usize ^ ((crc >> (i * 8)) as u8 as usize)]; }
            else { crc_temp ^= CRC16T_ANSI[TABLE_SIZE_CRC16_ANSI - i - 1][chunk[i] as usize]; }
        }
        crc = crc_temp;
    });

    return crc;
}
