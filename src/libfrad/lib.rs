/**                          AAPM@Audio-8151 Library                          */
/**
 * Copyright 2024 HaמuL
 * Description: Library for AAPM@Audio-8151(Fourier Analogue-in-Digital) codec
 */

mod backend;
mod fourier;
mod tools;

mod encode;
mod decode;
mod repair;

pub use backend::{PCMFormat, Endian, f64cvt};
pub use fourier::profiles;
pub use tools::{head, stream::StreamInfo};

pub mod common;
pub use encode::Encode;
pub use decode::Decode;
pub use repair::Repair;