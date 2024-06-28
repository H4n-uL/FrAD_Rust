/**                               Common tools                                */
/**
 * Copyright 2024 HaמuL
 * Function: Common tools for FrAD
 */

use std::{fs::File, io::Read};

// Frame signature
pub const FRM_SIGN: [u8; 4] = [0xff, 0xd0, 0xd2, 0x97];

// Pipe and null device
pub static PIPEIN: &[&str] = &["pipe:0", "-"];
pub static PIPEOUT: &[&str] = &["pipe:1", "-"];
pub static DEVNULL: &str = if cfg!(windows) { "NUL" } else { "/dev/null" };

// CRC-32 Table generator
const fn gcrc32t() -> [u32; 256] {
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

// CRC-32 Table
const CRC32T: [u32; 256] = gcrc32t();

/** crc32
 * Calculates CRC-32 checksum of a byte array
 * Parameters: Byte array
 * Returns: CRC-32 checksum in byte array
 */
pub fn crc32(data: &[u8]) -> Vec<u8> {
    let mut crc = 0xffffffff;
    for &byte in data {
        crc = (crc >> 8) ^ CRC32T[((crc & 0xff) ^ byte as u32) as usize];
    }

    return (crc ^ 0xffffffff).to_be_bytes().to_vec();
}

// CRC-16 ANSI Table generator
const fn gcrc16t_ansi() -> [u16; 256] {
    let mut table = [0u16; 256];
    let mut i = 0;
    while i < 256 {
        let mut crc = i as u16;
        let mut j = 0;
        while j < 8 {
            crc = if crc & 0x0001 == 0x0001 { (crc >> 1) ^ 0xA001 } else { crc >> 1 };
            j += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}

// CRC-16 ANSI Table
const CRC16T_ANSI: [u16; 256] = gcrc16t_ansi();

/** crc16_ansi
 * Calculates CRC-16 ANSI checksum of a byte array
 * Parameters: Byte array
 * Returns: CRC-16 ANSI checksum in byte array
 */
pub fn crc16_ansi(data: &[u8]) -> Vec<u8> {
    let mut crc = 0u16;
    for &byte in data {
        crc = (crc >> 8) ^ CRC16T_ANSI[((crc ^ byte as u16) & 0xff) as usize];
    }
    return crc.to_be_bytes().to_vec();
}

/** read_exact
 * Reads a file or stdin to a buffer with exact size
 * Parameters: File(&mut), Buffer(&mut), Pipe flag
 * Returns: Total bytes read
 */
pub fn read_exact(file: &mut File, buf: &mut [u8], pipe: bool) -> usize {
    let mut total_read = 0;
    let mut stdin = std::io::stdin();

    let reader: &mut dyn Read = 
    if pipe { &mut stdin } else { file };

    while total_read < buf.len() {
        let read_size = reader.read(&mut buf[total_read..]).unwrap();
        if read_size == 0 { break; }
        total_read += read_size;
    }
    return total_read;
}