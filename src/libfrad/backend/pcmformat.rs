//!                                PCM Format                                !//
//!
//! Copyright 2024-2025 HaמuL
//! Description: Enum for PCM format

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
}