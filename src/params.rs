#[cfg(not(any(feature = "full", feature = "dry")))]
pub const NS_WORKERS: [usize; 5] = [1, 2, 4, 8, 12];
#[cfg(feature = "full")]
pub const NS_WORKERS: [usize; 11] = [1, 2, 4, 8, 12, 14, 16, 18, 20, 22, 24];
#[cfg(feature = "dry")]
pub const NS_WORKERS: [usize; 1] = [2];

pub const NS_SPAWN_GLOBAL: [usize; 16] = [
    100, 200, 250, 300, 500, 750, 1000, 2_000, 3000, 4000, 5_000, 6_000, 7_000, 8_000, 9000, 10_000,
];
pub const NS_SPAWN_LOCAL: [usize; 15] = [
    50, 100, 150, 200, 230, 240, 250, 260, 270, 300, 320, 350, 400, 420, 450,
];

#[cfg(not(any(feature = "full", feature = "dry")))]
pub const NS_SPLIT_LOCAL: [usize; 12] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
#[cfg(feature = "full")]
pub const NS_SPLIT_LOCAL: [usize; 24] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
];
#[cfg(feature = "dry")]
pub const NS_SPLIT_LOCAL: [usize; 1] = [2];

#[cfg(not(any(feature = "full", feature = "dry")))]
pub const NS_SPLIT_GLOBAL: [usize; 5] = [10, 50, 100, 150, 200];
#[cfg(feature = "full")]
pub const NS_SPLIT_GLOBAL: [usize; 10] = [10, 50, 100, 150, 200, 250, 300, 350, 400, 450];
#[cfg(feature = "dry")]
pub const NS_SPLIT_GLOBAL: [usize; 1] = [2];

pub const N_SPAWN_GLOBAL: usize = 100_000;
pub const N_SPAWN_LOCAL: usize = 10_000;

pub const YIEDL_BOUND: usize = 10;

#[cfg(not(feature = "full"))]
pub const N_WORKERS: usize = 12;
#[cfg(feature = "full")]
pub const N_WORKERS: usize = 24;

pub mod work {
    pub const FST: usize = 25;
    pub const SND: usize = 50;
    pub const THD: usize = 100;
    pub const FTH: usize = 200;
    pub const FFT: usize = 500;
}

pub mod metrics {
    pub const SAMPLE_SLICE: u64 = 50;

    pub const CHAN_SIZE: usize = 100000;

    pub const N_ITER: usize = 100;
}
