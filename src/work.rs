use std::hint::black_box;

const STRANGE_FLOAT: f64 = 3.123456;

pub type Work = fn() -> ();

pub enum Op {
    Mul,
    Add,
    MulAdd,
}

#[inline(always)]
fn float(count: usize, op: Op) {
    let mut n: f64 = 0.0;
    for _ in 0..count {
        match op {
            Op::Add => n += black_box(STRANGE_FLOAT),
            Op::Mul => n *= black_box(STRANGE_FLOAT),
            Op::MulAdd => n = n.mul_add(black_box(STRANGE_FLOAT), black_box(STRANGE_FLOAT)),
        }
    }
    black_box(n);
}

// float
#[inline(always)]
pub fn float_fst() {
    float(crate::params::work::FST, Op::MulAdd);
}

#[inline(always)]
pub fn float_snd() {
    float(crate::params::work::SND, Op::MulAdd);
}

#[inline(always)]
pub fn float_thd() {
    float(crate::params::work::THD, Op::MulAdd);
}

#[inline(always)]
pub fn float_fth() {
    float(crate::params::work::FTH, Op::MulAdd);
}

#[inline(always)]
pub fn float_fft() {
    float(crate::params::work::FFT, Op::MulAdd);
}

#[inline(always)]
pub fn nothing() {}
