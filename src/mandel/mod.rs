pub mod cplx;
use cplx::Cplx;

#[derive(Clone, Copy)]
pub struct Mandel {
    c: Cplx<f64>,
    n: f64,
    zp: (f64, f64),
    n_max: usize,
}

impl Mandel {
    pub fn new(c: Cplx<f64>, n_max: usize) -> Self {
        Mandel {
            c,
            n_max,
            zp: (f64::NAN, f64::NAN),
            n: f64::NAN,
        }
    }
    pub fn new_empty() -> Self {
        Mandel {
            c: Cplx::<f64> { re: 0., im: 0. },
            n_max: 256,
            zp: (f64::NAN, f64::NAN),
            n: f64::NAN,
        }
    }
    #[inline]
    pub fn get_mandel(&self) -> usize {
        let mut z = self.c;
        for n in 1..self.n_max {
            if z.sq_abs() >= 4. {
                return n;
            }
            z = z.square() + self.c;
        }
        self.n_max
    }

    #[inline]
    pub fn get_finished(&self) -> Option<f64> {
        if !self.n.is_nan() {
            Some(self.n)
        } else {
            None
        }
    }

    #[inline]
    pub fn get_shadow(&self) -> Option<(f64, f64)> {
        if !self.n.is_nan() {
            Some(self.zp)
        } else {
            None
        }
    }

    #[inline]
    pub fn calculate_mandel_smooth(&mut self) {
        let mut z = self.c;
        const M: f64 = 32.;
        for i in 1..self.n_max {
            if z.sq_abs() >= M * M {
                self.n = i as f64;
                break;
            }
            z = z.square() + self.c;
        }
        if self.n.is_nan() {
            self.n = f64::INFINITY;
        } else {

            // n - fast_log2(0.5*fast_ln(z.sq_abs()))
            self.n -= fast_log2(0.5 * fast_ln(z.sq_abs()));
            // self.zp.0 -= self.n;
            // self.zp.1 -= self.n;
            // N + 1 + 1/ln(p)*ln(ln(M)/ln(r)) //M = big escape value, p = power (2 here), r = radius at escape
            // => N + 1 + log2(ln(M)/ln(r))
            // => N + 1 + log2(ln(M)) - log2(ln(r)) //we can get rid of constants, they are just a shift
            // => N - log2(ln(r))
            // => N - log2(log2(r)/log2(e)) => N - log2(log2(r))
        }
    }

    #[inline]
    pub fn get_julia_smooth(&self, c: Cplx<f64>) -> f64 {
        let mut z = self.c;
        let mut n = 0.;
        const M: f64 = 10.;
        for i in 1..self.n_max {
            if z.sq_abs() >= M * M {
                n = i as f64;
                break;
            }
            z = z.square() + c;
        }
        if n.is_nan() {
            return f64::INFINITY;
        }

        // n - fast_log2(0.5*fast_ln(z.sq_abs()))
        n - fast_log2(0.5 * fast_ln(z.sq_abs()))
        // N + 1 + 1/ln(p)*ln(ln(M)/ln(r)) //M = big escape value, p = power (2 here), r = radius at escape
        // => N + 1 + log2(ln(M)/ln(r))
        // => N + 1 + log2(ln(M)) - log2(ln(r)) //we can get rid of constants, they are just a shift
        // => N - log2(ln(r))
        // => N - log2(log2(r)/log2(e)) => N - log2(log2(r))
    }
}

#[inline]
pub fn fast_log2(x: f64) -> f64 {
    let l2 = (x.to_bits() >> 52 & 0x07FF) as i16 - 0x0400;
    let v = f64::from_bits(x.to_bits() & !(1 << 62) | 0x3FF00000_00000000);
    let v = (-v / 3. + 2.) * v - 2. / 3.;
    v + l2 as f64
}

#[inline]
fn fast_ln(x: f64) -> f64 {
    fast_log2(x) / 1.4426950408889634
}
