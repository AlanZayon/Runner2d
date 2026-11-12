//! Geometric + intent-content segment validation (no GPU).

use crate::components::HazardKind;
use crate::config::{
    GROUND_Y, LOW_BAR_HEIGHT, MAX_DASH_THROUGH_W, MAX_SAFE_GAP, PLAYER_H, PLAYER_W,
    ROLL_HEIGHT_SCALE,
};
use crate::segment::{PlatformSpec, SegmentDefinition, SegmentIntent};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidateError {
    EmptyPlatforms,
    GapTooWide { gap: u32 },
    LowBarTooTall,
    SolidTooWide,
    NoGroundCoverage,
    EntryExitMismatch,
    IntentMissingContent,
}

pub type ValidateResult = Result<(), ValidateError>;

/// Ground platforms near `ground_y` (±2) covering the segment floor.
pub fn ground_spans_at(platforms: &[PlatformSpec], ground_y: f32) -> Vec<(f32, f32)> {
    let mut spans: Vec<(f32, f32)> = platforms
        .iter()
        .filter(|p| (p.y - ground_y).abs() < 2.0 && p.h >= 20.0)
        .map(|p| (p.x, p.x + p.w))
        .collect();
    spans.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut merged: Vec<(f32, f32)> = Vec::new();
    for (a, b) in spans {
        if let Some(last) = merged.last_mut() {
            if a <= last.1 + 1.0 {
                last.1 = last.1.max(b);
                continue;
            }
        }
        merged.push((a, b));
    }
    merged
}

pub fn ground_spans(platforms: &[PlatformSpec]) -> Vec<(f32, f32)> {
    ground_spans_at(platforms, GROUND_Y)
}

pub fn max_gap_in_segment(seg: &SegmentDefinition) -> f32 {
    let spans = ground_spans_at(&seg.platforms, seg.ground_y);
    if spans.is_empty() {
        return seg.width;
    }
    let mut max_gap = 0.0_f32;
    if spans[0].0 > seg.world_x0 + 1.0 {
        max_gap = max_gap.max(spans[0].0 - seg.world_x0);
    }
    for win in spans.windows(2) {
        let gap = win[1].0 - win[0].1;
        if gap > 0.0 {
            max_gap = max_gap.max(gap);
        }
    }
    let end = seg.world_x1();
    let last = spans.last().unwrap().1;
    if end > last + 1.0 {
        max_gap = max_gap.max(end - last);
    }
    max_gap
}

fn validate_intent_content(seg: &SegmentDefinition) -> ValidateResult {
    let gap = max_gap_in_segment(seg);
    let solids = seg
        .hazards
        .iter()
        .filter(|h| h.kind == HazardKind::SolidKill)
        .count();
    let lows = seg
        .hazards
        .iter()
        .filter(|h| h.kind == HazardKind::LowBar)
        .count();

    match seg.intent {
        SegmentIntent::TeachJump | SegmentIntent::RiskReward => {
            if gap < 40.0 {
                return Err(ValidateError::IntentMissingContent);
            }
        }
        SegmentIntent::TeachDash => {
            if solids < 1 {
                return Err(ValidateError::IntentMissingContent);
            }
        }
        SegmentIntent::TeachRoll => {
            if lows < 1 {
                return Err(ValidateError::IntentMissingContent);
            }
        }
        SegmentIntent::JumpDash => {
            if gap < 40.0 || solids < 1 {
                return Err(ValidateError::IntentMissingContent);
            }
        }
        SegmentIntent::JumpRoll => {
            if gap < 40.0 || lows < 1 {
                return Err(ValidateError::IntentMissingContent);
            }
        }
        SegmentIntent::Recovery | SegmentIntent::Transition => {}
        SegmentIntent::GhostChallenge => {
            let always = seg
                .hazards
                .iter()
                .filter(|h| h.category == crate::collision_cat::CollisionCategory::AlwaysSolid)
                .count();
            let ghost_pass = seg
                .hazards
                .iter()
                .filter(|h| h.category == crate::collision_cat::CollisionCategory::GhostPassable)
                .count();
            if always < 1 || ghost_pass < 1 || gap < 40.0 {
                return Err(ValidateError::IntentMissingContent);
            }
        }
        SegmentIntent::FallingShaft => {
            let Some(shaft) = seg.shaft else {
                return Err(ValidateError::IntentMissingContent);
            };
            if shaft.pit_x1 <= shaft.pit_x0 + 40.0 || shaft.floor_y <= shaft.enter_y + 80.0 {
                return Err(ValidateError::IntentMissingContent);
            }
            if (shaft.exit_y - shaft.floor_y).abs() > 2.0 {
                return Err(ValidateError::IntentMissingContent);
            }
            // Entry lip + deep floor/runway + walls.
            if seg.platforms.len() < 3 {
                return Err(ValidateError::IntentMissingContent);
            }
        }
    }
    Ok(())
}

pub fn validate_segment(seg: &SegmentDefinition) -> ValidateResult {
    if seg.platforms.is_empty() {
        return Err(ValidateError::EmptyPlatforms);
    }

    let spans = ground_spans_at(&seg.platforms, seg.ground_y);
    if spans.is_empty() {
        return Err(ValidateError::NoGroundCoverage);
    }

    // FallingShaft intentionally has a wide ground gap; validated via ShaftSpec instead.
    if seg.intent != SegmentIntent::FallingShaft {
        let gap = max_gap_in_segment(seg);
        if gap > MAX_SAFE_GAP + 0.5 {
            return Err(ValidateError::GapTooWide {
                gap: gap.round() as u32,
            });
        }
    }

    let _roll_h = PLAYER_H * ROLL_HEIGHT_SCALE;
    for h in &seg.hazards {
        match h.kind {
            HazardKind::LowBar => {
                if h.h > LOW_BAR_HEIGHT + 4.0 {
                    return Err(ValidateError::LowBarTooTall);
                }
            }
            HazardKind::SolidKill => {
                // Air obstacles in shafts may be wider than dash-through pillars.
                if seg.intent != SegmentIntent::FallingShaft
                    && seg.intent != SegmentIntent::GhostChallenge
                    && h.w > MAX_DASH_THROUGH_W
                {
                    return Err(ValidateError::SolidTooWide);
                }
            }
        }
    }

    validate_intent_content(seg)
}

pub fn validate_continuity(segments: &[SegmentDefinition]) -> ValidateResult {
    if segments.is_empty() {
        return Err(ValidateError::EmptyPlatforms);
    }
    for win in segments.windows(2) {
        let a = &win[0];
        let b = &win[1];
        if (b.world_x0 - a.world_x1()).abs() > 1.0 {
            return Err(ValidateError::EntryExitMismatch);
        }
        // Continuity is checked on the shared exit/entry altitude.
        // FallingShaft enters at enter_y and exits at floor_y (altitude drop).
        let a_gy = a.exit_ground_y();
        let b_gy = b.entry_ground_y();
        if (a_gy - b_gy).abs() > 2.0 {
            return Err(ValidateError::EntryExitMismatch);
        }
        let a_spans = ground_spans_at(&a.platforms, a_gy);
        let b_spans = ground_spans_at(&b.platforms, b_gy);
        let a_end = a_spans.last().map(|s| s.1).unwrap_or(a.world_x0);
        let b_start = b_spans.first().map(|s| s.0).unwrap_or(b.world_x1());
        let seam_gap = b_start - a_end;
        if seam_gap > MAX_SAFE_GAP + 0.5 {
            return Err(ValidateError::GapTooWide {
                gap: seam_gap.round() as u32,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
pub fn impossible_gap_segment(world_x0: f32) -> SegmentDefinition {
    use crate::segment::PlatformSpec;
    SegmentDefinition {
        intent: SegmentIntent::TeachJump,
        template_id: 999,
        world_x0,
        width: 400.0,
        ground_y: GROUND_Y,
        platforms: vec![
            PlatformSpec {
                x: world_x0,
                y: GROUND_Y,
                w: 40.0,
                h: 40.0,
                tint: (0.2, 0.2, 0.2),
            },
            PlatformSpec {
                x: world_x0 + 40.0 + MAX_SAFE_GAP + 40.0,
                y: GROUND_Y,
                w: 80.0,
                h: 40.0,
                tint: (0.2, 0.2, 0.2),
            },
        ],
        hazards: vec![],
        coins: vec![],
        powerups: vec![],
        shaft: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::generate_segment;
    use crate::rng::XorShift64;

    #[test]
    fn recovery_valid() {
        let mut rng = XorShift64::from_seed(1);
        let seg = generate_segment(&mut rng, SegmentIntent::Recovery, 0.0, GROUND_Y);
        assert!(validate_segment(&seg).is_ok());
    }

    #[test]
    fn validator_rejects_impossible_gap() {
        let seg = impossible_gap_segment(0.0);
        match validate_segment(&seg) {
            Err(ValidateError::GapTooWide { .. }) => {}
            other => panic!("expected GapTooWide, got {other:?}"),
        }
    }

    #[test]
    fn teach_dash_requires_solid() {
        let mut seg = generate_segment(
            &mut XorShift64::from_seed(2),
            SegmentIntent::TeachDash,
            0.0,
            GROUND_Y,
        );
        seg.hazards.clear();
        assert_eq!(
            validate_segment(&seg),
            Err(ValidateError::IntentMissingContent)
        );
    }

    #[test]
    fn parametric_intents_pass() {
        let mut rng = XorShift64::from_seed(77);
        for intent in [
            SegmentIntent::TeachJump,
            SegmentIntent::TeachDash,
            SegmentIntent::TeachRoll,
            SegmentIntent::JumpDash,
            SegmentIntent::JumpRoll,
            SegmentIntent::RiskReward,
            SegmentIntent::Transition,
            SegmentIntent::FallingShaft,
            SegmentIntent::GhostChallenge,
        ] {
            let seg = generate_segment(&mut rng, intent, 100.0, GROUND_Y);
            assert!(
                validate_segment(&seg).is_ok(),
                "{intent:?}: {:?}",
                validate_segment(&seg)
            );
        }
    }
}

#[allow(dead_code)]
fn _keep_player_w() -> f32 {
    PLAYER_W
}
