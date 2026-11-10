//! Falling Mode state machine (Phase 7) — special vertical drop, not airborne Running.

use crate::config::{FALLING_ENTER_DURATION, FALLING_EXIT_DURATION, FALLING_MAX_DURATION};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RunMode {
    #[default]
    Run,
    FallingEnter,
    Falling,
    FallingExit,
}

impl RunMode {
    pub fn is_falling_like(self) -> bool {
        matches!(
            self,
            RunMode::FallingEnter | RunMode::Falling | RunMode::FallingExit
        )
    }

    pub fn uses_strafe(self) -> bool {
        matches!(self, RunMode::FallingEnter | RunMode::Falling)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FallingState {
    pub timer: f32,
    pub mode_timer: f32,
    pub shaft_x0: f32,
    pub shaft_x1: f32,
    pub floor_y: f32,
    pub enter_y: f32,
    pub exit_x: f32,
    pub exit_y: f32,
}

impl FallingState {
    pub fn new(
        shaft_x0: f32,
        shaft_x1: f32,
        floor_y: f32,
        enter_y: f32,
        exit_x: f32,
        exit_y: f32,
    ) -> Self {
        Self {
            timer: 0.0,
            mode_timer: 0.0,
            shaft_x0,
            shaft_x1,
            floor_y,
            enter_y,
            exit_x,
            exit_y,
        }
    }

    pub fn timed_out(&self) -> bool {
        self.mode_timer >= FALLING_MAX_DURATION
    }

    pub fn enter_done(&self) -> bool {
        self.timer >= FALLING_ENTER_DURATION
    }

    pub fn exit_done(&self) -> bool {
        self.timer >= FALLING_EXIT_DURATION
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strafe_only_while_falling() {
        assert!(RunMode::Falling.uses_strafe());
        assert!(!RunMode::Run.uses_strafe());
        assert!(!RunMode::FallingExit.uses_strafe());
    }
}
