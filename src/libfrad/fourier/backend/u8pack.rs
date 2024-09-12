/**                             Bytearray packer                              */
/**
 * Copyright 2024 HaמuL
 * Description: Packer and unpacker for floats <-> byte arrays
 * Dependencies: byteorder, half
 */

use crate::backend::bitcvt;
use half::f16;

/** cut_float3s
 * Cuts off last bits of floats to make their bit depth to 12, 24, or 48
 * Parameters: Bitstream, Bit depth divisable by 3
 * Returns: bitstream
 */
fn cut_float3s(bstr: Vec<bool>, bits: usize, be: bool) -> Vec<bool> {
    return bstr.chunks(bits * 4 / 3).flat_map(|x| {
        x.iter().skip(if be { 0 } else { bits / 3 }).take(bits).cloned()
    }).collect();
}

/** pack
 * Makes Vec<f64> into byte array with specified bit depth and endianness
 * Parameters: Flat f64 array, Bit depth, Big endian toggle
 * Returns: Byte array
 */
pub fn pack(input: Vec<f64>, bits: i16, mut be: bool) -> Vec<u8> {
    let bits = bits as usize;
    if bits % 8 != 0 { be = true }

    let bytes = match bits {
        12 | 16 => pack_f16(input, be),
        24 | 32 => pack_f32(input, be),
        48 | 64 => pack_f64(input, be),
        _ => panic!("Invalid bit depth")
    };

    if bits % 3 == 0 { 
        return bitcvt::to_bytes(cut_float3s(bitcvt::to_bits(bytes), bits, be));
    }
    return bytes;
}

fn pack_f16(input: Vec<f64>, be: bool) -> Vec<u8> {
    return input.into_iter().flat_map(|x|
        if be { u16::to_be_bytes(f16::from_f64(x).to_bits()) }
        else  { u16::to_le_bytes(f16::from_f64(x).to_bits()) }
    ).collect();
}
fn pack_f32(input: Vec<f64>, be: bool) -> Vec<u8> {
    return input.into_iter().flat_map(|x|
        if be { f32::to_be_bytes(x as f32) }
        else  { f32::to_le_bytes(x as f32) }
    ).collect();
}
fn pack_f64(input: Vec<f64>, be: bool) -> Vec<u8> {
    return input.into_iter().flat_map(|x|
        if be { f64::to_be_bytes(x) }
        else  { f64::to_le_bytes(x) }
    ).collect();
}

/** pad_float3s
 * Pads floats to make them readable directly as 16, 32, or 64 bit floats
 * Parameters: Bitstream, Bit depth divisable by 3
 * Returns: bitstream
 */
fn pad_float3s(bstr: Vec<bool>, bits: usize, be: bool) -> Vec<bool> {
    return bstr.chunks(bits).flat_map(|y| {
        let pad = vec![false; bits / 3 * 4 - y.len()];
        if be { [Vec::from(y), pad] } else { [pad, Vec::from(y)] }.concat()
    }).collect();
}

/** unpack
 * Makes byte array with specified bit depth and endianness into Vec<f64>
 * Parameters: Byte array, Bit depth, Big endian toggle
 * Returns: Flat f64 array
 */
pub fn unpack(mut input: Vec<u8>, bits: i16, mut be: bool) -> Vec<f64> {
    let bits = bits as usize;

    if bits % 8 != 0 { be = true }
    if bits % 3 == 0 { input = bitcvt::to_bytes(pad_float3s(bitcvt::to_bits(input), bits, be)); }

    return match bits {
        12 | 16 => unpack_f16(input, be),
        24 | 32 => unpack_f32(input, be),
        48 | 64 => unpack_f64(input, be),
        _ => panic!("Invalid bit depth")
    };
}

fn unpack_f16(input: Vec<u8>, be: bool) -> Vec<f64> {
    return input.chunks(2)
    .map(|bytes| {
        f64::from(f16::from_bits(
            if be { u16::from_be_bytes(bytes.try_into().unwrap()) }
            else  { u16::from_le_bytes(bytes.try_into().unwrap()) }
        ))
    }).collect();
}
fn unpack_f32(input: Vec<u8>, be: bool) -> Vec<f64> {
    return input.chunks(4)
    .map(|bytes| {
        f64::from(
            if be { f32::from_be_bytes(bytes.try_into().unwrap()) }
            else  { f32::from_le_bytes(bytes.try_into().unwrap()) }
        )
    }).collect();
}
fn unpack_f64(input: Vec<u8>, be: bool) -> Vec<f64> {
    return input.chunks(8)
    .map(|bytes| {
        if be { f64::from_be_bytes(bytes.try_into().unwrap()) }
        else  { f64::from_le_bytes(bytes.try_into().unwrap()) }
    }).collect();
}