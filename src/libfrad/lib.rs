//!                          AAPM@Audio-8151 Library                         !//
//!
//! Copyright 2024-2025 HaמuL
//! Description: Fourier Analogue-in-Digital Rust Master Library

mod backend;
mod fourier;
mod tools;

mod encoder;
mod decoder;
mod repairer;

pub use fourier::{AVAILABLE, BIT_DEPTHS, SEGMAX, profiles};
pub use tools::head;

pub mod common;
pub use tools::asfh::ASFH;
pub use encoder::{Encoder, EncodeResult, EncoderParams};
pub use decoder::{Decoder, DecodeResult};
pub use repairer::Repairer;