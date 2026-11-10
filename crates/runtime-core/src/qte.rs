//! Super Strength QTE: short prompt sequence → performance rating → duration.

use crate::config::{STRENGTH_QTE_PROMPTS, STRENGTH_QTE_WINDOW};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum QteRating {
    Poor = 0,
    Good = 1,
    Great = 2,
    Perfect = 3,
}

impl QteRating {
    pub fn from_hits(hits: u32, total: u32) -> Self {
        if total == 0 {
            return QteRating::Poor;
        }
        match hits {
            h if h >= total => QteRating::Perfect,
            h if h + 1 >= total => QteRating::Great,
            h if h * 2 >= total => QteRating::Good,
            _ => QteRating::Poor,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StrengthQte {
    pub prompts_total: u32,
    pub prompts_done: u32,
    pub hits: u32,
    /// Time left in the current prompt window.
    pub window_left: f32,
    pub awaiting_press: bool,
}

impl StrengthQte {
    pub fn new() -> Self {
        Self {
            prompts_total: STRENGTH_QTE_PROMPTS,
            prompts_done: 0,
            hits: 0,
            window_left: STRENGTH_QTE_WINDOW,
            awaiting_press: true,
        }
    }

    pub fn prompt_index(&self) -> u32 {
        self.prompts_done
    }

    /// Returns Some(rating) when the sequence completes.
    pub fn tick(&mut self, dt: f32, pressed: bool) -> Option<QteRating> {
        if self.prompts_done >= self.prompts_total {
            return Some(QteRating::from_hits(self.hits, self.prompts_total));
        }

        if pressed && self.awaiting_press {
            self.hits += 1;
            self.advance_prompt();
        } else {
            self.window_left -= dt;
            if self.window_left <= 0.0 {
                // Missed window.
                self.advance_prompt();
            }
        }

        if self.prompts_done >= self.prompts_total {
            Some(QteRating::from_hits(self.hits, self.prompts_total))
        } else {
            None
        }
    }

    fn advance_prompt(&mut self) {
        self.prompts_done += 1;
        self.window_left = STRENGTH_QTE_WINDOW;
        self.awaiting_press = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_on_all_hits() {
        let mut q = StrengthQte::new();
        let mut rating = None;
        for _ in 0..STRENGTH_QTE_PROMPTS {
            rating = q.tick(0.01, true);
        }
        assert_eq!(rating, Some(QteRating::Perfect));
    }

    #[test]
    fn poor_on_all_misses() {
        let mut q = StrengthQte::new();
        let mut rating = None;
        for _ in 0..STRENGTH_QTE_PROMPTS {
            rating = q.tick(STRENGTH_QTE_WINDOW + 0.01, false);
        }
        assert_eq!(rating, Some(QteRating::Poor));
    }

    #[test]
    fn rating_table() {
        assert_eq!(QteRating::from_hits(4, 4), QteRating::Perfect);
        assert_eq!(QteRating::from_hits(3, 4), QteRating::Great);
        assert_eq!(QteRating::from_hits(2, 4), QteRating::Good);
        assert_eq!(QteRating::from_hits(1, 4), QteRating::Poor);
    }
}
