//! Legacy thin wrappers — parametric generation lives in `generate`.

use crate::generate::{generate_segment, safe_recovery as gen_safe};
use crate::rng::XorShift64;
use crate::segment::{SegmentDefinition, SegmentIntent};

pub use crate::generate::safe_recovery;

#[derive(Clone, Copy)]
pub struct TemplateMeta {
    pub id: u32,
    pub intent: SegmentIntent,
    pub width: f32,
}

static RECOVERY: [TemplateMeta; 1] = [TemplateMeta {
    id: 100,
    intent: SegmentIntent::Recovery,
    width: 220.0,
}];
static TEACH_JUMP: [TemplateMeta; 1] = [TemplateMeta {
    id: 110,
    intent: SegmentIntent::TeachJump,
    width: 240.0,
}];
static TEACH_DASH: [TemplateMeta; 1] = [TemplateMeta {
    id: 120,
    intent: SegmentIntent::TeachDash,
    width: 220.0,
}];
static TEACH_ROLL: [TemplateMeta; 1] = [TemplateMeta {
    id: 130,
    intent: SegmentIntent::TeachRoll,
    width: 220.0,
}];
static JUMP_DASH: [TemplateMeta; 1] = [TemplateMeta {
    id: 140,
    intent: SegmentIntent::JumpDash,
    width: 260.0,
}];
static JUMP_ROLL: [TemplateMeta; 1] = [TemplateMeta {
    id: 150,
    intent: SegmentIntent::JumpRoll,
    width: 260.0,
}];
static RISK_REWARD: [TemplateMeta; 1] = [TemplateMeta {
    id: 160,
    intent: SegmentIntent::RiskReward,
    width: 280.0,
}];
static TRANSITION: [TemplateMeta; 1] = [TemplateMeta {
    id: 170,
    intent: SegmentIntent::Transition,
    width: 240.0,
}];
static FALLING_SHAFT: [TemplateMeta; 1] = [TemplateMeta {
    id: 180,
    intent: SegmentIntent::FallingShaft,
    width: 400.0,
}];
static GHOST_CHALLENGE: [TemplateMeta; 1] = [TemplateMeta {
    id: 190,
    intent: SegmentIntent::GhostChallenge,
    width: 360.0,
}];

/// Compatibility shim: one meta entry per intent for older call sites.
pub fn templates_for(intent: SegmentIntent) -> &'static [TemplateMeta] {
    match intent {
        SegmentIntent::Recovery => &RECOVERY,
        SegmentIntent::TeachJump => &TEACH_JUMP,
        SegmentIntent::TeachDash => &TEACH_DASH,
        SegmentIntent::TeachRoll => &TEACH_ROLL,
        SegmentIntent::JumpDash => &JUMP_DASH,
        SegmentIntent::JumpRoll => &JUMP_ROLL,
        SegmentIntent::RiskReward => &RISK_REWARD,
        SegmentIntent::Transition => &TRANSITION,
        SegmentIntent::FallingShaft => &FALLING_SHAFT,
        SegmentIntent::GhostChallenge => &GHOST_CHALLENGE,
    }
}

/// Build via parametric generator (seeded from template id for stability).
pub fn build_template(meta: TemplateMeta, world_x0: f32) -> SegmentDefinition {
    let mut rng = XorShift64::from_seed(meta.id as u64 * 0x9E37 + meta.width.to_bits() as u64);
    let mut seg = generate_segment(&mut rng, meta.intent, world_x0);
    seg.template_id = meta.id;
    seg
}

pub fn safe_recovery_width(world_x0: f32, width: f32) -> SegmentDefinition {
    gen_safe(world_x0, width)
}
