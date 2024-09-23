/**                          AAPM@Audio-8151 Library                          */
/**
 * Copyright 2024 HaמuL
 * Description: Library for AAPM@Audio-8151(Fourier Analogue-in-Digital) codec
 */

mod backend;
mod fourier;
mod tools;

mod encoder;
mod decoder;
mod repairer;

pub use backend::{PCMFormat, Endian, f64cvt};
pub use fourier::profiles;
pub use tools::{head, stream::StreamInfo};

pub mod common;
pub use encoder::Encoder;
pub use decoder::Decoder;
pub use repairer::Repairer;