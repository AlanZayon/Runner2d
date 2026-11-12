//! Segment definitions (data) separate from live world instances.

use crate::collision_cat::CollisionCategory;
use crate::components::{HazardKind, PowerupKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum SegmentIntent {
    TeachJump = 0,
    TeachDash = 1,
    TeachRoll = 2,
    JumpDash = 3,
    JumpRoll = 4,
    RiskReward = 5,
    Recovery = 6,
    Transition = 7,
    /// Vertical drop shaft (Phase 7 Falling Mode).
    FallingShaft = 8,
    /// Ghost-required shortcut past AlwaysSolid trap (Phase 12).
    GhostChallenge = 9,
}

/// Geometry hints for Falling Mode enter/exit (absolute world coords).
#[derive(Clone, Copy, Debug)]
pub struct ShaftSpec {
    pub pit_x0: f32,
    pub pit_x1: f32,
    pub floor_y: f32,
    pub enter_y: f32,
    pub exit_x: f32,
    pub exit_y: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct PlatformSpec {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub tint: (f32, f32, f32),
}

#[derive(Clone, Copy, Debug)]
pub struct DestructibleSpec {
    pub health: f32,
    pub reward_coins: u32,
    pub material: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct HazardSpec {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub kind: HazardKind,
    pub tint: (f32, f32, f32),
    pub category: CollisionCategory,
    pub destructible: Option<DestructibleSpec>,
}

#[derive(Clone, Copy, Debug)]
pub struct CoinSpec {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct PowerupSpec {
    pub kind: PowerupKind,
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug)]
pub struct SegmentDefinition {
    pub intent: SegmentIntent,
    pub template_id: u32,
    /// Absolute world X of segment origin.
    pub world_x0: f32,
    pub width: f32,
    /// Y of the run-floor platforms for this segment (updates after FallingShaft).
    pub ground_y: f32,
    pub platforms: Vec<PlatformSpec>,
    pub hazards: Vec<HazardSpec>,
    pub coins: Vec<CoinSpec>,
    pub powerups: Vec<PowerupSpec>,
    pub shaft: Option<ShaftSpec>,
}

impl SegmentDefinition {
    pub fn world_x1(&self) -> f32 {
        self.world_x0 + self.width
    }

    /// Ground altitude where this segment is entered from the previous one.
    pub fn entry_ground_y(&self) -> f32 {
        self.shaft.map(|s| s.enter_y).unwrap_or(self.ground_y)
    }

    /// Ground altitude the next segment should continue from.
    pub fn exit_ground_y(&self) -> f32 {
        self.shaft.map(|s| s.exit_y).unwrap_or(self.ground_y)
    }
}

#[derive(Clone, Debug)]
pub struct GeneratedCourse {
    pub seed: u64,
    pub generator_version: u32,
    pub segments: Vec<SegmentDefinition>,
    pub course_end: f32,
}

impl GeneratedCourse {
    pub fn fingerprint(&self) -> u64 {
        let mut h: u64 = self.generator_version as u64;
        h = h.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(self.seed);
        h = h
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            .wrapping_add(self.segments.len() as u64);
        h = h
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            .wrapping_add(self.course_end.to_bits() as u64);
        for seg in &self.segments {
            h = h
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .wrapping_add(seg.template_id as u64);
            h = h
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .wrapping_add(seg.width.to_bits() as u64);
            h = h
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .wrapping_add(seg.platforms.len() as u64);
            h = h
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .wrapping_add(seg.hazards.len() as u64);
            h = h
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .wrapping_add(seg.coins.len() as u64);
            h = h
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .wrapping_add(seg.powerups.len() as u64);
            for p in &seg.platforms {
                h = h
                    .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                    .wrapping_add(p.x.to_bits() as u64);
                h = h
                    .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                    .wrapping_add(p.w.to_bits() as u64);
            }
            for hz in &seg.hazards {
                h = h
                    .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                    .wrapping_add(hz.x.to_bits() as u64);
                let kind_tag = match hz.kind {
                    HazardKind::SolidKill => 1u64,
                    HazardKind::LowBar => 2u64,
                };
                h = h.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(kind_tag);
                h = h
                    .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                    .wrapping_add(hz.category as u64);
            }
            for pu in &seg.powerups {
                h = h
                    .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                    .wrapping_add(pu.kind as u64);
                h = h
                    .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                    .wrapping_add(pu.x.to_bits() as u64);
            }
        }
        h
    }
}
