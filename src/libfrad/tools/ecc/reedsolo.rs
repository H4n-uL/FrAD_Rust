//!                         Reed-Solomon ECC for Rust                        !//
//!
//! Ported from https://github.com/tomerfiliba-org/reedsolomon
//! Copyright - Tomer Filiba

use core::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    result::Result as CoreResult
};

use alloc::{format, string::{String, ToString}, vec::Vec};

#[derive(Clone, Debug)]
pub struct ReedSolomonError {
    pub message: String
}

impl Display for ReedSolomonError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "ReedSolomonError: {}", self.message)
    }
}

impl Error for ReedSolomonError {}
type Result<T> = CoreResult<T, ReedSolomonError>;

#[derive(Clone)]
struct GFContext {
    gf_exp: Vec<u8>,
    gf_log: Vec<u8>,
    field_charac: usize
}

impl GFContext {
    fn new(prim: usize, generator: usize, c_exp: usize) -> Self {
        let field_charac = (1 << c_exp) - 1;
        let mut gf_exp = alloc::vec![0u8; field_charac * 2];
        let mut gf_log = alloc::vec![0u8; field_charac + 1];

        let mut x = 1;
        for i in 0..field_charac {
            gf_exp[i] = x as u8;
            gf_log[x] = i as u8;
            x = gf_mult_nolut(x, generator, prim, field_charac + 1);
        }

        for i in field_charac..(field_charac * 2) {
            gf_exp[i] = gf_exp[i - field_charac];
        }

        return Self {
            gf_exp,
            gf_log,
            field_charac
        };
    }

    // fn gf_add(&self, x: u8, y: u8) -> u8 {
    //     return x ^ y;
    // }

    fn gf_sub(&self, x: u8, y: u8) -> u8 {
        return x ^ y;
    }

    fn gf_mul(&self, x: u8, y: u8) -> u8 {
        if x == 0 || y == 0 {
            return 0;
        }
        return self.gf_exp[((self.gf_log[x as usize] as usize + self.gf_log[y as usize] as usize) % self.field_charac) as usize];
    }

    fn gf_div(&self, x: u8, y: u8) -> Result<u8> {
        if y == 0 {
            return Err(ReedSolomonError {
                message: "Division by zero".to_string()
            });
        }
        if x == 0 {
            return Ok(0);
        }
        return Ok(self.gf_exp[((self.gf_log[x as usize] as usize + self.field_charac - self.gf_log[y as usize] as usize) % self.field_charac) as usize]);
    }

    fn gf_pow(&self, x: u8, power: usize) -> u8 {
        if x == 0 {
            return 0;
        }
        return self.gf_exp[((self.gf_log[x as usize] as usize * power) % self.field_charac) as usize];
    }

    fn gf_inverse(&self, x: u8) -> u8 {
        return self.gf_exp[self.field_charac - self.gf_log[x as usize] as usize];
    }

    // Polynomial operations
    fn gf_poly_scale(&self, p: &[u8], x: u8) -> Vec<u8> {
        return p.iter().map(|&pi| self.gf_mul(pi, x)).collect();
    }

    fn gf_poly_add(&self, p: &[u8], q: &[u8]) -> Vec<u8> {
        let max_len = p.len().max(q.len());
        let mut r = alloc::vec![0u8; max_len];

        let p_offset = max_len - p.len();
        let q_offset = max_len - q.len();

        for i in 0..p.len() {
            r[i + p_offset] = p[i];
        }

        for i in 0..q.len() {
            r[i + q_offset] ^= q[i];
        }

        return r;
    }

    fn gf_poly_mul(&self, p: &[u8], q: &[u8]) -> Vec<u8> {
        let mut r = alloc::vec![0u8; p.len() + q.len() - 1];

        let lp: Vec<u8> = p.iter().map(|&x| if x != 0 { self.gf_log[x as usize] } else { 0 }).collect();

        for j in 0..q.len() {
            let qj = q[j];
            if qj != 0 {
                let lq = self.gf_log[qj as usize];
                for i in 0..p.len() {
                    if p[i] != 0 {
                        r[i + j] ^= self.gf_exp[(lp[i] as usize + lq as usize) as usize];
                    }
                }
            }
        }
        return r;
    }

    fn gf_poly_eval(&self, poly: &[u8], x: u8) -> u8 {
        let mut y = poly[0];
        for i in 1..poly.len() {
            y = self.gf_mul(y, x) ^ poly[i];
        }
        return y;
    }
}

fn gf_mult_nolut(mut x: usize, mut y: usize, prim: usize, field_charac_full: usize) -> usize {
    let mut r = 0;
    while y > 0 {
        if y & 1 != 0 {
            r ^= x;
        }
        y >>= 1;
        x <<= 1;
        if prim > 0 && x & field_charac_full != 0 {
            x ^= prim;
        }
    }
    return r;
}

fn rs_generator_poly(ctx: &GFContext, nsym: usize, fcr: usize, generator: usize) -> Vec<u8> {
    let mut g = alloc::vec![1u8];
    for i in 0..nsym {
        g = ctx.gf_poly_mul(&g, &[1, ctx.gf_pow(generator as u8, i + fcr)]);
    }
    return g;
}

fn rs_encode_msg(ctx: &GFContext, msg_in: &[u8], nsym: usize, fcr: usize, generator: usize, gen_poly: Option<&[u8]>) -> Result<Vec<u8>> {
    if msg_in.len() + nsym > ctx.field_charac {
        return Err(ReedSolomonError {
            message: format!("Message is too long ({} when max is {})", msg_in.len() + nsym, ctx.field_charac)
        });
    }

    let gen_poly = match gen_poly {
        Some(g) => g.to_vec(),
        None => rs_generator_poly(ctx, nsym, fcr, generator)
    };

    let mut msg_out = msg_in.to_vec();
    msg_out.extend(alloc::vec![0u8; gen_poly.len() - 1]);

    let lgen: Vec<u8> = gen_poly.iter().map(|&x| if x != 0 { ctx.gf_log[x as usize] } else { 0 }).collect();

    for i in 0..msg_in.len() {
        let coef = msg_out[i];
        if coef != 0 {
            let lcoef = ctx.gf_log[coef as usize];
            for j in 1..gen_poly.len() {
                msg_out[i + j] ^= ctx.gf_exp[(lcoef as usize + lgen[j] as usize) as usize];
            }
        }
    }

    for i in 0..msg_in.len() {
        msg_out[i] = msg_in[i];
    }

    return Ok(msg_out);
}

fn rs_calc_syndromes(ctx: &GFContext, msg: &[u8], nsym: usize, fcr: usize, generator: usize) -> Vec<u8> {
    let mut synd = alloc::vec![0u8; nsym + 1];
    for i in 0..nsym {
        synd[i + 1] = ctx.gf_poly_eval(msg, ctx.gf_pow(generator as u8, i + fcr));
    }
    return synd;
}

fn rs_find_error_locator(ctx: &GFContext, synd: &[u8], nsym: usize, erase_loc: Option<&[u8]>, erase_count: usize) -> Result<Vec<u8>> {
    let (mut err_loc, mut old_loc) = match erase_loc {
        Some(loc) => (loc.to_vec(), loc.to_vec()),
        None => (alloc::vec![1u8], alloc::vec![1u8])
    };

    let synd_shift = if synd.len() > nsym { synd.len() - nsym } else { 0 };

    for i in 0..(nsym - erase_count) {
        let k = if erase_loc.is_some() {
            erase_count + i + synd_shift
        } else {
            i + synd_shift
        };

        let mut delta = synd[k];
        for j in 1..err_loc.len() {
            delta ^= ctx.gf_mul(err_loc[err_loc.len() - j - 1], synd[k - j]);
        }

        old_loc.push(0);

        if delta != 0 {
            if old_loc.len() > err_loc.len() {
                let new_loc = ctx.gf_poly_scale(&old_loc, delta);
                old_loc = ctx.gf_poly_scale(&err_loc, ctx.gf_inverse(delta));
                err_loc = new_loc;
            }
            err_loc = ctx.gf_poly_add(&err_loc, &ctx.gf_poly_scale(&old_loc, delta));
        }
    }

    while err_loc.len() > 0 && err_loc[0] == 0 {
        err_loc.remove(0);
    }

    let errs = err_loc.len() - 1;
    if (errs - erase_count) * 2 + erase_count > nsym {
        return Err(ReedSolomonError {
            message: "Too many errors to correct".to_string()
        });
    }

    return Ok(err_loc);
}

fn rs_find_errata_locator(ctx: &GFContext, e_pos: &[usize], generator: usize) -> Vec<u8> {
    let mut e_loc = alloc::vec![1u8];
    for &i in e_pos {
        let poly = alloc::vec![1u8, ctx.gf_pow(generator as u8, i), 0];
        e_loc = ctx.gf_poly_mul(&e_loc, &ctx.gf_poly_add(&[1], &poly[1..]));
    }
    return e_loc;
}

fn rs_find_error_evaluator(ctx: &GFContext, synd: &[u8], err_loc: &[u8], nsym: usize) -> Vec<u8> {
    let remainder = ctx.gf_poly_mul(synd, err_loc);
    let start = if remainder.len() > nsym + 1 {
        remainder.len() - (nsym + 1)
    } else {
        0
    };
    return remainder[start..].to_vec();
}

fn rs_find_errors(ctx: &GFContext, err_loc: &[u8], nmess: usize, generator: usize) -> Result<Vec<usize>> {
    let mut err_pos = Vec::new();
    for i in 0..nmess {
        if ctx.gf_poly_eval(err_loc, ctx.gf_pow(generator as u8, i)) == 0 {
            err_pos.push(nmess - 1 - i);
        }
    }

    let errs = err_loc.len() - 1;
    if err_pos.len() != errs {
        return Err(ReedSolomonError {
            message: "Too many (or few) errors found by Chien Search".to_string()
        });
    }

    return Ok(err_pos);
}

fn inverted(msg: &[u8]) -> Vec<u8> {
    return msg.iter().rev().cloned().collect();
}

fn rs_correct_errata(ctx: &GFContext, msg_in: &[u8], synd: &[u8], err_pos: &[usize], fcr: usize, generator: usize) -> Result<Vec<u8>> {
    let mut msg = msg_in.to_vec();

    let coef_pos: Vec<usize> = err_pos.iter().map(|&p| msg.len() - 1 - p).collect();
    let err_loc = rs_find_errata_locator(ctx, &coef_pos, generator);
    let err_eval = inverted(&rs_find_error_evaluator(ctx, &inverted(synd), &err_loc, err_loc.len() - 1));

    let mut x_vec = alloc::vec![0u8; coef_pos.len()];
    for i in 0..coef_pos.len() {
        let l = ctx.field_charac - coef_pos[i];
        x_vec[i] = ctx.gf_pow(generator as u8, ctx.field_charac - l);
    }

    let mut e = alloc::vec![0u8; msg.len()];
    for (i, &xi) in x_vec.iter().enumerate() {
        let xi_inv = ctx.gf_inverse(xi);

        let mut err_loc_prime = 1u8;
        for j in 0..x_vec.len() {
            if j != i {
                err_loc_prime = ctx.gf_mul(err_loc_prime, ctx.gf_sub(1, ctx.gf_mul(xi_inv, x_vec[j])));
            }
        }

        if err_loc_prime == 0 {
            return Err(ReedSolomonError {
                message: "Forney algorithm failed".to_string()
            });
        }

        let mut y = ctx.gf_poly_eval(&inverted(&err_eval), xi_inv);
        y = ctx.gf_mul(ctx.gf_pow(xi, 1 - fcr), y);

        let magnitude = ctx.gf_div(y, err_loc_prime)?;
        e[err_pos[i]] = magnitude;
    }

    msg = ctx.gf_poly_add(&msg, &e);
    return Ok(msg);
}

fn rs_forney_syndromes(ctx: &GFContext, synd: &[u8], pos: &[usize], nmess: usize, generator: usize) -> Vec<u8> {
    let erase_pos_reversed: Vec<usize> = pos.iter().map(|&p| nmess - 1 - p).collect();
    let mut fsynd = synd[1..].to_vec();

    for &i in &erase_pos_reversed {
        let x = ctx.gf_pow(generator as u8, i);
        for j in 0..(fsynd.len() - 1) {
            fsynd[j] = ctx.gf_mul(fsynd[j], x) ^ fsynd[j + 1];
        }
    }

    return fsynd;
}

fn rs_correct_msg(
    ctx: &GFContext,
    msg_in: &[u8],
    nsym: usize,
    fcr: usize,
    generator: usize,
    erase_pos: Option<&[usize]>,
    only_erasures: bool
) -> Result<(Vec<u8>, Vec<u8>, Vec<usize>)> {
    if msg_in.len() > ctx.field_charac {
        return Err(ReedSolomonError {
            message: format!("Message is too long ({} when max is {})", msg_in.len(), ctx.field_charac)
        });
    }

    let mut msg_out = msg_in.to_vec();
    let erase_pos = erase_pos.unwrap_or(&[]);

    for &e_pos in erase_pos {
        if e_pos < msg_out.len() {
            msg_out[e_pos] = 0;
        }
    }

    if erase_pos.len() > nsym {
        return Err(ReedSolomonError {
            message: "Too many erasures to correct".to_string()
        });
    }

    let synd = rs_calc_syndromes(ctx, &msg_out, nsym, fcr, generator);
    if synd[1..].iter().all(|&x| x == 0) {
        let msg_len = msg_out.len() - nsym;
        return Ok((
            msg_out[..msg_len].to_vec(),
            msg_out[msg_len..].to_vec(),
            erase_pos.to_vec()
        ));
    }

    let err_pos = if only_erasures {
        Vec::new()
    } else {
        let fsynd = rs_forney_syndromes(ctx, &synd, erase_pos, msg_out.len(), generator);
        let err_loc = rs_find_error_locator(ctx, &fsynd, nsym, None, erase_pos.len())?;
        rs_find_errors(ctx, &inverted(&err_loc), msg_out.len(), generator)?
    };

    let mut all_err_pos = erase_pos.to_vec();
    all_err_pos.extend(&err_pos);

    msg_out = rs_correct_errata(ctx, &msg_out, &synd, &all_err_pos, fcr, generator)?;

    let synd_check = rs_calc_syndromes(ctx, &msg_out, nsym, fcr, generator);
    if synd_check[1..].iter().any(|&x| x != 0) {
        return Err(ReedSolomonError {
            message: "Could not correct message".to_string()
        });
    }

    let msg_len = msg_out.len() - nsym;
    return Ok((
        msg_out[..msg_len].to_vec(),
        msg_out[msg_len..].to_vec(),
        all_err_pos
    ));
}

// fn rs_check(ctx: &GFContext, msg: &[u8], nsym: usize, fcr: usize, generator: usize) -> bool {
//     let synd = rs_calc_syndromes(ctx, msg, nsym, fcr, generator);
//     return synd[1..].iter().all(|&x| x == 0);
// }

pub struct ReedSolomon {
    nsym: usize,
    nsize: usize,
    fcr: usize,
    // prim: usize,
    generator: usize,
    // c_exp: usize,
    gen_polys: Vec<Vec<u8>>,
    ctx: GFContext
}

impl ReedSolomon {
    pub fn new(data_size: usize, parity_size: usize) -> Self {
        let nsize = data_size + parity_size;
        return Self::new_with_params(parity_size, nsize, 0, 0x11d, 2, 8, true);
    }

    pub fn new_with_params(
        nsym: usize,
        mut nsize: usize,
        fcr: usize,
        mut prim: usize,
        generator: usize,
        mut c_exp: usize,
        single_gen: bool
    ) -> Self {
        if nsize > 255 && c_exp <= 8 {
            c_exp = ((nsize as f64).log2().ceil()) as usize;
        }

        if c_exp != 8 && prim == 0x11d {
            prim = find_prime_poly(generator, c_exp);
            if nsize == 255 {
                nsize = (1 << c_exp) - 1;
            }
        }

        if nsym >= nsize {
            panic!("Parity size must be strictly less than the total message length");
        }

        let ctx = GFContext::new(prim, generator, c_exp);

        let gen_polys = if single_gen {
            let mut gen_vec = alloc::vec![alloc::vec![]; nsize + 1];
            gen_vec[nsym] = rs_generator_poly(&ctx, nsym, fcr, generator);
            gen_vec
        } else {
            (0..=nsize)
                .map(|n| rs_generator_poly(&ctx, n, fcr, generator))
                .collect()
        };

        return Self {
            nsym,
            nsize,
            fcr,
            // prim,
            generator,
            // c_exp,
            gen_polys,
            ctx
        };
    }

    // pub fn data_size(&self) -> usize {
    //     return self.nsize - self.nsym;
    // }

    // pub fn parity_size(&self) -> usize {
    //     return self.nsym;
    // }

    pub fn encode(&self, data: &[u8]) -> Result<Vec<u8>> {
        return self.encode_with_nsym(data, self.nsym);
    }

    pub fn encode_with_nsym(&self, data: &[u8], nsym: usize) -> Result<Vec<u8>> {
        let chunk_size = self.nsize - nsym;
        let total_chunks = (data.len() + chunk_size - 1) / chunk_size;
        let mut enc = Vec::with_capacity(total_chunks * self.nsize);

        for i in 0..total_chunks {
            let start = i * chunk_size;
            let end = (start + chunk_size).min(data.len());
            let chunk = &data[start..end];

            let gen_poly = if self.gen_polys[nsym].is_empty() {
                None
            } else {
                Some(self.gen_polys[nsym].as_slice())
            };

            let encoded_chunk = rs_encode_msg(&self.ctx, chunk, nsym, self.fcr, self.generator, gen_poly)?;
            enc.extend_from_slice(&encoded_chunk);
        }

        return Ok(enc);
    }

    pub fn decode(&self, data: &[u8]) -> Result<(Vec<u8>, Vec<u8>, Vec<usize>)> {
        return self.decode_with_params(data, self.nsym, None, false);
    }

    pub fn decode_with_params(
        &self,
        data: &[u8],
        nsym: usize,
        erase_pos: Option<&[usize]>,
        only_erasures: bool
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<usize>)> {
        let chunk_size = self.nsize;
        let total_chunks = (data.len() + chunk_size - 1) / chunk_size;
        let nmes = self.nsize - nsym;

        let mut dec = Vec::with_capacity(total_chunks * nmes);
        let mut dec_full = Vec::with_capacity(total_chunks * self.nsize);
        let mut errata_pos_all = Vec::new();

        for i in 0..total_chunks {
            let start = i * chunk_size;
            let end = (start + chunk_size).min(data.len());
            let chunk = &data[start..end];

            let chunk_erase_pos = if let Some(erase_pos) = erase_pos {
                let chunk_start = i * chunk_size;
                let chunk_end = chunk_start + chunk_size;
                let chunk_erases: Vec<usize> = erase_pos
                    .iter()
                    .filter_map(|&pos| {
                        if pos >= chunk_start && pos < chunk_end {
                            Some(pos - chunk_start)
                        } else {
                            None
                        }
                    })
                    .collect();
                if chunk_erases.is_empty() {
                    None
                } else {
                    Some(chunk_erases)
                }
            } else {
                None
            };

            let (rmes, recc, errata_pos) = rs_correct_msg(
                &self.ctx,
                chunk,
                nsym,
                self.fcr,
                self.generator,
                chunk_erase_pos.as_deref(),
                only_erasures
            )?;

            dec.extend_from_slice(&rmes);
            dec_full.extend_from_slice(&rmes);
            dec_full.extend_from_slice(&recc);

            for pos in errata_pos {
                errata_pos_all.push(pos + i * chunk_size);
            }
        }

        return Ok((dec, dec_full, errata_pos_all));
    }

    // pub fn check(&self, data: &[u8]) -> Vec<bool> {
    //     return self.check_with_nsym(data, self.nsym);
    // }

    // pub fn check_with_nsym(&self, data: &[u8], nsym: usize) -> Vec<bool> {
    //     let chunk_size = self.nsize;
    //     let total_chunks = (data.len() + chunk_size - 1) / chunk_size;
    //     let mut check = alloc::vec![false; total_chunks];

    //     for i in 0..total_chunks {
    //         let start = i * chunk_size;
    //         let end = (start + chunk_size).min(data.len());
    //         let chunk = &data[start..end];
    //         check[i] = rs_check(&self.ctx, chunk, nsym, self.fcr, self.generator);
    //     }

    //     return check;
    // }
}

fn find_prime_poly(generator: usize, c_exp: usize) -> usize {
    let field_charac = (1 << c_exp) - 1;
    let field_charac_next = (1 << (c_exp + 1)) - 1;

    for prim in (field_charac + 2)..field_charac_next {
        let mut seen = alloc::vec![false; field_charac + 1];
        let mut conflict = false;
        let mut x = 1;

        for _ in 0..field_charac {
            x = gf_mult_nolut(x, generator, prim, field_charac + 1);
            if x > field_charac || seen[x] {
                conflict = true;
                break;
            }
            seen[x] = true;
        }

        if !conflict {
            return prim;
        }
    }

    return 0x11d;
}