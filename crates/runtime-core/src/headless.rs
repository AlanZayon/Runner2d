//! Headless path proof using the real World simulation + scripted policy.

use crate::components::HazardKind;
use crate::config::{AUTO_RUN_SPEED, GROUND_Y};
use crate::segment::{GeneratedCourse, SegmentDefinition};
use crate::validate::ground_spans;
use crate::world::{InputSnapshot, World};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProveError {
    Died { ticks: u32 },
    Timeout,
}

#[derive(Clone, Copy, Debug)]
enum Cue {
    JumpUntil { x_end: f32 },
    RollUntil { x_end: f32 },
    DashAt { x: f32 },
}

fn build_cues(segments: &[SegmentDefinition]) -> Vec<(f32, Cue)> {
    // (trigger_x, cue) sorted by trigger
    let mut cues = Vec::new();
    for seg in segments {
        let spans = ground_spans(&seg.platforms);
        for win in spans.windows(2) {
            let gap_start = win[0].1;
            let gap_end = win[1].0;
            if gap_end - gap_start > 20.0 {
                // Start jumping shortly before leaving the ledge.
                let trigger = gap_start - 35.0;
                cues.push((
                    trigger,
                    Cue::JumpUntil {
                        x_end: gap_end + 20.0,
                    },
                ));
            }
        }
        for h in &seg.hazards {
            match h.kind {
                HazardKind::LowBar => {
                    // Start close enough that ROLL_DURATION covers the full bar width.
                    cues.push((
                        h.x - 40.0,
                        Cue::RollUntil {
                            x_end: h.x + h.w + 30.0,
                        },
                    ));
                }
                HazardKind::SolidKill => {
                    // Fire just before contact so DASH_DURATION covers the pillar.
                    cues.push((h.x - 30.0, Cue::DashAt { x: h.x - 5.0 }));
                }
            }
        }
    }
    cues.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    cues
}

pub fn prove_course(course: &GeneratedCourse) -> Result<(), ProveError> {
    let cues = build_cues(&course.segments);
    let mut world = World::from_course_fully_spawned(course);
    let dt = 1.0 / 60.0;
    let max_ticks = ((course.course_end / AUTO_RUN_SPEED) / dt * 2.2) as u32 + 900;
    let mut was_jumping = false;
    let mut fired_dash: Vec<bool> = cues
        .iter()
        .map(|(_, c)| matches!(c, Cue::DashAt { .. }))
        .map(|_| false)
        .collect();
    // Mark non-dash cues as already "fired" so we only gate DashAt.
    for (i, (_, c)) in cues.iter().enumerate() {
        if !matches!(c, Cue::DashAt { .. }) {
            fired_dash[i] = true; // unused
        } else {
            fired_dash[i] = false;
        }
    }

    for tick in 0..max_ticks {
        let rs = world.run_state();
        if rs.finished {
            return Ok(());
        }
        if !rs.alive {
            return Err(ProveError::Died { ticks: tick });
        }
        let t = world.player_transform().unwrap();
        let c = world.player_collider().unwrap();
        let nose = t.x + c.w;
        let mut input = InputSnapshot::default();

        let mut want_roll = false;
        for (i, &(trig, cue)) in cues.iter().enumerate() {
            match cue {
                Cue::JumpUntil { x_end } => {
                    if nose >= trig && nose <= x_end {
                        input.jump_held = true;
                    }
                }
                Cue::RollUntil { x_end } => {
                    if nose >= trig && nose <= x_end {
                        want_roll = true;
                    }
                }
                Cue::DashAt { x } => {
                    // One-shot: fire once in a tight band just before the pillar.
                    if !fired_dash[i] && nose >= trig && nose <= x + 25.0 {
                        input.dash_pressed = true;
                        fired_dash[i] = true;
                    }
                }
            }
        }
        // Roll windows win over jump (needed after landing into a LowBar).
        if want_roll {
            input.roll_held = true;
            input.jump_held = false;
            was_jumping = false;
        } else if input.jump_held {
            input.jump_pressed = !was_jumping;
            was_jumping = true;
        } else {
            was_jumping = false;
        }

        world.tick(input, dt);
        if !world.run_state().alive {
            return Err(ProveError::Died { ticks: tick + 1 });
        }
        let x = world.player_transform().map(|tr| tr.x).unwrap_or(0.0);
        if x >= course.course_end {
            return Ok(());
        }
    }
    let _ = GROUND_Y;
    Err(ProveError::Timeout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DEFAULT_SEED;
    use crate::director::{generate_course, generate_course_geometry};

    #[test]
    fn headless_proves_geometry_seed_42() {
        let course = generate_course_geometry(42);
        let r = prove_course(&course);
        assert!(r.is_ok(), "geometry 42 prove failed: {r:?}");
    }

    #[test]
    fn headless_proves_default_seed() {
        let course = generate_course(DEFAULT_SEED);
        let r = prove_course(&course);
        assert!(r.is_ok(), "DEFAULT prove failed: {r:?}");
    }

    #[test]
    fn headless_proves_seed_42() {
        let course = generate_course(42);
        let r = prove_course(&course);
        assert!(r.is_ok(), "seed 42 prove failed: {r:?}");
    }
}
