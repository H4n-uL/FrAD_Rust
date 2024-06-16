mod backend;
mod fourier;
mod tools;
mod common;

mod encode;
mod decode;
mod reecc;

// use std::env;

fn main() {
    encode::encode();
    decode::decode();
    reecc::reecc();
}