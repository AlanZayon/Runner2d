//! Animation clips separate from PlayerController (§18).

use crate::components::PlayerState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum AnimClip {
    Idle = 0,
    Run = 1,
    Jump = 2,
    Fall = 3,
    Dash = 4,
    Roll = 5,
    Death = 6,
}

/// Horizontal frame strip: 4 frames per clip row (placeholder atlas layout).
pub const FRAMES_PER_CLIP: u32 = 4;
pub const CLIP_ROWS: u32 = 7;

#[derive(Clone, Debug)]
pub struct AnimController {
    pub clip: AnimClip,
    pub frame: u32,
    pub timer: f32,
    pub fps: f32,
}

impl Default for AnimController {
    fn default() -> Self {
        Self {
            clip: AnimClip::Idle,
            frame: 0,
            timer: 0.0,
            fps: 10.0,
        }
    }
}

impl AnimController {
    pub fn sync_from_player_state(&mut self, state: PlayerState, grounded: bool, dt: f32) {
        let desired = match state {
            PlayerState::Dead => AnimClip::Death,
            PlayerState::Dashing => AnimClip::Dash,
            PlayerState::Rolling => AnimClip::Roll,
            PlayerState::Jumping => AnimClip::Jump,
            PlayerState::Falling => AnimClip::Fall,
            PlayerState::Running => {
                if grounded {
                    AnimClip::Run
                } else {
                    AnimClip::Fall
                }
            }
        };
        if desired != self.clip {
            self.clip = desired;
            self.frame = 0;
            self.timer = 0.0;
        }
        self.timer += dt;
        let frame_dur = 1.0 / self.fps.max(1.0);
        while self.timer >= frame_dur {
            self.timer -= frame_dur;
            self.frame = (self.frame + 1) % FRAMES_PER_CLIP;
        }
    }

    /// UV rect for the active clip/frame.
    /// Shipping art is still a single full-frame sprite (`demo/sprite.png`), so we keep
    /// full-quad UVs. Clip/frame remain available for a future atlas without breaking draw.
    pub fn uv_rect(&self) -> (f32, f32, f32, f32) {
        let _ = (self.clip, self.frame);
        (0.0, 0.0, 1.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_switches_on_state() {
        let mut a = AnimController::default();
        a.sync_from_player_state(PlayerState::Dashing, false, 0.05);
        assert_eq!(a.clip, AnimClip::Dash);
        let (u0, v0, u1, v1) = a.uv_rect();
        assert_eq!((u0, v0, u1, v1), (0.0, 0.0, 1.0, 1.0));
    }
}
