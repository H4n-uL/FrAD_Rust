mod backend;
mod fourier;
mod tools;
mod common;

mod encode;
mod decode;

// use std::env;

fn main() {
    encode::encode();
    decode::decode();
}