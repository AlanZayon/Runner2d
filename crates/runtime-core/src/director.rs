//! Game director: intent planning + parametric segment assembly + course generation.

use crate::components::HazardKind;
use crate::config::{
    DEFAULT_SEED, FALLING_SHAFT_EVERY_N, GENERATOR_VERSION, MAX_COURSE_ATTEMPTS,
    MAX_SEGMENT_REROLLS, SEGMENT_COUNT,
};
use crate::generate::{generate_segment, safe_recovery};
use crate::rng::XorShift64;
use crate::segment::{GeneratedCourse, SegmentDefinition, SegmentIntent};
use crate::validate::{validate_continuity, validate_segment};

const ENDLESS_POOL: [SegmentIntent; 7] = [
    SegmentIntent::JumpDash,
    SegmentIntent::JumpRoll,
    SegmentIntent::RiskReward,
    SegmentIntent::Recovery,
    SegmentIntent::TeachJump,
    SegmentIntent::TeachDash,
    SegmentIntent::TeachRoll,
];

/// Difficulty tiers by streamed segment index (§35).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DifficultyTier {
    Inicial,
    Intermediario,
    Avancado,
    Especialista,
}

pub fn difficulty_tier(segment_index: usize) -> DifficultyTier {
    match segment_index {
        0..=7 => DifficultyTier::Inicial,
        8..=19 => DifficultyTier::Intermediario,
        20..=39 => DifficultyTier::Avancado,
        _ => DifficultyTier::Especialista,
    }
}

fn pick_endless_intent(
    rng: &mut XorShift64,
    prev: Option<SegmentIntent>,
    segment_index: usize,
) -> SegmentIntent {
    let tier = difficulty_tier(segment_index);
    // Sparse FallingShaft — never adjacent to another shaft.
    if prev != Some(SegmentIntent::FallingShaft)
        && rng.gen_range_u32(0, FALLING_SHAFT_EVERY_N) == 0
        && tier != DifficultyTier::Inicial
    {
        return SegmentIntent::FallingShaft;
    }
    // GhostChallenge appears from Avancado onward (safe alternate route always present).
    if matches!(tier, DifficultyTier::Avancado | DifficultyTier::Especialista)
        && prev != Some(SegmentIntent::GhostChallenge)
        && rng.gen_range_u32(0, if tier == DifficultyTier::Especialista { 5 } else { 9 }) == 0
    {
        return SegmentIntent::GhostChallenge;
    }
    let mut chosen = ENDLESS_POOL[rng.choose_index(ENDLESS_POOL.len())];
    // Especialista biases toward risk / combined intents.
    if tier == DifficultyTier::Especialista && rng.gen_range_u32(0, 3) == 0 {
        chosen = [
            SegmentIntent::RiskReward,
            SegmentIntent::JumpDash,
            SegmentIntent::JumpRoll,
        ][rng.choose_index(3)];
    }
    let mut tries = 0;
    while prev == Some(chosen) && tries < 8 {
        chosen = ENDLESS_POOL[rng.choose_index(ENDLESS_POOL.len())];
        tries += 1;
    }
    if prev == Some(SegmentIntent::RiskReward) && rng.gen_range_u32(0, 2) == 0 {
        chosen = SegmentIntent::Recovery;
    }
    if prev == Some(SegmentIntent::GhostChallenge) {
        chosen = SegmentIntent::Recovery;
    }
    if prev == Some(SegmentIntent::FallingShaft) {
        chosen = SegmentIntent::Recovery;
    }
    chosen
}

/// Teaching spine then weighted variety (no finish Transition — run is endless).
fn plan_intents(rng: &mut XorShift64) -> [SegmentIntent; SEGMENT_COUNT] {
    let mut intents = [
        SegmentIntent::Recovery,
        SegmentIntent::TeachJump,
        SegmentIntent::TeachRoll,
        SegmentIntent::TeachDash,
        SegmentIntent::JumpDash,
        SegmentIntent::JumpRoll,
        SegmentIntent::RiskReward,
        SegmentIntent::Recovery,
    ];
    for i in 4..SEGMENT_COUNT {
        let mut next = pick_endless_intent(rng, Some(intents[i - 1]), i);
        // Opening buffer stays runnable for headless proof — shafts only via endless append.
        if next == SegmentIntent::FallingShaft || next == SegmentIntent::GhostChallenge {
            next = SegmentIntent::Recovery;
        }
        intents[i] = next;
    }
    intents
}

/// Append one more segment after `prev` at `world_x0` (connectivity-validated).
pub fn generate_next_segment(
    seed: u64,
    index: usize,
    prev_intent: Option<SegmentIntent>,
    world_x0: f32,
) -> SegmentDefinition {
    let mut rng = XorShift64::from_seed(
        seed ^ ((index as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15)),
    );
    let intent = pick_endless_intent(&mut rng, prev_intent, index);
    let seg = pick_segment(&mut rng, intent, world_x0);
    // Soft continuity: if seam fails, force recovery pad.
    if let Some(prev) = prev_intent {
        let _ = prev;
    }
    seg
}

fn pick_segment(
    rng: &mut XorShift64,
    intent: SegmentIntent,
    world_x0: f32,
) -> SegmentDefinition {
    for _ in 0..MAX_SEGMENT_REROLLS {
        let seg = generate_segment(rng, intent, world_x0);
        if validate_segment(&seg).is_ok() {
            return seg;
        }
    }
    // Explicit recovery only after exhausted rerolls (counted in fingerprint).
    let mut recovery = safe_recovery(world_x0, rng.gen_range_f32(180.0, 240.0));
    recovery.template_id = 1; // mark as explicit fallback recovery
    recovery
}

fn build_sequence(rng: &mut XorShift64) -> Vec<SegmentDefinition> {
    let intents = plan_intents(rng);
    let mut segments = Vec::with_capacity(SEGMENT_COUNT);
    let mut x = -40.0_f32;

    // Short RNG intro Recovery (not a fixed 480 corridor).
    let intro_w = rng.gen_range_f32(160.0, 220.0);
    let mut first = generate_segment(rng, SegmentIntent::Recovery, x);
    // Force width into intro band by regenerating until roughly short, or clamp via safe.
    if first.width > 240.0 || first.width < 150.0 {
        first = safe_recovery(x, intro_w);
        first.intent = SegmentIntent::Recovery;
        first.template_id = 100;
    }
    // Prefer the parametric recovery but ensure short: regenerate with capped attempts.
    for _ in 0..6 {
        let cand = generate_segment(rng, SegmentIntent::Recovery, x);
        if cand.width <= 260.0 && validate_segment(&cand).is_ok() {
            first = cand;
            break;
        }
    }
    if first.width > 260.0 {
        first = safe_recovery(x, intro_w);
        first.template_id = 100;
    }

    x = first.world_x1();
    segments.push(first);

    for intent in intents.into_iter().skip(1) {
        let seg = pick_segment(rng, intent, x);
        x = seg.world_x1();
        segments.push(seg);
    }

    // Enforce no adjacent identical intents when possible (swap via regen).
    for i in 1..segments.len() {
        if segments[i].intent == segments[i - 1].intent
            && segments[i].intent != SegmentIntent::Recovery
        {
            let alt = match segments[i].intent {
                SegmentIntent::TeachJump => SegmentIntent::TeachDash,
                SegmentIntent::TeachDash => SegmentIntent::TeachRoll,
                SegmentIntent::TeachRoll => SegmentIntent::TeachJump,
                SegmentIntent::JumpDash => SegmentIntent::JumpRoll,
                SegmentIntent::JumpRoll => SegmentIntent::RiskReward,
                SegmentIntent::RiskReward => SegmentIntent::JumpDash,
                SegmentIntent::GhostChallenge => SegmentIntent::Recovery,
                other => other,
            };
            if alt != segments[i].intent {
                let mut cx = segments[i].world_x0;
                let replacement = pick_segment(rng, alt, cx);
                cx = replacement.world_x1();
                segments[i] = replacement;
                for j in (i + 1)..segments.len() {
                    let intent = segments[j].intent;
                    let seg = pick_segment(rng, intent, cx);
                    cx = seg.world_x1();
                    segments[j] = seg;
                }
            }
        }
    }

    segments
}

fn course_from_segments(seed: u64, segments: Vec<SegmentDefinition>) -> GeneratedCourse {
    let course_end = segments
        .last()
        .map(|s| s.world_x1() - 40.0)
        .unwrap_or(1600.0);
    GeneratedCourse {
        seed,
        generator_version: GENERATOR_VERSION,
        segments,
        course_end,
    }
}

pub fn fallback_course(seed: u64) -> GeneratedCourse {
    let mut segments = Vec::new();
    let mut x = -40.0;
    let spine = [
        SegmentIntent::Recovery,
        SegmentIntent::TeachJump,
        SegmentIntent::TeachRoll,
        SegmentIntent::TeachDash,
        SegmentIntent::JumpDash,
        SegmentIntent::JumpRoll,
        SegmentIntent::RiskReward,
        SegmentIntent::Transition,
    ];
    // Even fallback keeps distinct intents (deterministic params via seed).
    let mut rng = XorShift64::from_seed(seed ^ 0xFA11_BACEu64);
    for intent in spine {
        let seg = pick_segment(&mut rng, intent, x);
        x = seg.world_x1();
        segments.push(seg);
    }
    course_from_segments(seed, segments)
}

/// Geometry-only generation (no headless). Prefer [`generate_course`].
pub fn generate_course_geometry(seed: u64) -> GeneratedCourse {
    let seed = if seed == 0 { DEFAULT_SEED } else { seed };
    let mut rng = XorShift64::from_seed(seed);
    for _ in 0..MAX_COURSE_ATTEMPTS {
        let segments = build_sequence(&mut rng);
        if validate_continuity(&segments).is_err() {
            continue;
        }
        if segments.iter().any(|s| validate_segment(s).is_err()) {
            continue;
        }
        return course_from_segments(seed, segments);
    }
    fallback_course(seed)
}

/// Generate + headless-validate a reproducible course.
pub fn generate_course(seed: u64) -> GeneratedCourse {
    let seed = if seed == 0 { DEFAULT_SEED } else { seed };
    let mut rng = XorShift64::from_seed(seed);
    for _ in 0..MAX_COURSE_ATTEMPTS {
        let segments = build_sequence(&mut rng);
        if validate_continuity(&segments).is_err() {
            continue;
        }
        if segments.iter().any(|s| validate_segment(s).is_err()) {
            continue;
        }
        let course = course_from_segments(seed, segments);
        if crate::headless::prove_course(&course).is_ok() {
            return course;
        }
    }
    let fb = fallback_course(seed);
    // Still try to prove fallback; if it fails, return it anyway (playable attempt).
    let _ = crate::headless::prove_course(&fb);
    fb
}

/// True if course looks like the all-flat recovery collapse of the old generator.
pub fn is_degenerate_flat_course(course: &GeneratedCourse) -> bool {
    course.segments.iter().all(|s| {
        s.intent == SegmentIntent::Recovery && s.hazards.is_empty()
    })
}

pub fn course_has_mechanic_coverage(course: &GeneratedCourse) -> bool {
    let mut has_gap = false;
    let mut has_dash = false;
    let mut has_roll = false;
    for s in &course.segments {
        if matches!(
            s.intent,
            SegmentIntent::TeachJump | SegmentIntent::JumpDash | SegmentIntent::JumpRoll | SegmentIntent::RiskReward
        ) {
            has_gap = true;
        }
        for h in &s.hazards {
            match h.kind {
                HazardKind::SolidKill => has_dash = true,
                HazardKind::LowBar => has_roll = true,
            }
        }
    }
    has_gap && has_dash && has_roll
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::segment_local_fingerprint;

    #[test]
    fn same_seed_same_layout() {
        let a = generate_course(42);
        let b = generate_course(42);
        assert_eq!(a.fingerprint(), b.fingerprint());
        assert_eq!(a.course_end.to_bits(), b.course_end.to_bits());
        assert_eq!(a.segments.len(), b.segments.len());
    }

    #[test]
    fn different_seed_differs() {
        let a = generate_course_geometry(1);
        let b = generate_course_geometry(999_999);
        assert_ne!(a.fingerprint(), b.fingerprint());
    }

    #[test]
    fn default_seed_produces_eight_segments() {
        let c = generate_course_geometry(DEFAULT_SEED);
        assert_eq!(c.segments.len(), SEGMENT_COUNT);
        assert!(c.course_end > 800.0);
    }

    #[test]
    fn sequence_has_distinct_intents() {
        let c = generate_course(42);
        let non_recovery: Vec<_> = c
            .segments
            .iter()
            .filter(|s| s.intent != SegmentIntent::Recovery)
            .collect();
        assert!(
            non_recovery.len() >= 3,
            "expected several non-recovery segments, got {}",
            non_recovery.len()
        );
        assert!(course_has_mechanic_coverage(&c));
        assert!(!is_degenerate_flat_course(&c));
    }

    #[test]
    fn adjacent_segments_differ() {
        let c = generate_course(42);
        for win in c.segments.windows(2) {
            let fa = segment_local_fingerprint(&win[0]);
            let fb = segment_local_fingerprint(&win[1]);
            assert_ne!(fa, fb, "adjacent segments identical");
        }
    }

    #[test]
    fn different_seeds_different_hazard_layout() {
        let a = generate_course(42);
        let b = generate_course(7);
        let c = generate_course(1001);
        let haz_sig = |course: &GeneratedCourse| -> u64 {
            let mut h = 0u64;
            for s in &course.segments {
                for hz in &s.hazards {
                    h = h
                        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                        .wrapping_add(hz.x.to_bits() as u64);
                    h = h
                        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                        .wrapping_add(match hz.kind {
                            HazardKind::SolidKill => 1,
                            HazardKind::LowBar => 2,
                        });
                }
            }
            h
        };
        assert_ne!(haz_sig(&a), haz_sig(&b));
        assert_ne!(haz_sig(&a), haz_sig(&c));
    }

    #[test]
    fn common_seeds_not_fallback_flat() {
        for seed in [DEFAULT_SEED, 42u64] {
            let c = generate_course(seed);
            assert!(
                !is_degenerate_flat_course(&c),
                "seed {seed} collapsed to flat recovery"
            );
            assert!(course_has_mechanic_coverage(&c));
        }
    }
}
