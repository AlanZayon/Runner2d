//! Fixed-capacity CPU particle pool (no per-frame alloc, no GPU particles).

use crate::config::EFFECT_POOL_CAP;

#[derive(Clone, Copy, Debug)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max_life: f32,
    pub w: f32,
    pub h: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Clone, Debug)]
pub struct EffectPool {
    slots: Vec<Option<Particle>>,
    count: usize,
    /// Soft cap for quality profiles (≤ slots.len()).
    active_cap: usize,
    trail_accum: f32,
    slow_accum: f32,
}

impl Default for EffectPool {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectPool {
    pub fn new() -> Self {
        Self {
            slots: vec![None; EFFECT_POOL_CAP],
            count: 0,
            active_cap: EFFECT_POOL_CAP,
            trail_accum: 0.0,
            slow_accum: 0.0,
        }
    }

    pub fn clear(&mut self) {
        for s in &mut self.slots {
            *s = None;
        }
        self.count = 0;
        self.trail_accum = 0.0;
        self.slow_accum = 0.0;
    }

    pub fn live_count(&self) -> usize {
        self.count
    }

    pub fn cap(&self) -> usize {
        self.active_cap.min(self.slots.len())
    }

    pub fn set_active_cap(&mut self, cap: usize) {
        self.active_cap = cap.clamp(8, self.slots.len());
    }

    pub fn spawn_dash_trail(&mut self, x: f32, y: f32) {
        self.spawn(Particle {
            x,
            y,
            vx: -60.0,
            vy: 0.0,
            life: 0.18,
            max_life: 0.18,
            w: 14.0,
            h: 8.0,
            r: 0.3,
            g: 0.9,
            b: 1.0,
            a: 0.55,
        });
    }

    pub fn spawn_speed_lines(&mut self, x: f32, y: f32) {
        for i in 0..3 {
            self.spawn(Particle {
                x: x - 20.0 - i as f32 * 8.0,
                y: y + i as f32 * 6.0,
                vx: -120.0,
                vy: 0.0,
                life: 0.12,
                max_life: 0.12,
                w: 18.0,
                h: 2.0,
                r: 0.9,
                g: 0.9,
                b: 1.0,
                a: 0.4,
            });
        }
    }

    fn spawn(&mut self, p: Particle) {
        if self.count >= self.cap() {
            return;
        }
        if self.count >= self.slots.len() {
            // Drop oldest by overwriting first dead-or-zero; if full, overwrite slot 0.
            let idx = self
                .slots
                .iter()
                .position(|s| s.is_none())
                .unwrap_or(0);
            if self.slots[idx].is_none() {
                self.count += 1;
            }
            self.slots[idx] = Some(p);
            self.count = self.count.min(self.slots.len());
            return;
        }
        if let Some(idx) = self.slots.iter().position(|s| s.is_none()) {
            self.slots[idx] = Some(p);
            self.count += 1;
        }
    }

    pub fn spawn_burst(&mut self, x: f32, y: f32, r: f32, g: f32, b: f32, n: usize) {
        for i in 0..n {
            let ang = (i as f32) * 1.7;
            self.spawn(Particle {
                x,
                y,
                vx: ang.cos() * 80.0,
                vy: ang.sin() * 80.0 - 40.0,
                life: 0.35,
                max_life: 0.35,
                w: 6.0,
                h: 6.0,
                r,
                g,
                b,
                a: 0.9,
            });
        }
    }

    pub fn spawn_magnet_pull(&mut self, x: f32, y: f32) {
        self.spawn(Particle {
            x,
            y,
            vx: 0.0,
            vy: -20.0,
            life: 0.2,
            max_life: 0.2,
            w: 4.0,
            h: 4.0,
            r: 1.0,
            g: 0.85,
            b: 0.2,
            a: 0.7,
        });
    }

    pub fn tick_ghost_trail(&mut self, dt: f32, x: f32, y: f32, interval: f32) {
        self.trail_accum += dt;
        if self.trail_accum >= interval {
            self.trail_accum = 0.0;
            self.spawn(Particle {
                x,
                y,
                vx: -30.0,
                vy: 10.0,
                life: 0.25,
                max_life: 0.25,
                w: 10.0,
                h: 18.0,
                r: 0.55,
                g: 0.75,
                b: 1.0,
                a: 0.35,
            });
        }
    }

    pub fn tick_slow_fall(&mut self, dt: f32, x: f32, y: f32, interval: f32) {
        self.slow_accum += dt;
        if self.slow_accum >= interval {
            self.slow_accum = 0.0;
            self.spawn(Particle {
                x,
                y: y + 20.0,
                vx: 15.0,
                vy: 40.0,
                life: 0.3,
                max_life: 0.3,
                w: 5.0,
                h: 8.0,
                r: 0.6,
                g: 0.9,
                b: 1.0,
                a: 0.5,
            });
        }
    }

    pub fn tick(&mut self, dt: f32) {
        let mut live = 0;
        for slot in &mut self.slots {
            if let Some(p) = slot.as_mut() {
                p.life -= dt;
                p.x += p.vx * dt;
                p.y += p.vy * dt;
                p.a = (p.life / p.max_life).clamp(0.0, 1.0) * 0.9;
                if p.life <= 0.0 {
                    *slot = None;
                } else {
                    live += 1;
                }
            }
        }
        self.count = live;
    }

    pub fn iter_live(&self) -> impl Iterator<Item = &Particle> {
        self.slots.iter().filter_map(|s| s.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_respects_cap() {
        let mut pool = EffectPool::new();
        for _ in 0..EFFECT_POOL_CAP * 3 {
            pool.spawn_burst(0.0, 0.0, 1.0, 1.0, 1.0, 8);
        }
        assert!(pool.live_count() <= pool.cap());
    }
}
