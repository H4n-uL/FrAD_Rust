/**                             Bytearray packer                              */
/**
 * Copyright 2024 HaמuL
 * Function: Pack float array into byte array and vice versa
 * Dependencies: byteorder, half
 */

use byteorder::{BigEndian, LittleEndian, WriteBytesExt};
use half::f16;

use crate::backend::bitify;

/** cut_float3s
 * Cuts off last bits of floats to make their bit depth to 12, 24, or 48
 * Parameters:
 *      Bitstream, Bit depth divisable by 3
 * Returns: bitstream
 */
fn cut_float3s(bstr: Vec<bool>, bits: i16) -> Vec<bool> {
    return bstr.chunks(bits as usize * 4 / 3).flat_map(|c| c.iter().take(bits as usize)).cloned().collect();
}

/** pack
 * Makes Vec<f64> into byte array with specified bit depth and endianness
 * Parameters:
 *      Flat f64 array, Bit depth, Big endian toggle
 * Returns: Byte array
 */
pub fn pack(input: Vec<f64>, bits: i16, mut be: bool) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    if bits % 8 != 0 { be = true }

    if bits == 12 || bits == 16 {
        let input: Vec<f16> = input.iter().map(|&x| f16::from_f64(x)).collect();
        for &x in &input {
            if be { bytes.write_u16::<BigEndian>   (x.to_bits() as u16).unwrap(); }
            else  { bytes.write_u16::<LittleEndian>(x.to_bits() as u16).unwrap(); }
        }
    }
    else if bits == 24 || bits == 32 {
        for &x in &input {
            if be { bytes.write_f32::<BigEndian>   (x as f32).unwrap(); }
            else  { bytes.write_f32::<LittleEndian>(x as f32).unwrap(); }
        }
    }
    else if bits == 48 || bits == 64 {
        for &x in &input {
            if be { bytes.write_f64::<BigEndian>   (x).unwrap(); }
            else  { bytes.write_f64::<LittleEndian>(x).unwrap(); }
        }
    }

    if bits % 3 == 0 {
        let bitstrm: Vec<bool> = bitify::fromvec(bytes.clone());
        bytes = bitify::tovec(cut_float3s(bitstrm, bits));
    }

    return bytes;
}

/** pad_float3s
 * Pads floats to make them readable directly as 16, 32, or 64 bit floats
 * Parameters:
 *      Bitstream, Bit depth divisable by 3
 * Returns: bitstream
 */
fn pad_float3s(bstr: Vec<bool>, bits: i16) -> Vec<bool> {
    bstr.chunks(bits as usize).flat_map(|c| {
        let mut padded = Vec::from(c);
        padded.extend(std::iter::repeat(false).take(bits as usize / 3));
        return padded
    }).collect()
}

/** unpack
 * Makes byte array with specified bit depth and endianness into Vec<f64>
 * Parameters:
 *      Byte array, Bit depth, Big endian toggle
 * Returns: Flat f64 array
 */
pub fn unpack(mut input: Vec<u8>, bits: i16, mut be: bool) -> Vec<f64> {
    let mut vec: Vec<f64> = Vec::new();
    if bits % 8 != 0 { be = true }

    if bits % 3 == 0 {
        let mut bitstrm: Vec<bool> = bitify::fromvec(input.clone());
        bitstrm.truncate(bitstrm.len() - bitstrm.len() % bits as usize);
        input = bitify::tovec(pad_float3s(bitstrm, bits));
    }

    if bits == 12 || bits == 16 {
        vec = input
            .chunks(2)
            .map(|bytes| {
                let x = f16::from_bits(
                    if be { u16::from_be_bytes(bytes.try_into().unwrap()) }
                    else { u16::from_le_bytes(bytes.try_into().unwrap()) }
                );
                f64::from(x)
            })
            .collect();
    }
    else if bits == 24 || bits == 32 {
        vec = input
            .chunks(4)
            .map(|bytes| {
                let x =
                    if be { f32::from_be_bytes(bytes.try_into().unwrap()) }
                    else { f32::from_le_bytes(bytes.try_into().unwrap()) }
                ;
                f64::from(x)
            })
            .collect();
    }
    else if bits == 48 || bits == 64 {
        vec = input
            .chunks(8)
            .map(|bytes| {
                let x =
                    if be { f64::from_be_bytes(bytes.try_into().unwrap()) }
                    else { f64::from_le_bytes(bytes.try_into().unwrap()) }
                ;
                f64::from(x)
            })
            .collect();
    }

    return vec;
}