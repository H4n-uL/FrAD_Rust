/**                               Common tools                                */
/**
 * Copyright 2024 HaמuL
 * Function: Common tools for FrAD
 */

use half::f16;
use std::{fs::File, io::{Read, Write}};

// signatures
pub const SIGNATURE: [u8; 4] = [0x66, 0x52, 0x61, 0x64];
pub const FRM_SIGN: [u8; 4] = [0xff, 0xd0, 0xd2, 0x97];

// Pipe and null device
pub static PIPEIN: &[&str] = &["pipe:", "pipe:0", "-", "/dev/stdin", "dev/fd/0"];
pub static PIPEOUT: &[&str] = &["pipe:", "pipe:1", "-", "/dev/stdout", "dev/fd/1"];


#[derive(Clone, Copy)]
pub enum PCMFormat {
    F16(Endian), F32(Endian), F64(Endian),
    I8, I16(Endian), I24(Endian), I32(Endian), I64(Endian),
    U8, U16(Endian), U24(Endian), U32(Endian), U64(Endian),
}

impl PCMFormat {
    pub fn bit_depth(&self) -> usize {
        match self {
            PCMFormat::I8 | PCMFormat::U8 => 8,
            PCMFormat::F16(_) | PCMFormat::I16(_) | PCMFormat::U16(_) => 16,
                                PCMFormat::I24(_) | PCMFormat::U24(_) => 24,
            PCMFormat::F32(_) | PCMFormat::I32(_) | PCMFormat::U32(_) => 32,
            PCMFormat::F64(_) | PCMFormat::I64(_) | PCMFormat::U64(_) => 64
        }
    }
    pub fn float(&self) -> bool {
        match self { PCMFormat::F16(_) | PCMFormat::F32(_) | PCMFormat::F64(_) => true, _ => false }
    }
    pub fn signed(&self) -> bool {
        match self { PCMFormat::I8 | PCMFormat::I16(_) | PCMFormat::I24(_) | PCMFormat::I32(_) | PCMFormat::I64(_) => true, _ => false }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Endian { Big, Little }

// CRC-32 Table generator
const fn gcrc32t() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let (mut crc, mut j) = (i as u32, 0);
        while j < 8 {
            if crc & 1 == 1 { crc = (crc >> 1) ^ 0xedb88320; } else { crc >>= 1; }
            j += 1;
        }
        (table[i], i) = (crc, i + 1);
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
 * Parameters: File(&mut), Buffer(&mut)
 * Returns: Total bytes read
 */
pub fn read_exact(file: &mut Box<dyn Read>, buf: &mut [u8]) -> usize {
    let mut total_read = 0;

    while total_read < buf.len() {
        let read_size = file.read(&mut buf[total_read..]).unwrap();
        if read_size == 0 { break; }
        total_read += read_size;
    }
    return total_read;
}

pub fn move_all(readfile: &mut File, writefile: &mut File, bufsize: usize) -> () {
    loop {
        let mut buf: Vec<u8> = vec![0; bufsize];
        let mut total_read = 0;

        while total_read < buf.len() {
            let read_size = readfile.read(&mut buf[total_read..]).unwrap();
            if read_size == 0 { break; }
            total_read += read_size;
        }
        if total_read == 0 { break; }
        writefile.write_all(&buf[..total_read]).unwrap();
    }
}

/** norm_into
 * Normalise integer sample beteween -1.0 and 1.0
 * Parameters: unnormalised sample, PCM format
 * Returns: Normalised sample
 */
fn norm_into(x: f64, pcm_fmt: &PCMFormat) -> f64 {
    return if pcm_fmt.float() { x }
    else {
        let y = x / 2.0f64.powi(pcm_fmt.bit_depth() as i32 - 1);
        return if pcm_fmt.signed() { y } else { y - 1.0 };
    };
}

/** macro! to_f64
 * Convert byte array to f64 with built-in Rust types
 * Parameters: Type, Byte array, Endian
 * Returns: f64
 */
macro_rules! to_f64 {
    ($type:ty, $bytes:expr, $endian:expr) => {
        if $endian.eq(&Endian::Big) { <$type>::from_be_bytes($bytes.try_into().unwrap()) }
        else {  <$type>::from_le_bytes($bytes.try_into().unwrap()) }
    };
}

/** macro! int24_to_32
 * Convert 24-bit integer to 32-bit integer
 * Parameters: Byte array, Endian, Signed flag
 * Returns: 32-bit integer
 */
macro_rules! int24_to_32 {
    ($bytes:expr, $endian:expr, $signed:expr) => {{
        let sign_bit = if $endian.eq(&Endian::Big) { $bytes[0] } else { $bytes[2] } & 0x80;
        let extra_byte = if !$signed || sign_bit == 0 { 0 } else { 0xFF };
        if $endian.eq(&Endian::Big) { [extra_byte, $bytes[0], $bytes[1], $bytes[2]] }
        else { [$bytes[0], $bytes[1], $bytes[2], extra_byte] }
    }};
}

/** any_to_f64
 * Convert single sample to f64 via PCM format
 * Parameters: Byte array, PCM format
 * Returns: f64
 */
pub fn any_to_f64(bytes: &[u8], pcm_fmt: &PCMFormat) -> f64 {
    return if bytes.len() != pcm_fmt.bit_depth() as usize / 8 { 0.0 }
    else {
        norm_into(match pcm_fmt {
            PCMFormat::F16(en) => to_f64!(f16, bytes, en).to_f64(),
            PCMFormat::F32(en) => to_f64!(f32, bytes, en) as f64,
            PCMFormat::F64(en) => to_f64!(f64, bytes, en),

            PCMFormat::I8 => i8::from_ne_bytes(bytes.try_into().unwrap()) as f64,
            PCMFormat::I16(en) => to_f64!(i16, bytes, en) as f64,
            PCMFormat::I24(en) => to_f64!(i32, int24_to_32!(bytes, en, true), en) as f64,
            PCMFormat::I32(en) => to_f64!(i32, bytes, en) as f64,
            PCMFormat::I64(en) => to_f64!(i64, bytes, en) as f64,

            PCMFormat::U8 => u8::from_ne_bytes(bytes.try_into().unwrap()) as f64,
            PCMFormat::U16(en) => to_f64!(u16, bytes, en) as f64,
            PCMFormat::U24(en) => to_f64!(u32, int24_to_32!(bytes, en, false), en) as f64,
            PCMFormat::U32(en) => to_f64!(u32, bytes, en) as f64,
            PCMFormat::U64(en) => to_f64!(u64, bytes, en) as f64,
        }, pcm_fmt)
    };
}