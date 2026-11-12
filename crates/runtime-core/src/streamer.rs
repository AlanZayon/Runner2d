//! Segment streaming window: spawn ahead, despawn behind.
//!
//! Cost stays O(window + active), not O(total course length), so endless
//! runs do not slow down as definitions accumulate.

use crate::config::{STREAM_AHEAD, STREAM_AHEAD_PX, STREAM_BEHIND, STREAM_BEHIND_PX};
use crate::segment::SegmentDefinition;

#[derive(Clone, Debug, Default)]
pub struct SegmentStreamer {
    /// Which segment indices currently have entities spawned.
    pub active: Vec<bool>,
    /// Compact list of active indices (avoids scanning the whole course each frame).
    active_indices: Vec<usize>,
}

impl SegmentStreamer {
    pub fn new(segment_count: usize) -> Self {
        Self {
            active: vec![false; segment_count],
            active_indices: Vec::with_capacity(STREAM_AHEAD + STREAM_BEHIND + 8),
        }
    }

    /// Segments are contiguous and sorted by X — binary search, O(log n).
    pub fn segment_index_at(segments: &[SegmentDefinition], x: f32) -> usize {
        if segments.is_empty() {
            return 0;
        }
        if x >= segments[segments.len() - 1].world_x1() {
            return segments.len() - 1;
        }
        segments.partition_point(|s| s.world_x1() <= x)
    }

    fn first_overlapping(segments: &[SegmentDefinition], win_lo: f32) -> usize {
        if segments.is_empty() {
            return 0;
        }
        segments.partition_point(|s| s.world_x1() < win_lo)
    }

    fn last_overlapping(segments: &[SegmentDefinition], win_hi: f32) -> usize {
        if segments.is_empty() {
            return 0;
        }
        let i = segments.partition_point(|s| s.world_x0 <= win_hi);
        i.saturating_sub(1)
    }

    /// Returns (to_spawn, to_despawn) segment indices.
    /// Uses index window ∪ pixel window; only touches that range + current actives.
    pub fn plan(
        &self,
        segments: &[SegmentDefinition],
        player_x: f32,
    ) -> (Vec<usize>, Vec<usize>) {
        let n = segments.len();
        if n == 0 {
            return (Vec::new(), Vec::new());
        }
        let cur = Self::segment_index_at(segments, player_x);
        let min_i = cur.saturating_sub(STREAM_BEHIND);
        let max_i = (cur + STREAM_AHEAD).min(n.saturating_sub(1));
        let win_lo = player_x - STREAM_BEHIND_PX;
        let win_hi = player_x + STREAM_AHEAD_PX;
        let px_lo = Self::first_overlapping(segments, win_lo).min(n - 1);
        let px_hi = Self::last_overlapping(segments, win_hi).min(n - 1);

        let lo = min_i.min(px_lo);
        let hi = max_i.max(px_hi);

        let mut spawn = Vec::new();
        let mut despawn = Vec::new();

        for i in lo..=hi {
            let want = true;
            let is_on = self.active.get(i).copied().unwrap_or(false);
            if want && !is_on {
                spawn.push(i);
            }
        }

        for &i in &self.active_indices {
            if i < lo || i > hi {
                despawn.push(i);
            }
        }
        (spawn, despawn)
    }

    pub fn mark_active(&mut self, index: usize, active: bool) {
        if index >= self.active.len() {
            return;
        }
        let was = self.active[index];
        if was == active {
            return;
        }
        self.active[index] = active;
        if active {
            self.active_indices.push(index);
        } else if let Some(pos) = self.active_indices.iter().position(|&i| i == index) {
            self.active_indices.swap_remove(pos);
        }
    }

    pub fn push_inactive(&mut self) {
        self.active.push(false);
    }

    pub fn active_count(&self) -> usize {
        self.active_indices.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::templates::safe_recovery;

    fn make_course(n: usize) -> Vec<SegmentDefinition> {
        let mut segs = Vec::new();
        let mut x = 0.0;
        for _ in 0..n {
            let s = safe_recovery(x, 200.0, crate::config::GROUND_Y);
            x = s.world_x1();
            segs.push(s);
        }
        segs
    }

    #[test]
    fn window_spawns_ahead() {
        let segs = make_course(8);
        let streamer = SegmentStreamer::new(segs.len());
        let (spawn, despawn) = streamer.plan(&segs, 10.0);
        assert!(despawn.is_empty());
        assert!(spawn.contains(&0));
    }

    #[test]
    fn segment_index_binary_search_matches_scan() {
        let segs = make_course(64);
        for x in [0.0, 50.0, 400.0, 1999.0, 5000.0, 12_000.0] {
            let slow = {
                let mut i = 0;
                for (idx, seg) in segs.iter().enumerate() {
                    if x < seg.world_x1() {
                        i = idx;
                        break;
                    }
                    i = segs.len() - 1;
                }
                i
            };
            assert_eq!(SegmentStreamer::segment_index_at(&segs, x), slow);
        }
    }

    #[test]
    fn plan_cost_independent_of_far_history() {
        let segs = make_course(500);
        let mut streamer = SegmentStreamer::new(segs.len());
        let player = segs[400].world_x0 + 10.0;
        let (spawn, _) = streamer.plan(&segs, player);
        for i in spawn {
            streamer.mark_active(i, true);
        }
        assert!(streamer.active_count() <= STREAM_AHEAD + STREAM_BEHIND + 20);
        // Advance further: early actives must despawn; count stays bounded.
        let player2 = segs[450].world_x0 + 10.0;
        let (spawn2, despawn2) = streamer.plan(&segs, player2);
        for i in despawn2 {
            streamer.mark_active(i, false);
        }
        for i in spawn2 {
            streamer.mark_active(i, true);
        }
        assert!(streamer.active_count() <= STREAM_AHEAD + STREAM_BEHIND + 20);
        assert!(!streamer.active.get(0).copied().unwrap_or(true));
    }
}
