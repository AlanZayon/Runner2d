//! Centralized temporary abilities — do not scatter ability logic across the runtime.

use crate::config::{
    GHOST_DURATION, MAGNET_DURATION, SLOW_FALL_DURATION, STRENGTH_DURATION_GOOD,
    STRENGTH_DURATION_GREAT, STRENGTH_DURATION_PERFECT, STRENGTH_DURATION_POOR,
};
use crate::modifiers::{CollisionModifiers, MovementModifiers};
use crate::qte::{QteRating, StrengthQte};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AbilityId {
    Magnet = 0,
    SlowFall = 1,
    Ghost = 2,
    SuperStrength = 3,
}

impl AbilityId {
    pub const ALL: [AbilityId; 4] = [
        AbilityId::Magnet,
        AbilityId::SlowFall,
        AbilityId::Ghost,
        AbilityId::SuperStrength,
    ];

    pub fn bit(self) -> u32 {
        1u32 << (self as u8)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AbilityTimer {
    pub remaining: f32,
}

impl AbilityTimer {
    pub fn active(&self) -> bool {
        self.remaining > 0.0
    }
}

#[derive(Clone, Debug, Default)]
pub struct AbilityManager {
    timers: [AbilityTimer; 4],
    pub qte: Option<StrengthQte>,
}

impl AbilityManager {
    pub fn clear(&mut self) {
        self.timers = [AbilityTimer::default(); 4];
        self.qte = None;
    }

    pub fn is_active(&self, id: AbilityId) -> bool {
        self.timers[id as usize].active()
    }

    pub fn remaining(&self, id: AbilityId) -> f32 {
        self.timers[id as usize].remaining
    }

    pub fn active_mask(&self) -> u32 {
        let mut m = 0u32;
        for id in AbilityId::ALL {
            if self.is_active(id) {
                m |= id.bit();
            }
        }
        m
    }

    pub fn qte_active(&self) -> bool {
        self.qte.is_some()
    }

    /// Pickup / activate. Super Strength starts QTE instead of immediate strength.
    pub fn activate(&mut self, id: AbilityId) {
        match id {
            AbilityId::SuperStrength => {
                // Restart QTE; cancel any in-progress strength timer until rated.
                self.timers[AbilityId::SuperStrength as usize] = AbilityTimer::default();
                self.qte = Some(StrengthQte::new());
            }
            AbilityId::Magnet => self.refresh(id, MAGNET_DURATION),
            AbilityId::SlowFall => self.refresh(id, SLOW_FALL_DURATION),
            AbilityId::Ghost => self.refresh(id, GHOST_DURATION),
        }
    }

    pub fn refresh(&mut self, id: AbilityId, duration: f32) {
        self.timers[id as usize].remaining = duration.max(0.0);
    }

    pub fn activate_strength_from_rating(&mut self, rating: QteRating) {
        let dur = match rating {
            QteRating::Poor => STRENGTH_DURATION_POOR,
            QteRating::Good => STRENGTH_DURATION_GOOD,
            QteRating::Great => STRENGTH_DURATION_GREAT,
            QteRating::Perfect => STRENGTH_DURATION_PERFECT,
        };
        self.qte = None;
        self.refresh(AbilityId::SuperStrength, dur);
    }

    pub fn tick(&mut self, dt: f32) {
        for t in &mut self.timers {
            if t.remaining > 0.0 {
                t.remaining = (t.remaining - dt).max(0.0);
            }
        }
    }

    /// Feed QTE edge press; returns completed rating when the sequence ends.
    pub fn tick_qte(&mut self, dt: f32, qte_pressed: bool) -> Option<QteRating> {
        let Some(qte) = self.qte.as_mut() else {
            return None;
        };
        if let Some(rating) = qte.tick(dt, qte_pressed) {
            self.activate_strength_from_rating(rating);
            return Some(rating);
        }
        None
    }

    pub fn movement_modifiers(&self) -> MovementModifiers {
        let mut m = MovementModifiers::default();
        if self.is_active(AbilityId::SlowFall) {
            m.gravity_scale = crate::config::SLOW_FALL_GRAVITY_SCALE;
            m.max_fall_scale = crate::config::SLOW_FALL_MAX_FALL_SCALE;
        }
        m
    }

    pub fn collision_modifiers(&self) -> CollisionModifiers {
        CollisionModifiers {
            ghost_active: self.is_active(AbilityId::Ghost),
            strength_active: self.is_active(AbilityId::SuperStrength),
            invulnerable: self.is_active(AbilityId::SuperStrength),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refresh_does_not_stack() {
        let mut m = AbilityManager::default();
        m.activate(AbilityId::Magnet);
        let first = m.remaining(AbilityId::Magnet);
        m.activate(AbilityId::Magnet);
        assert!((m.remaining(AbilityId::Magnet) - first).abs() < 0.01);
    }

    #[test]
    fn strength_starts_qte_not_active() {
        let mut m = AbilityManager::default();
        m.activate(AbilityId::SuperStrength);
        assert!(m.qte_active());
        assert!(!m.is_active(AbilityId::SuperStrength));
    }

    #[test]
    fn expire_clears_modifiers() {
        let mut m = AbilityManager::default();
        m.refresh(AbilityId::Ghost, 0.05);
        m.tick(0.1);
        assert!(!m.collision_modifiers().ghost_active);
    }
}
