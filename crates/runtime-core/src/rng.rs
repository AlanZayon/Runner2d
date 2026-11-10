//! Deterministic XorShift64 RNG (no external crate) + ephemeral play seeds.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::DEFAULT_SEED;

#[derive(Clone, Debug)]
pub struct XorShift64 {
    state: u64,
}

/// Fresh seed for play sessions / unlocked resets (not used for `--seed` runs).
pub fn random_seed() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(DEFAULT_SEED);
    // Mix with a second sample so back-to-back resets differ even within 1ns resolution quirks.
    let nanos2 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let mut mix = nanos
        ^ nanos2.rotate_left(21)
        ^ 0xD1B5_4A32_D192_ED03
        ^ (nanos.wrapping_mul(0x9E37_79B9_7F4A_7C15));
    if mix == 0 {
        mix = DEFAULT_SEED;
    }
    mix
}

impl XorShift64 {
    pub fn from_seed(seed: u64) -> Self {
        // Avoid zero state (XorShift degenerates).
        Self {
            state: seed ^ 0xA5A5_A5A5_A5A5_A5A5,
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    /// Inclusive range [lo, hi].
    pub fn gen_range_u32(&mut self, lo: u32, hi: u32) -> u32 {
        if lo >= hi {
            return lo;
        }
        lo + self.next_u32() % (hi - lo + 1)
    }

    pub fn gen_range_f32(&mut self, lo: f32, hi: f32) -> f32 {
        let t = (self.next_u32() as f32) / (u32::MAX as f32);
        lo + (hi - lo) * t
    }

    pub fn choose_index(&mut self, len: usize) -> usize {
        if len == 0 {
            return 0;
        }
        (self.next_u64() as usize) % len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_stream() {
        let mut a = XorShift64::from_seed(42);
        let mut b = XorShift64::from_seed(42);
        for _ in 0..32 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seed_diverges() {
        let mut a = XorShift64::from_seed(1);
        let mut b = XorShift64::from_seed(2);
        assert_ne!(a.next_u64(), b.next_u64());
    }
}
