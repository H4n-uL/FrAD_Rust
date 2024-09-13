/**                             float64 Converter                             */
/**
 * Copyright 2024 HaמuL
 * Description: float64 <-> PCM format converter
 */

use crate::backend::{PCMFormat, Endian};
use half::f16;

/** norm_into
 * Normalise integer sample beteween -1.0 and 1.0
 * Parameters: Unnormalised sample, PCM format
 * Returns: Normalised sample
 */
fn norm_into(mut x: f64, pcm_fmt: &PCMFormat) -> f64 {
    return if pcm_fmt.float() { x }
    else {
        x /= pcm_fmt.scale();
        return if pcm_fmt.signed() { x } else { x - 1.0 };
    };
}

/** norm_from
 * Denormalise f64 sample to integer dynamic range
 * Parameters: Normalised sample, PCM format
 * Returns: Denormalised sample
 */
fn norm_from(mut x: f64, pcm_fmt: &PCMFormat) -> f64 {
    return if pcm_fmt.float() { x }
    else {
        x = if pcm_fmt.signed() { x } else { x + 1.0 };
        return (x * pcm_fmt.scale()).round();
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

/** macro! from_f64
 * Convert f64 to byte array with specified PCM format
 * Parameters: Type, f64, Endian
 * Returns: Byte array
 */
macro_rules! from_f64 {
    ($type:ty, $x:expr, $endian:expr) => {
        if $endian.eq(&Endian::Big) { <$type>::to_be_bytes($x) }
        else { <$type>::to_le_bytes($x) }
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

/** macro! int32_to_24
 * Convert 32-bit integer to 24-bit integer
 * Parameters: 32-bit integer, Endian, Signed flag
 * Returns: Byte array
 */
macro_rules! int32_to_24 {
    ($x:expr, $endian:expr, $signed:expr) => {{
        let (lo, hi) = if $signed { (-0x800000, 0x7fffff) } else { (0, 0xffffff) };
        let y = $x.max(lo).min(hi);
        if $endian.eq(&Endian::Big) { [(y >> 16) as u8, (y >> 8) as u8, y as u8] }
        else { [y as u8, (y >> 8) as u8, (y >> 16) as u8] }
    }};
}

/** any_to_f64
 * Convert single sample to f64 via PCM format
 * Parameters: Byte array, PCM format
 * Returns: f64
 */
pub fn any_to_f64(bytes: &[u8], pcm_fmt: &PCMFormat) -> f64 {
    if bytes.len() != pcm_fmt.bit_depth() / 8 { return 0.0 }
    return norm_into(
        match pcm_fmt {
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
        }, pcm_fmt
    );
}

/** f64_to_any
 * Convert f64 to single sample via PCM format
 * Parameters: f64, PCM format
 * Returns: Byte array
 */
pub fn f64_to_any(mut x: f64, pcm_fmt: &PCMFormat) -> Vec<u8> {
    x = norm_from(x, pcm_fmt);

    return match pcm_fmt {
        PCMFormat::F16(en) => from_f64!(f16, f16::from_f64(x), en).to_vec(),
        PCMFormat::F32(en) => from_f64!(f32, x as f32, en).to_vec(),
        PCMFormat::F64(en) => from_f64!(f64, x, en).to_vec(),

        PCMFormat::I8 => (x as i8).to_ne_bytes().to_vec(),
        PCMFormat::I16(en) => from_f64!(i16, x as i16, en).to_vec(),
        PCMFormat::I24(en) => int32_to_24!(x as i32, en, true).to_vec(),
        PCMFormat::I32(en) => from_f64!(i32, x as i32, en).to_vec(),
        PCMFormat::I64(en) => from_f64!(i64, x as i64, en).to_vec(),

        PCMFormat::U8 => (x as u8).to_ne_bytes().to_vec(),
        PCMFormat::U16(en) => from_f64!(u16, x as u16, en).to_vec(),
        PCMFormat::U24(en) => int32_to_24!(x as i32, en, false).to_vec(),
        PCMFormat::U32(en) => from_f64!(u32, x as u32, en).to_vec(),
        PCMFormat::U64(en) => from_f64!(u64, x as u64, en).to_vec(),
    };
}