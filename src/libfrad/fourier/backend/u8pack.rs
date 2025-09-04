//!                             Bytearray packer                             !//
//!
//! Copyright 2024-2025 HaמuL
//! Description: Packer and unpacker for floats <-> byte arrays
//! Dependencies: half

use crate::backend::bitcvt;
use alloc::vec::Vec;
use half::f16;

/// cut_float3s
/// Cuts off last bits of floats to make their bit depth to 12, 24, or 48
/// Parameters: Bitstream, Bit depth divisable by 3, Little endian toggle
/// Returns: bitstream
fn cut_float3s(bytes: Vec<u8>, bits: usize, little_endian: bool) -> Vec<u8> {
    let size = if bits % 8 == 0 { bits / 8 } else { bits };
    let skip = if !little_endian { 0 } else { size / 3 };

    return if bits % 8 != 0 {
        bitcvt::to_bytes(&bitcvt::to_bits(&bytes)
        .chunks(size * 4 / 3).flat_map(|x| x.iter().skip(skip).take(size).cloned()).collect::<Vec<bool>>()
    )}
    else { bytes.chunks(size * 4 / 3).flat_map(|x| x.iter().skip(skip).take(size).cloned()).collect() }
}

/// pack
/// Makes Vec<f64> into byte array with specified bit depth and endianness
/// Parameters: Flat f64 array, Bit depth, Little endian toggle
/// Returns: Byte array
pub fn pack(input: Vec<f64>, bits: u16, mut little_endian: bool) -> Vec<u8> {
    let bits = bits as usize;
    if bits % 8 != 0 { little_endian = false }

    let bytes = match bits {
        12 | 16 => pack_f16(input, little_endian),
        24 | 32 => pack_f32(input, little_endian),
        48 | 64 => pack_f64(input, little_endian),
        _ => panic!("Invalid bit depth")
    };

    if bits % 3 == 0 {
        return cut_float3s(bytes, bits, little_endian);
    }
    return bytes;
}

fn pack_f16(input: Vec<f64>, little_endian: bool) -> Vec<u8> {
    return input.into_iter().flat_map(|x|
        if !little_endian { u16::to_be_bytes(f16::from_f64(x).to_bits()) }
        else { u16::to_le_bytes(f16::from_f64(x).to_bits()) }
    ).collect();
}
fn pack_f32(input: Vec<f64>, little_endian: bool) -> Vec<u8> {
    return input.into_iter().flat_map(|x|
        if !little_endian { f32::to_be_bytes(x as f32) }
        else { f32::to_le_bytes(x as f32) }
    ).collect();
}
fn pack_f64(input: Vec<f64>, little_endian: bool) -> Vec<u8> {
    return input.into_iter().flat_map(|x|
        if !little_endian { f64::to_be_bytes(x) }
        else { f64::to_le_bytes(x) }
    ).collect();
}

/// pad_float3s
/// Pads floats to make them readable directly as 16, 32, or 64 bit floats
/// Parameters: Bitstream, Bit depth divisable by 3, Little endian toggle
/// Returns: bitstream
fn pad_float3s(bstr: Vec<u8>, bits: usize, little_endian: bool) -> Vec<u8> {
    let (pad_bits, pad_bytes) = (alloc::vec![false; bits / 3], alloc::vec![0; bits / 24]);
    return if bits % 8 != 0 {
        bitcvt::to_bytes(&bitcvt::to_bits(&bstr)
        .chunks(bits).filter(|x| x.len() == bits).flat_map(|x| {
        if !little_endian { x.iter().chain(pad_bits.iter()) } else { pad_bits.iter().chain(x.iter()) }
        }).cloned().collect::<Vec<bool>>())
    }
    else {
        bstr.chunks(bits / 8).filter(|x| x.len() == bits / 8).flat_map(|x| {
        if !little_endian { x.iter().chain(pad_bytes.iter()) } else { pad_bytes.iter().chain(x.iter()) }
        }).cloned().collect()
    }
}

/// unpack
/// Makes byte array with specified bit depth and endianness into Vec<f64>
/// Parameters: Byte array, Bit depth, Little endian toggle
/// Returns: Flat f64 array
pub fn unpack(mut input: Vec<u8>, bits: u16, mut little_endian: bool) -> Vec<f64> {
    let bits = bits as usize;

    if bits % 8 != 0 { little_endian = false }
    if bits % 3 == 0 { input = pad_float3s(input, bits, little_endian); }

    return match bits {
        12 | 16 => unpack_f16(input, little_endian),
        24 | 32 => unpack_f32(input, little_endian),
        48 | 64 => unpack_f64(input, little_endian),
        _ => panic!("Invalid bit depth")
    };
}

fn unpack_f16(input: Vec<u8>, little_endian: bool) -> Vec<f64> {
    return input.chunks(2)
    .map(|bytes| {
        f64::from(f16::from_bits(
            if !little_endian { u16::from_be_bytes(bytes.try_into().unwrap()) }
            else { u16::from_le_bytes(bytes.try_into().unwrap()) }
        ))
    }).collect();
}
fn unpack_f32(input: Vec<u8>, little_endian: bool) -> Vec<f64> {
    return input.chunks(4)
    .map(|bytes| {
        f64::from(
            if !little_endian { f32::from_be_bytes(bytes.try_into().unwrap()) }
            else { f32::from_le_bytes(bytes.try_into().unwrap()) }
        )
    }).collect();
}
fn unpack_f64(input: Vec<u8>, little_endian: bool) -> Vec<f64> {
    return input.chunks(8)
    .map(|bytes| {
        if !little_endian { f64::from_be_bytes(bytes.try_into().unwrap()) }
        else { f64::from_le_bytes(bytes.try_into().unwrap()) }
    }).collect();
}