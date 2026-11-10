//! Lightweight nanosecond accumulators for Phase 10 (always-on, drained on demand).

#[derive(Clone, Copy, Debug, Default)]
pub struct TickProfile {
    pub ticks: u64,
    pub control_ns: u64,
    pub integrate_ns: u64,
    pub collision_ns: u64,
    pub hazards_ns: u64,
    pub streamer_ns: u64,
    pub extract_ns: u64,
    pub other_ns: u64,
}

impl TickProfile {
    pub fn total_ns(&self) -> u64 {
        self.control_ns
            + self.integrate_ns
            + self.collision_ns
            + self.hazards_ns
            + self.streamer_ns
            + self.extract_ns
            + self.other_ns
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

#[inline]
pub fn elapsed_ns(start: std::time::Instant) -> u64 {
    start.elapsed().as_nanos().min(u64::MAX as u128) as u64
}
