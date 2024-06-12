mod fourier;
mod tools;

fn main() {
    let pcm = vec![
        vec![0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5],
        vec![0.0, -0.5, -1.0, -0.5, 0.0, 0.5, 1.0, 0.5]
    ];
    let bits = 32;
    let channels = pcm.len() as i16;
    let little_endian = false;
    let (frad, bits) = fourier::analogue(pcm, bits, little_endian);
    println!("{:?}", frad);

    let stream = tools::ecc::encode_rs(frad, 96, 24);

    // Transmission as bytestream

    let frad = tools::ecc::decode_rs(stream, 96, 24);
    let restored = fourier::digital(frad, bits, channels, little_endian);
    println!("{:?}", restored);
}