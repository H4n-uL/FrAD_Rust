/**                                PCM Format                                 */
/**
 * Copyright 2024 HaמuL
 * Description: Enum for PCM format
 */

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
        match self { PCMFormat::U8 | PCMFormat::U16(_) | PCMFormat::U24(_) | PCMFormat::U32(_) | PCMFormat::U64(_) => false, _ => true }
    }
    pub fn scale(&self) -> f64 {
        match self {
            PCMFormat::I8 | PCMFormat::U8 => 128.0,
            PCMFormat::I16(_) | PCMFormat::U16(_) => 32768.0,
            PCMFormat::I24(_) | PCMFormat::U24(_) => 8388608.0,
            PCMFormat::I32(_) | PCMFormat::U32(_) => 2147483648.0,
            PCMFormat::I64(_) | PCMFormat::U64(_) => 9223372036854775808.0,
            _ => 1.0
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Endian { Big, Little }