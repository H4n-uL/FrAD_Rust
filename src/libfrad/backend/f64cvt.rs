///                             float64 Converter                            ///
///
/// Copyright 2024 HaמuL
/// Description: float64 <-> PCM format converter
/// Dependencies: half

use crate::PCMFormat;
use half::f16;

/// norm_into
/// Normalise integer sample beteween -1.0 and 1.0
/// Parameters: Unnormalised sample, PCM format
/// Returns: Normalised sample
fn norm_into(mut x: f64, pcm_fmt: &PCMFormat) -> f64 {
    return if pcm_fmt.float() { x }
    else {
        x /= pcm_fmt.scale();
        return if pcm_fmt.signed() { x } else { x - 1.0 };
    };
}

/// norm_from
/// Denormalise f64 sample to integer dynamic range
/// Parameters: Normalised sample, PCM format
/// Returns: Denormalised sample
fn norm_from(mut x: f64, pcm_fmt: &PCMFormat) -> f64 {
    return if pcm_fmt.float() { x }
    else {
        x = if pcm_fmt.signed() { x } else { x + 1.0 };
        return (x * pcm_fmt.scale()).round();
    };
}

/// i24_to_f64
/// Convert 24-bit integer to 64-bit float
/// Parameters: Byte array, Endian, Signed flag
/// Returns: 64-bit float
fn i24_to_f64(bytes: &[u8], little_endian: bool, signed: bool) -> f64 {
    let sign_bit = if little_endian { bytes[2] } else { bytes[0] } & 0x80;
    let mut buf = [if !signed || sign_bit == 0 { 0 } else { 0xFF }, 0, 0, 0];
    buf[1..4].copy_from_slice(&bytes[..]);
    if little_endian { buf[1..].reverse() }
    return i32::from_be_bytes(buf) as f64;
}

/// f64_to_i24
/// Convert 64-bit float to 24-bit integer
/// Parameters: 64-bit float, Endian, Signed flag
/// Returns: 24-bit integer Byte array
fn f64_to_i24(x: f64, little_endian: bool, signed: bool) -> [u8; 3] {
    let (lo, hi) = if signed { (-0x800000, 0x7fffff) } else { (0, 0xffffff) };
    let y = x.max(lo as f64).min(hi as f64) as i32;
    return if !little_endian { [(y >> 16) as u8, (y >> 8) as u8, y as u8] }
    else { [y as u8, (y >> 8) as u8, (y >> 16) as u8] };
}

/// any_to_f64
/// Convert single sample to f64 via PCM format
/// Parameters: Byte array, PCM format
/// Returns: f64
pub fn any_to_f64(bytes: &[u8], pcm_fmt: &PCMFormat) -> f64 {
    if bytes.len() != pcm_fmt.bit_depth() / 8 { return 0.0 }
    return norm_into(
        match pcm_fmt {
            PCMFormat::F16BE => f16::from_be_bytes(bytes.try_into().unwrap()).to_f64(),
            PCMFormat::F16LE => f16::from_le_bytes(bytes.try_into().unwrap()).to_f64(),
            PCMFormat::F32BE => f32::from_be_bytes(bytes.try_into().unwrap()) as f64,
            PCMFormat::F32LE => f32::from_le_bytes(bytes.try_into().unwrap()) as f64,
            PCMFormat::F64BE => f64::from_be_bytes(bytes.try_into().unwrap()),
            PCMFormat::F64LE => f64::from_le_bytes(bytes.try_into().unwrap()),

            PCMFormat::S8 => i8::from_ne_bytes(bytes.try_into().unwrap()) as f64,
            PCMFormat::S16BE => i16::from_be_bytes(bytes.try_into().unwrap()) as f64,
            PCMFormat::S16LE => i16::from_le_bytes(bytes.try_into().unwrap()) as f64,
            PCMFormat::S24BE => i24_to_f64(bytes, false, true),
            PCMFormat::S24LE => i24_to_f64(bytes, true, true),
            PCMFormat::S32BE => i32::from_be_bytes(bytes.try_into().unwrap()) as f64,
            PCMFormat::S32LE => i32::from_le_bytes(bytes.try_into().unwrap()) as f64,
            PCMFormat::S64BE => i64::from_be_bytes(bytes.try_into().unwrap()) as f64,
            PCMFormat::S64LE => i64::from_le_bytes(bytes.try_into().unwrap()) as f64,

            PCMFormat::U8 => u8::from_ne_bytes(bytes.try_into().unwrap()) as f64,
            PCMFormat::U16BE => u16::from_be_bytes(bytes.try_into().unwrap()) as f64,
            PCMFormat::U16LE => u16::from_le_bytes(bytes.try_into().unwrap()) as f64,
            PCMFormat::U24BE => i24_to_f64(bytes, false, false),
            PCMFormat::U24LE => i24_to_f64(bytes, true, false),
            PCMFormat::U32BE => u32::from_be_bytes(bytes.try_into().unwrap()) as f64,
            PCMFormat::U32LE => u32::from_le_bytes(bytes.try_into().unwrap()) as f64,
            PCMFormat::U64BE => u64::from_be_bytes(bytes.try_into().unwrap()) as f64,
            PCMFormat::U64LE => u64::from_le_bytes(bytes.try_into().unwrap()) as f64,
        }, pcm_fmt
    );
}

/// f64_to_any
/// Convert f64 to single sample via PCM format
/// Parameters: f64, PCM format
/// Returns: Byte array
pub fn f64_to_any(mut x: f64, pcm_fmt: &PCMFormat) -> Vec<u8> {
    x = norm_from(x, pcm_fmt);
    return match pcm_fmt {
        PCMFormat::F16BE => f16::from_f64(x).to_be_bytes().to_vec(),
        PCMFormat::F16LE => f16::from_f64(x).to_le_bytes().to_vec(),
        PCMFormat::F32BE => (x as f32).to_be_bytes().to_vec(),
        PCMFormat::F32LE => (x as f32).to_le_bytes().to_vec(),
        PCMFormat::F64BE => x.to_be_bytes().to_vec(),
        PCMFormat::F64LE => x.to_le_bytes().to_vec(),

        PCMFormat::S8 => (x as i8).to_ne_bytes().to_vec(),
        PCMFormat::S16BE => (x as i16).to_be_bytes().to_vec(),
        PCMFormat::S16LE => (x as i16).to_le_bytes().to_vec(),
        PCMFormat::S24BE => f64_to_i24(x, false, true).to_vec(),
        PCMFormat::S24LE => f64_to_i24(x, true, true).to_vec(),
        PCMFormat::S32BE => (x as i32).to_be_bytes().to_vec(),
        PCMFormat::S32LE => (x as i32).to_le_bytes().to_vec(),
        PCMFormat::S64BE => (x as i64).to_be_bytes().to_vec(),
        PCMFormat::S64LE => (x as i64).to_le_bytes().to_vec(),

        PCMFormat::U8 => (x as u8).to_ne_bytes().to_vec(),
        PCMFormat::U16BE => (x as u16).to_be_bytes().to_vec(),
        PCMFormat::U16LE => (x as u16).to_le_bytes().to_vec(),
        PCMFormat::U24BE => f64_to_i24(x, false, false).to_vec(),
        PCMFormat::U24LE => f64_to_i24(x, true, false).to_vec(),
        PCMFormat::U32BE => (x as u32).to_be_bytes().to_vec(),
        PCMFormat::U32LE => (x as u32).to_le_bytes().to_vec(),
        PCMFormat::U64BE => (x as u64).to_be_bytes().to_vec(),
        PCMFormat::U64LE => (x as u64).to_le_bytes().to_vec()
    };
}