use super::super::backend::core_fast::dct;
use super::tools::p1tools;
use half::f16;

pub const SRATES: [u32; 12] = [96000, 88200, 64000, 48000, 44100, 32000, 24000, 22050, 16000, 12000, 11025, 8000];
pub const SMPLS: [(u32, [u32; 8]); 3] = [
    (128, [128, 256, 512, 1024, 2048, 4096, 8192, 16384]),
    (144, [144, 288, 576, 1152, 2304, 4608, 9216, 18432]),
    (192, [192, 384, 768, 1536, 3072, 6144, 12288, 24576]),
];

pub fn get_smpls_from_value(key: &u32) -> u32 {
    SMPLS.iter().find(|&(_, v)| v.iter().find(|&&x| x == *key).is_some()).unwrap().0
}
pub const SMPLS_LI: [u32; 24] = [
    128, 144, 192,
    256, 288, 384,
    512, 576, 768,
    1024, 1152, 1536,
    2048, 2304, 3072,
    4096, 4608, 6144,
    8192, 9216, 12288,
    16384, 18432, 24576,
];

pub const DEPTHS: [i16; 7] = [8, 12, 16, 24, 32, 48, 64];

fn pad_pcm(mut pcm: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let len_smpl = pcm.len();
    let chnl = pcm[0].len();
    let pad_len = *SMPLS_LI.iter().find(|&&x| x as usize >= len_smpl).unwrap_or(&(len_smpl as u32)) as usize - len_smpl;

    pcm.extend(std::iter::repeat(vec![0.0; chnl]).take(pad_len));
    return pcm;
}

pub fn analogue(pcm: Vec<Vec<f64>>, bits: i16, srate: u32, level: u8) -> (Vec<u8>, i16) {
    let pcm = pad_pcm(pcm);
    let pcm_trans: Vec<Vec<f64>> = (0..pcm[0].len())
        .map(|i| pcm.iter().map(|inner| inner[i] * 2.0_f64.powf((bits-1) as f64) / pcm.len() as f64).collect())
        .collect();

    let freqs: Vec<Vec<f64>> = pcm_trans.iter().map(|x| dct(x.to_vec())).collect();

    let (freqs, pns) = p1tools::quant(freqs, pcm[0].len() as i16, srate, level);

    let freqs_flat: Vec<i64> = (0..freqs[0].len())
        .map(|i| freqs.iter().map(|inner| inner[i]).collect::<Vec<i64>>())
        .into_iter().flatten().collect();

    let pns_flat: Vec<i64> = (0..pns[0].len())
        .map(|i| pns.iter().map(|inner| f16::from_f64(inner[i] / 2_i64.pow(bits as u32-1) as f64).to_bits() as i64 ).collect::<Vec<i64>>())
        .into_iter().flatten().collect();

    let pns_glm = p1tools::exp_golomb_rice_encode(pns_flat);
    let freqs_glm = p1tools::exp_golomb_rice_encode(freqs_flat);

    let frad: Vec<u8> = (pns_glm.len() as u32).to_be_bytes().to_vec()
        .into_iter().chain(pns_glm.into_iter())
        .chain(freqs_glm.into_iter()).collect();

    // TODO: Implement zlib compression

    return (frad, DEPTHS.iter().position(|&x| x == bits).unwrap() as i16);
}