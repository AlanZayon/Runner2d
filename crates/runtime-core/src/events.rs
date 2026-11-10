//! Compact game-event ring for host SFX / feedback (Phase 8).

pub const EVENT_RING_CAP: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum GameEventKind {
    Jump = 1,
    Land = 2,
    Dash = 3,
    Roll = 4,
    Coin = 5,
    Death = 6,
    AbilityOn = 7,
    FallEnter = 8,
    FallExit = 9,
    QteTick = 10,
    Destroy = 11,
}

#[derive(Clone, Copy, Debug)]
pub struct GameEvent {
    pub kind: GameEventKind,
    pub x: f32,
    pub y: f32,
    pub data: u32,
}

#[derive(Clone, Debug)]
pub struct EventQueue {
    buf: [Option<GameEvent>; EVENT_RING_CAP],
    head: usize,
    len: usize,
}

impl Default for EventQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl EventQueue {
    pub fn new() -> Self {
        Self {
            buf: [None; EVENT_RING_CAP],
            head: 0,
            len: 0,
        }
    }

    pub fn clear(&mut self) {
        self.buf = [None; EVENT_RING_CAP];
        self.head = 0;
        self.len = 0;
    }

    pub fn push(&mut self, ev: GameEvent) {
        if self.len == EVENT_RING_CAP {
            // Drop oldest.
            self.head = (self.head + 1) % EVENT_RING_CAP;
            self.len -= 1;
        }
        let idx = (self.head + self.len) % EVENT_RING_CAP;
        self.buf[idx] = Some(ev);
        self.len += 1;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    /// Drain up to `cap` events into `out`; returns number written.
    pub fn drain_into(&mut self, out: &mut [GameEvent]) -> usize {
        let n = self.len.min(out.len());
        for i in 0..n {
            let idx = (self.head + i) % EVENT_RING_CAP;
            out[i] = self.buf[idx].take().unwrap_or(GameEvent {
                kind: GameEventKind::Jump,
                x: 0.0,
                y: 0.0,
                data: 0,
            });
        }
        self.head = (self.head + n) % EVENT_RING_CAP;
        self.len -= n;
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overflow_drops_oldest() {
        let mut q = EventQueue::new();
        for i in 0..(EVENT_RING_CAP + 5) {
            q.push(GameEvent {
                kind: GameEventKind::Coin,
                x: i as f32,
                y: 0.0,
                data: i as u32,
            });
        }
        assert_eq!(q.len(), EVENT_RING_CAP);
        let mut out = [GameEvent {
            kind: GameEventKind::Jump,
            x: 0.0,
            y: 0.0,
            data: 0,
        }; EVENT_RING_CAP];
        let n = q.drain_into(&mut out);
        assert_eq!(n, EVENT_RING_CAP);
        assert_eq!(out[0].data, 5); // first five dropped
        assert_eq!(q.len(), 0);
    }
}
