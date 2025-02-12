///                                PCM Format                                ///
///
/// Copyright 2024 HaמuL
/// Description: Enum for PCM format

#[derive(Clone, Copy)]
pub enum PCMFormat {
    F16BE, F16LE, F32BE, F32LE, F64BE, F64LE,
    S8, S16BE, S16LE, S24BE, S24LE, S32BE, S32LE, S64BE, S64LE,
    U8, U16BE, U16LE, U24BE, U24LE, U32BE, U32LE, U64BE, U64LE
}

impl PCMFormat {
    pub fn bit_depth(&self) -> usize {
        match self {
            Self::S8 | Self::U8 => 8,
            Self::F16BE | Self::F16LE | Self::S16BE | Self::S16LE | Self::U16BE | Self::U16LE => 16,
                                        Self::S24BE | Self::S24LE | Self::U24BE | Self::U24LE => 24,
            Self::F32BE | Self::F32LE | Self::S32BE | Self::S32LE | Self::U32BE | Self::U32LE => 32,
            Self::F64BE | Self::F64LE | Self::S64BE | Self::S64LE | Self::U64BE | Self::U64LE => 64
        }
    }
    pub fn float(&self) -> bool {
        match self { Self::F16BE | Self::F16LE | Self::F32BE | Self::F32LE | Self::F64BE | Self::F64LE => true, _ => false }
    }
    pub fn signed(&self) -> bool {
        match self { Self::U8 | Self::U16BE | Self::U16LE | Self::U24BE | Self::U24LE | Self::U32BE | Self::U32LE | Self::U64BE | Self::U64LE => false, _ => true }
    }
    pub fn scale(&self) -> f64 {
        match self {
            Self::S8 | Self::U8 => 128.0,
            Self::S16BE | Self::S16LE | Self::U16BE | Self::U16LE => 32768.0,
            Self::S24BE | Self::S24LE | Self::U24BE | Self::U24LE => 8388608.0,
            Self::S32BE | Self::S32LE | Self::U32BE | Self::U32LE => 2147483648.0,
            Self::S64BE | Self::S64LE | Self::U64BE | Self::U64LE => 9223372036854775808.0,
            _ => 1.0
        }
    }
}