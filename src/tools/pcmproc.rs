//!                               PCM Processor                              !//
//!
//! Copyright 2024-2025 HaמuL
//! Description: PCM processing utilities
//! Dependencies: half

use half::f16;

#[derive(Clone, Copy)]
pub enum PCMFormat {
    F16BE = 16 << 3 | 0b010,
    F16LE = 16 << 3 | 0b011,
    F32BE = 32 << 3 | 0b010,
    F32LE = 32 << 3 | 0b011,
    F64BE = 64 << 3 | 0b010,
    F64LE = 64 << 3 | 0b011,

    S8    =  8 << 3 | 0b110,
    S16BE = 16 << 3 | 0b110,
    S16LE = 16 << 3 | 0b111,
    S24BE = 24 << 3 | 0b110,
    S24LE = 24 << 3 | 0b111,
    S32BE = 32 << 3 | 0b110,
    S32LE = 32 << 3 | 0b111,
    S64BE = 64 << 3 | 0b110,
    S64LE = 64 << 3 | 0b111,

    U8    =  8 << 3 | 0b100,
    U16BE = 16 << 3 | 0b100,
    U16LE = 16 << 3 | 0b101,
    U24BE = 24 << 3 | 0b100,
    U24LE = 24 << 3 | 0b101,
    U32BE = 32 << 3 | 0b100,
    U32LE = 32 << 3 | 0b101,
    U64BE = 64 << 3 | 0b100,
    U64LE = 64 << 3 | 0b101
}

impl PCMFormat {
    pub fn bit_depth(&self) -> usize {
        *self as usize >> 3
    }
    pub fn float(&self) -> bool {
        *self as usize & 0b100 == 0
    }
    pub fn signed(&self) -> bool {
        *self as usize & 0b010 != 0
    }
    pub fn scale(&self) -> f64 {
        return if self.float() {
            1.0
        } else {
            2.0_f64.powf(self.bit_depth() as f64 - 1.0)
        };
    }

    /// norm_into
    /// Normalise integer sample beteween -1.0 and 1.0
    /// Parameters: Unnormalised sample, PCM format
    /// Returns: Normalised sample
    fn norm_into(&self, mut x: f64) -> f64 {
        return if self.float() { x }
        else {
            x /= self.scale();
            return if self.signed() { x } else { x - 1.0 };
        };
    }

    /// norm_from
    /// Denormalise f64 sample to integer dynamic range
    /// Parameters: Normalised sample, PCM format
    /// Returns: Denormalised sample
    fn norm_from(&self, mut x: f64) -> f64 {
        return if self.float() { x }
        else {
            x = if self.signed() { x } else { x + 1.0 };
            return (x * self.scale()).round();
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
    pub fn any_to_f64(&self, bytes: &[u8]) -> f64 {
        if bytes.len() != self.bit_depth() / 8 { return 0.0 }
        return self.norm_into(
            match self {
                PCMFormat::F16BE => f16::from_be_bytes(bytes.try_into().unwrap()).to_f64(),
                PCMFormat::F16LE => f16::from_le_bytes(bytes.try_into().unwrap()).to_f64(),
                PCMFormat::F32BE => f32::from_be_bytes(bytes.try_into().unwrap()) as f64,
                PCMFormat::F32LE => f32::from_le_bytes(bytes.try_into().unwrap()) as f64,
                PCMFormat::F64BE => f64::from_be_bytes(bytes.try_into().unwrap()),
                PCMFormat::F64LE => f64::from_le_bytes(bytes.try_into().unwrap()),

                PCMFormat::S8 => i8::from_ne_bytes(bytes.try_into().unwrap()) as f64,
                PCMFormat::S16BE => i16::from_be_bytes(bytes.try_into().unwrap()) as f64,
                PCMFormat::S16LE => i16::from_le_bytes(bytes.try_into().unwrap()) as f64,
                PCMFormat::S24BE => Self::i24_to_f64(bytes, false, true),
                PCMFormat::S24LE => Self::i24_to_f64(bytes, true, true),
                PCMFormat::S32BE => i32::from_be_bytes(bytes.try_into().unwrap()) as f64,
                PCMFormat::S32LE => i32::from_le_bytes(bytes.try_into().unwrap()) as f64,
                PCMFormat::S64BE => i64::from_be_bytes(bytes.try_into().unwrap()) as f64,
                PCMFormat::S64LE => i64::from_le_bytes(bytes.try_into().unwrap()) as f64,

                PCMFormat::U8 => u8::from_ne_bytes(bytes.try_into().unwrap()) as f64,
                PCMFormat::U16BE => u16::from_be_bytes(bytes.try_into().unwrap()) as f64,
                PCMFormat::U16LE => u16::from_le_bytes(bytes.try_into().unwrap()) as f64,
                PCMFormat::U24BE => Self::i24_to_f64(bytes, false, false),
                PCMFormat::U24LE => Self::i24_to_f64(bytes, true, false),
                PCMFormat::U32BE => u32::from_be_bytes(bytes.try_into().unwrap()) as f64,
                PCMFormat::U32LE => u32::from_le_bytes(bytes.try_into().unwrap()) as f64,
                PCMFormat::U64BE => u64::from_be_bytes(bytes.try_into().unwrap()) as f64,
                PCMFormat::U64LE => u64::from_le_bytes(bytes.try_into().unwrap()) as f64,
            }
        );
    }

    /// f64_to_any
    /// Convert f64 to single sample via PCM format
    /// Parameters: f64, PCM format
    /// Returns: Byte array
    pub fn f64_to_any(&self, mut x: f64) -> Vec<u8> {
        x = self.norm_from(x);
        return match self {
            PCMFormat::F16BE => f16::from_f64(x).to_be_bytes().to_vec(),
            PCMFormat::F16LE => f16::from_f64(x).to_le_bytes().to_vec(),
            PCMFormat::F32BE => (x as f32).to_be_bytes().to_vec(),
            PCMFormat::F32LE => (x as f32).to_le_bytes().to_vec(),
            PCMFormat::F64BE => x.to_be_bytes().to_vec(),
            PCMFormat::F64LE => x.to_le_bytes().to_vec(),

            PCMFormat::S8 => (x as i8).to_ne_bytes().to_vec(),
            PCMFormat::S16BE => (x as i16).to_be_bytes().to_vec(),
            PCMFormat::S16LE => (x as i16).to_le_bytes().to_vec(),
            PCMFormat::S24BE => Self::f64_to_i24(x, false, true).to_vec(),
            PCMFormat::S24LE => Self::f64_to_i24(x, true, true).to_vec(),
            PCMFormat::S32BE => (x as i32).to_be_bytes().to_vec(),
            PCMFormat::S32LE => (x as i32).to_le_bytes().to_vec(),
            PCMFormat::S64BE => (x as i64).to_be_bytes().to_vec(),
            PCMFormat::S64LE => (x as i64).to_le_bytes().to_vec(),

            PCMFormat::U8 => (x as u8).to_ne_bytes().to_vec(),
            PCMFormat::U16BE => (x as u16).to_be_bytes().to_vec(),
            PCMFormat::U16LE => (x as u16).to_le_bytes().to_vec(),
            PCMFormat::U24BE => Self::f64_to_i24(x, false, false).to_vec(),
            PCMFormat::U24LE => Self::f64_to_i24(x, true, false).to_vec(),
            PCMFormat::U32BE => (x as u32).to_be_bytes().to_vec(),
            PCMFormat::U32LE => (x as u32).to_le_bytes().to_vec(),
            PCMFormat::U64BE => (x as u64).to_be_bytes().to_vec(),
            PCMFormat::U64LE => (x as u64).to_le_bytes().to_vec()
        };
    }
}

pub struct PCMProcessor {
    fmt: PCMFormat,
    buffer: Vec<u8>
}

impl PCMProcessor {
    pub fn new(fmt: PCMFormat) -> Self {
        Self { fmt, buffer: Vec::new() }
    }

    pub fn from_f64(&self, samples: &[f64]) -> Vec<u8> {
        samples.iter().flat_map(|&s| self.fmt.f64_to_any(s)).collect()
    }

    pub fn into_f64(&mut self, samples: &[u8]) -> Vec<f64> {
        self.buffer.extend_from_slice(samples);
        let mut chunks = self.buffer.chunks_exact(self.fmt.bit_depth() / 8);
        let result = chunks.by_ref().map(|b| self.fmt.any_to_f64(b)).collect();
        self.buffer = chunks.remainder().to_vec();
        result
    }
}
