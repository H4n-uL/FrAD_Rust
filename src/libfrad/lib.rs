/**                          AAPM@Audio-8151 Library                          */
/**
 * Copyright 2024 HaמuL
 * Description: Fourier Analogue-in-Digital Rust Master Library
 */

mod backend;
mod fourier;
mod tools;

mod encoder;
mod decoder;
mod repairer;

pub use backend::{PCMFormat, Endian, f64cvt};
pub use fourier::{AVAILABLE, BIT_DEPTHS, SEGMAX, profiles};
pub use tools::{head, process::ProcessInfo};

pub mod common;
pub use tools::asfh::ASFH;
pub use encoder::Encoder;
pub use decoder::Decoder;
pub use repairer::Repairer;