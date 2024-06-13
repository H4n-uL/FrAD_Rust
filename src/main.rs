mod fourier;
mod tools;

// use libsoxr::Soxr;

fn main() {
    let pcm = vec![
        vec![0.0, 0.0],
        vec![-1.0, 1.0],
        vec![0.0, 0.0],
        vec![1.0, -1.0],
        vec![0.0, 0.0],
        vec![-1.0, 1.0],
        vec![0.0, 0.0],
        vec![1.0, -1.0],
    ];
    let channels = pcm[0].len() as i16;
    let bits = 32;
    let little_endian = false;
    let (frad, bits) = fourier::analogue(pcm, bits, little_endian);

    let stream = tools::ecc::encode_rs(frad, 96, 24);

    println!("{:?}", stream);
    // Transmission as bytestream

    let frad = tools::ecc::decode_rs(stream, 96, 24);
    let pcm = fourier::digital(frad, bits, channels, little_endian);
    println!("{:?}", pcm);

    // let srate = 44100.0;
    // let new_srate = 48000.0;
    // let soxr = Soxr::create(srate, new_srate, channels as u32, None, None, None).unwrap();
    // let mut target = vec![vec![0.0; channels as usize]; (pcm.len() as f64 * srate / new_srate) as usize];

    // let _ = soxr.process(Some(&pcm), &mut target);
    // soxr.process::<f64, _>(None, &mut target[0..]).unwrap();

    // println!("{:?}", target);
}