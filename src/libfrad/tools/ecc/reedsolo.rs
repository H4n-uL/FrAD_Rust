//!                         Reed-Solomon ECC for Rust                        !//
//!
//! Ported from https://github.com/tomerfiliba/reedsolomon
//! Copyright - Tomer Filiba

#[derive(Debug)]
pub enum RSError {
    DivideByZero,
    MessageTooLong,
    TooManyErasures,
    TooManyErrors,
    ErrorLocationFailure,
    ErrorCorrectionFailure,
}

// Polynomial arithmetic routines
fn gf_add(x: u8, y: u8) -> u8 {
    return x ^ y;
}

fn gf_sub(x: u8, y: u8) -> u8 {
    return gf_add(x, y);
}

fn gf_mul(x: u8, y: u8, gf_exp: &[u8; 512], gf_log: &[u8; 256]) -> u8 {
    return if x == 0 || y == 0 { 0 } else {
        gf_exp[(
            gf_log[x as usize] as usize +
            gf_log[y as usize] as usize
        ) % 255]
    };
}

fn gf_div(x: u8, y: u8, gf_exp: &[u8; 512], gf_log: &[u8; 256]) -> Result<u8, RSError> {
    return if y == 0 { Err(RSError::DivideByZero) } else if x == 0 { Ok(0) } else {
        let log_x = gf_log[x as usize] as usize;
        let log_y = gf_log[y as usize] as usize;
        Ok(gf_exp[((log_x + 255 - log_y) % 255) as usize])
    };
}

fn gf_pow(x: u8, power: u8, gf_exp: &[u8; 512], gf_log: &[u8; 256]) -> u8 {
    return gf_exp[(gf_log[x as usize] as usize * power as usize) % 255];
}

fn gf_inverse(x: u8, gf_exp: &[u8; 512], gf_log: &[u8; 256]) -> u8 {
    return gf_exp[255 - gf_log[x as usize] as usize];
}

fn gf_poly_scale(p: &[u8], x: u8, gf_exp: &[u8; 512], gf_log: &[u8; 256]) -> Vec<u8> {
    return p.iter().map(|&p_coef| gf_mul(p_coef, x, gf_exp, gf_log)).collect();
}

fn gf_poly_add(p: &[u8], q: &[u8]) -> Vec<u8> {
    let mut r = vec![0; p.len().max(q.len())];
    let rlen = r.len();
    for i in 0..p.len() { r[i + rlen - p.len()] = p[i]; }
    for i in 0..q.len() { r[i + rlen - q.len()] ^= q[i]; }
    return r;
}

fn gf_poly_mul(p: &[u8], q: &[u8], gf_exp: &[u8; 512], gf_log: &[u8; 256]) -> Vec<u8> {
    let mut r = vec![0; p.len() + q.len() - 1];
    let lp: Vec<u8> = p.iter().map(|&p_i| gf_log[p_i as usize]).collect();
    for (j, &q_j) in q.iter().enumerate() {
        if q_j != 0 {
            let lq = gf_log[q_j as usize];
            for (i, &p_i) in p.iter().enumerate() {
                if p_i != 0 { r[i + j] ^= gf_exp[lp[i] as usize + lq as usize]; }
            }
        }
    }
    return r;
}

fn gf_poly_eval(poly: &[u8], x: u8, gf_exp: &[u8; 512], gf_log: &[u8; 256]) -> u8 {
    let mut y = poly[0];
    for i in 1..poly.len() {
        y = gf_mul(y, x, gf_exp, gf_log) ^ poly[i];
    }
    return y;
}

fn rs_generator_poly(parity_size: usize, fcr: u8, generator: u8, gf_exp: &[u8; 512], gf_log: &[u8; 256]) -> Vec<u8> {
    let mut g = vec![1];
    for i in 0..parity_size {
        g = gf_poly_mul(&g, &[1, gf_pow(generator, i as u8 + fcr, gf_exp, gf_log)], gf_exp, gf_log);
    }
    return g;
}

fn rs_encode_msg(msg_in: &[u8], parity_size: usize, fcr: u8, generator: u8, gen: Option<&[u8]>, gf_exp: &[u8; 512], gf_log: &[u8; 256]) -> Vec<u8> {
    if msg_in.len() + parity_size > 255 { panic!("Message is too long"); }
    let get_tmp = rs_generator_poly(parity_size, fcr, generator, gf_exp, gf_log);
    let gen = gen.unwrap_or_else(|| &get_tmp);

    let mut msg_out = vec![0; msg_in.len() + gen.len() - 1];
    msg_out[..msg_in.len()].copy_from_slice(msg_in);

    for i in 0..msg_in.len() {
        let coef = msg_out[i];

        if coef != 0 {
            for j in 1..gen.len() {
                msg_out[i + j] ^= gf_mul(gen[j], coef, gf_exp, gf_log);
            }
        }
    }

    msg_out[..msg_in.len()].copy_from_slice(msg_in);
    return msg_out;
}

fn rs_calc_syndromes(msg: &[u8], parity_size: usize, fcr: u8, generator: u8, gf_exp: &[u8; 512], gf_log: &[u8; 256]) -> Vec<u8> {
    let mut synd = vec![0; parity_size + 1];
    synd[0] = 0;
    for i in 0..parity_size {
        synd[i + 1] = gf_poly_eval(msg, gf_pow(generator, i as u8 + fcr, gf_exp, gf_log), gf_exp, gf_log);
    }
    return synd;
}

fn rs_correct_errata(msg_in: &mut [u8], synd: &[u8], err_pos: &[usize], generator: u8, gf_exp: &[u8; 512], gf_log: &[u8; 256]) -> Result<(), RSError> {
    let coef_pos: Vec<_> = err_pos.iter().map(|&p| msg_in.len() - 1 - p).collect();
    let err_loc = rs_find_errata_locator(&coef_pos, generator, gf_exp, gf_log);
    let err_eval = rs_find_error_evaluator(&synd[1..], &err_loc, gf_exp, gf_log);

    let mut xvec = vec![];
    for i in 0..coef_pos.len() {
        let l = 255 - coef_pos[i];
        xvec.push(gf_pow(generator, l as u8, gf_exp, gf_log));
    }

    let mut evec = vec![0; msg_in.len()];
    let xlength = xvec.len();

    for (i, xi) in xvec.iter().enumerate() {
        let xi_inv = gf_inverse(*xi, gf_exp, gf_log);

        let mut err_loc_prime_tmp = vec![];
        for j in 0..xlength {
            if j != i {
                err_loc_prime_tmp.push(gf_sub(1, gf_mul(xi_inv, xvec[j], gf_exp, gf_log)));
            }
        }
        let mut err_loc_prime = 1;
        for coef in &err_loc_prime_tmp {
            err_loc_prime = gf_mul(err_loc_prime, *coef, gf_exp, gf_log);
        }

        if err_loc_prime == 0 {
            return Err(RSError::ErrorCorrectionFailure);
        }

        let y = gf_poly_eval(&err_eval, xi_inv, gf_exp, gf_log);
        let magnitude = gf_div(y, err_loc_prime, gf_exp, gf_log)?;
        evec[err_pos[i]] = magnitude;
    }

    for (i, e) in evec.iter().enumerate() { msg_in[i] ^= *e; }

    return Ok(());
}

fn rs_find_error_locator(synd: &[u8], parity_size: usize, erase_loc: Option<&[u8]>, erase_count: usize, gf_exp: &[u8; 512], gf_log: &[u8; 256]) -> Result<Vec<u8>, RSError> {
    let mut err_loc = match erase_loc {
        Some(erase_loc) => erase_loc.to_vec(),
        None => vec![1],
    };
    let mut old_loc = err_loc.clone();

    let synd_shift = synd.len() - parity_size;

    for i in 0..parity_size-erase_count {
        let mut delta = synd[i+synd_shift];
        for j in 1..err_loc.len() {
            delta ^= gf_mul(err_loc[err_loc.len()-j-1], synd[i+synd_shift-j], gf_exp, gf_log);
        }

        old_loc.push(0);
        if delta != 0 {
            if old_loc.len() > err_loc.len() {
                let new_loc = gf_poly_scale(&old_loc, delta, gf_exp, gf_log);
                old_loc = gf_poly_scale(&err_loc, gf_inverse(delta, gf_exp, gf_log), gf_exp, gf_log);
                err_loc = new_loc;
            }

            err_loc = gf_poly_add(&err_loc, &gf_poly_scale(&old_loc, delta, gf_exp, gf_log));
        }
    }

    while let Some(0) = err_loc.first() {
        err_loc.remove(0);
    }

    let errs = err_loc.len() - 1;
    if errs*2 + erase_count > parity_size {
        return Err(RSError::TooManyErrors);
    }

    return Ok(err_loc);
}

fn rs_find_errata_locator(e_pos: &[usize], generator: u8, gf_exp: &[u8; 512], gf_log: &[u8; 256]) -> Vec<u8> {
    let mut e_loc = vec![1];
    for i in e_pos {
        e_loc = gf_poly_mul(&e_loc, &[1, gf_pow(generator, *i as u8, gf_exp, gf_log)], gf_exp, gf_log);
    }
    return e_loc;
}

fn rs_find_error_evaluator(synd: &[u8], err_loc: &[u8], gf_exp: &[u8; 512], gf_log: &[u8; 256]) -> Vec<u8> {
    let (_, remainder) = gf_poly_div(gf_poly_mul(synd, err_loc, gf_exp, gf_log), &[1], gf_exp, gf_log);
    return remainder;
}

fn rs_find_errors(err_loc: &[u8], nmess: usize, generator: u8, gf_exp: &[u8; 512], gf_log: &[u8; 256]) -> Result<Vec<usize>, RSError> {
    let mut errs = vec![];
    for i in 0..nmess {
        if gf_poly_eval(err_loc, gf_pow(generator, i as u8, gf_exp, gf_log), gf_exp, gf_log) == 0 {
            errs.push(nmess - 1 - i);
        }
    }

    if errs.len() != err_loc.len() - 1 {
        return Err(RSError::TooManyErrors);
    }

    return Ok(errs);
}

fn rs_forney_syndromes(synd: &[u8], pos: &[usize], nmess: usize, generator: u8, gf_exp: &[u8; 512], gf_log: &[u8; 256]) -> Vec<u8> {
    let mut fsynd = synd[1..].to_vec();

    for i in 0..pos.len() {
        let x = gf_pow(generator, (nmess-1-pos[i]) as u8, gf_exp, gf_log);
        for j in 0..fsynd.len()-1 {
            fsynd[j] = gf_mul(fsynd[j], x, gf_exp, gf_log) ^ fsynd[j+1];
        }
    }

    return fsynd;
}

fn rs_correct_msg(msg_in: &mut [u8], parity_size: usize, fcr: u8, generator: u8, erase_pos: Option<&[usize]>, only_erasures: bool, gf_exp: &[u8; 512], gf_log: &[u8; 256]) -> Result<(Vec<u8>, Vec<u8>, Vec<usize>), RSError> {
    if msg_in.len() > 255 {
        return Err(RSError::MessageTooLong);
    }

    let erase_pos = erase_pos.unwrap_or(&[]);
    for e_pos in erase_pos {
        msg_in[*e_pos] = 0;
    }

    if erase_pos.len() > parity_size {
        return Err(RSError::TooManyErasures);
    }

    let synd = rs_calc_syndromes(msg_in, parity_size, fcr, generator, gf_exp, gf_log);
    if synd.iter().max() == Some(&0) {
        return Ok((msg_in[..msg_in.len()-parity_size].to_vec(), msg_in.to_vec(), vec![]));
    }

    let mut err_pos = vec![];
    if !only_erasures {
        let fsynd = rs_forney_syndromes(&synd, erase_pos, msg_in.len(), generator, gf_exp, gf_log);
        let err_loc = rs_find_error_locator(&fsynd, parity_size, Some(&rs_find_errata_locator(erase_pos, generator, gf_exp, gf_log)), erase_pos.len(), gf_exp, gf_log)?;
        err_pos = rs_find_errors(&err_loc, msg_in.len(), generator, gf_exp, gf_log)?;
        if err_pos.is_empty() {
            return Err(RSError::ErrorLocationFailure);
        }
    }

    let corres = rs_correct_errata(msg_in, &synd, &(erase_pos.iter().chain(err_pos.iter()).cloned().collect::<Vec<_>>()), generator, gf_exp, gf_log);
    if corres.is_err() {
        return Err(corres.err().unwrap());
    }
    let synd = rs_calc_syndromes(msg_in, parity_size, fcr, generator, gf_exp, gf_log);
    if synd.iter().max() != Some(&0) {
        return Err(RSError::ErrorCorrectionFailure);
    }

    return Ok((msg_in[..msg_in.len()-parity_size].to_vec(), msg_in.to_vec(), erase_pos.iter().chain(err_pos.iter()).cloned().collect()));
}

// Polynomial division routines
fn gf_poly_div(mut dividend: Vec<u8>, divisor: &[u8], gf_exp: &[u8; 512], gf_log: &[u8; 256]) -> (Vec<u8>, Vec<u8>) {
    if dividend.len() < divisor.len() {
        return (vec![0; divisor.len()], dividend);
    }
    for i in 0..dividend.len() - (divisor.len()-1) {
        let coef = dividend[i];
        if coef != 0 {
            for j in 1..divisor.len() {
                if divisor[j] != 0 {
                    dividend[i+j] ^= gf_mul(divisor[j], coef, gf_exp, gf_log);
                }
            }
        }
    }

    let separator = -(divisor.len() as isize) + 1;
    return (dividend[..dividend.len()-(divisor.len()-1)].to_vec(), dividend[(separator as usize)..].to_vec());
}

// Log/antilog tables and polynomial generator routines
fn init_tables(prim: u16, generator: u8, c_exp: u32) -> ([u8; 256], [u8; 512]) {
    let mut gf_exp = [0; 512];
    let mut gf_log = [0; 256];
    let field_charac = 2_u32.pow(c_exp) - 1;

    let mut x = 1;
    for i in 0..field_charac as usize {
        gf_exp[i] = x as u8;
        gf_log[x as usize] = i as u8;
        x = gf_mul_no_lut(x as u16, generator, prim, field_charac as u16 + 1);
    }

    for i in field_charac as usize..field_charac as usize * 2 {
        gf_exp[i] = gf_exp[i - field_charac as usize];
    }

    return (gf_log, gf_exp);
}

fn gf_mul_no_lut(mut x: u16, y: u8, prim: u16, field_charac_full: u16) -> u8 {
    let mut result = 0u64;
    let mut y = y;
    while y > 0 {
        if y & 1 > 0 {
            result ^= x as u64;
        }
        y >>= 1;
        x <<= 1;
        if x & field_charac_full > 0 {
            x ^= prim;
        }
    }
    return result as u8;
}

// Galois Field arithmetic routines
fn find_prime_polys(generator: u8, c_exp: u32, fast_primes: bool, single: bool) -> Vec<u16> {
    let mut _prim_candidates: Vec<u32> = Vec::new();
    let mut correct_primes: Vec<u16> = Vec::new();

    let root_charac: u32 = 2;
    let field_charac = root_charac.pow(c_exp as u32) - 1;
    let field_charac_next = root_charac.pow(c_exp as u32 + 1) - 1;

    if fast_primes {
        _prim_candidates = primes_in_range(field_charac + 1, field_charac_next).into_iter().map(|x| x as u32).collect();
    }
    else {
        _prim_candidates = (field_charac..field_charac_next).step_by(2).collect();
    }

    for prim in _prim_candidates {
        let mut conflict = false;
        let mut x = 1;
        for _ in 0..field_charac {
            x = gf_mul_no_lut(x as u16, generator, prim as u16, field_charac as u16);
            if x > field_charac as u8 || x == 1 {
                conflict = true;
                break;
            }
        }

        if !conflict {
            correct_primes.push(prim as u16);
            if single {
                return correct_primes;
            }
        }
    }

    return correct_primes;
}

// Prime numbers routines
fn primes_in_range(start: u32, end: u32) -> Vec<u32> {
    let mut sieve = vec![true; ((end - start)/2) as usize];
    for i in ((start as f64).sqrt() as u32 - start)..((end as f64).sqrt() as u32 - start) {
        if sieve[(i/2) as usize] {
            for j in (i*i/2..((end-start)/2) as u32).step_by(i as usize) {
                sieve[j as usize] = false;
            }
        }
    }

    let mut primes = Vec::new();
    if start == 2 {
        primes.push(2);
    }
    for i in 0..((end-start)/2) as usize {
        if sieve[i] {
            primes.push(2*i as u32 + 1 + start);
        }
    }

    return primes;
}

pub struct RSCodec {
    pub data_size: usize,
    pub parity_size: usize,
    pub fcr: u8,
    pub prim: u16,
    pub generator: u8,
    pub c_exp: u32,
    gf_log: [u8; 256],
    gf_exp: [u8; 512],
    gen: Vec<u8>,
}

impl RSCodec {
    pub fn new(data_size: usize, parity_size: usize, fcr: u8, prim: u16, generator: u8, c_exp: u32) -> Self {
        let mut rs = Self {
            data_size, parity_size,
            fcr, prim,
            generator, c_exp,
            gf_log: [0; 256],
            gf_exp: [0; 512],
            gen: Vec::new(),
        };
        let block_size = data_size + parity_size;

        if block_size > 255 && c_exp <= 8 {
            rs.c_exp = (((block_size-1) as f64).ln() / 2.0_f64.ln()).ceil() as u32;
        }

        if c_exp != 8 && prim == 0x11d {
            rs.prim = find_prime_polys(generator, c_exp, true, true)[0];
        }

        let (gf_log, gf_exp) = init_tables(rs.prim, rs.generator, rs.c_exp);
        rs.gf_log = gf_log;
        rs.gf_exp = gf_exp;

        rs.gen = rs_generator_poly(parity_size, rs.fcr, rs.generator, &rs.gf_exp, &rs.gf_log);

        return rs;
    }

    pub fn new_default(data_size: usize, parity_size: usize) -> RSCodec {
        return RSCodec::new(data_size, parity_size, 0, 0x11d, 2, 8);
    }

    pub fn encode(&self, data: &[u8]) -> Vec<u8> {
        let chunk_size = self.data_size;

        let mut chunks = Vec::new();
        for chunk in data.chunks(chunk_size) {
            let chunk_enc = rs_encode_msg(&chunk.to_vec(), self.parity_size, self.fcr, self.generator, None, &self.gf_exp, &self.gf_log);
            chunks.push(chunk_enc);
        }
        return chunks.into_iter().flatten().collect();
    }

    pub fn decode(&self, data: &[u8], erase_pos: Option<&[usize]>) -> Result<Vec<u8>, RSError> {
        let chunk_size = self.data_size + self.parity_size;
        let enc_chunk_size = chunk_size + self.parity_size;
        let erase_pos = erase_pos.unwrap_or(&[]);

        let mut chunks = Vec::new();
        for (chunk_index, chunk) in data.chunks(enc_chunk_size).enumerate() {
            let chunk_erase_pos: Vec<usize> = erase_pos.iter()
                .filter_map(|&pos| {
                    if pos >= chunk_index * enc_chunk_size && pos < (chunk_index + 1) * enc_chunk_size
                    { Some(pos - chunk_index * enc_chunk_size) } else { None }
                })
                .collect();

            let chunk_dec = rs_correct_msg(&mut chunk.to_vec(), self.parity_size, self.fcr, self.generator, Some(&chunk_erase_pos), false, &self.gf_exp, &self.gf_log);
            if chunk_dec.is_err() { return Err(chunk_dec.err().unwrap()); }
            let (decoded, _, _) = chunk_dec.unwrap();
            chunks.push(decoded);
        }

        return Ok(chunks.into_iter().flatten().collect());
    }
}