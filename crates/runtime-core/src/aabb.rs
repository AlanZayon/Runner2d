//! Axis-aligned bounding boxes (Y grows downward).

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Aabb {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn left(self) -> f32 {
        self.x
    }
    pub fn right(self) -> f32 {
        self.x + self.w
    }
    pub fn top(self) -> f32 {
        self.y
    }
    pub fn bottom(self) -> f32 {
        self.y + self.h
    }

    pub fn overlaps(self, other: Self) -> bool {
        self.left() < other.right()
            && self.right() > other.left()
            && self.top() < other.bottom()
            && self.bottom() > other.top()
    }

    /// Resolve `self` out of `solid` by the smallest axis separation.
    /// Returns (dx, dy) applied to `self` and whether the contact was from above (landed).
    pub fn resolve_against(self, solid: Self) -> (f32, f32, bool) {
        if !self.overlaps(solid) {
            return (0.0, 0.0, false);
        }
        let overlap_x = (self.right().min(solid.right()) - self.left().max(solid.left())).abs();
        let overlap_y = (self.bottom().min(solid.bottom()) - self.top().max(solid.top())).abs();

        if overlap_x < overlap_y {
            let dx = if self.center_x() < solid.center_x() {
                -overlap_x
            } else {
                overlap_x
            };
            (dx, 0.0, false)
        } else {
            let from_above = self.center_y() < solid.center_y();
            let dy = if from_above { -overlap_y } else { overlap_y };
            (0.0, dy, from_above)
        }
    }

    pub fn center_x(self) -> f32 {
        self.x + self.w * 0.5
    }
    pub fn center_y(self) -> f32 {
        self.y + self.h * 0.5
    }

    /// Swept move of `self` by `(dx, dy)` against one solid (axis-separated + tunnel clamp).
    /// Returns applied delta and whether the contact was a landing from above.
    pub fn sweep_move(self, dx: f32, dy: f32, solid: Self) -> (f32, f32, bool) {
        let mut out_dx = dx;
        let mut out_dy = dy;
        let mut landed = false;

        // Tunneling: large +Y step that would skip through a platform top.
        if dy > 0.0
            && self.bottom() <= solid.top() + 1.0
            && self.bottom() + dy >= solid.top()
            && self.left() < solid.right()
            && self.right() > solid.left()
        {
            out_dy = solid.top() - self.bottom();
            landed = true;
            // Still allow X within the same frame after vertical clamp.
        }

        if out_dx != 0.0 {
            let next = Aabb::new(self.x + out_dx, self.y, self.w, self.h);
            if next.overlaps(solid) {
                if out_dx > 0.0 {
                    out_dx = (solid.left() - self.right()).clamp(0.0, out_dx);
                } else {
                    out_dx = (solid.right() - self.left()).clamp(out_dx, 0.0);
                }
            }
        }

        let mid = Aabb::new(self.x + out_dx, self.y, self.w, self.h);
        if out_dy != 0.0 {
            let next = Aabb::new(mid.x, mid.y + out_dy, mid.w, mid.h);
            if next.overlaps(solid) {
                if out_dy > 0.0 {
                    let clamped = (solid.top() - mid.bottom()).clamp(0.0, out_dy);
                    if mid.bottom() <= solid.top() + 2.0 || self.bottom() <= solid.top() + 2.0 {
                        landed = true;
                    }
                    out_dy = clamped;
                } else {
                    out_dy = (solid.bottom() - mid.top()).clamp(out_dy, 0.0);
                }
            }
        }

        (out_dx, out_dy, landed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn overlap_area(a: Aabb, b: Aabb) -> f32 {
        if !a.overlaps(b) {
            return 0.0;
        }
        let ox = a.right().min(b.right()) - a.left().max(b.left());
        let oy = a.bottom().min(b.bottom()) - a.top().max(b.top());
        ox.max(0.0) * oy.max(0.0)
    }

    proptest! {
        #[test]
        fn resolve_reduces_overlap_area(
            x in -50.0f32..50.0,
            y in -50.0f32..50.0,
            w in 1.0f32..40.0,
            h in 1.0f32..40.0,
            sx in -50.0f32..50.0,
            sy in -50.0f32..50.0,
            sw in 1.0f32..80.0,
            sh in 1.0f32..80.0,
        ) {
            let a = Aabb::new(x, y, w, h);
            let solid = Aabb::new(sx, sy, sw, sh);
            prop_assume!(a.overlaps(solid));
            let before = overlap_area(a, solid);
            let (dx, dy, _) = a.resolve_against(solid);
            let fixed = Aabb::new(a.x + dx, a.y + dy, a.w, a.h);
            let after = overlap_area(fixed, solid);
            prop_assert!(after <= before + 1.0e-3);
            prop_assert!(dx != 0.0 || dy != 0.0 || before < 1.0e-3);
        }
    }

    #[test]
    fn overlap_basic() {
        let a = Aabb::new(0.0, 0.0, 10.0, 10.0);
        let b = Aabb::new(5.0, 5.0, 10.0, 10.0);
        assert!(a.overlaps(b));
        let c = Aabb::new(20.0, 20.0, 5.0, 5.0);
        assert!(!a.overlaps(c));
    }

    #[test]
    fn resolve_lands_on_top() {
        let player = Aabb::new(0.0, 8.0, 10.0, 10.0);
        let ground = Aabb::new(0.0, 15.0, 40.0, 10.0);
        assert!(player.overlaps(ground));
        let (dx, dy, landed) = player.resolve_against(ground);
        assert_eq!(dx, 0.0);
        assert!(dy < 0.0);
        assert!(landed);
        let fixed = Aabb::new(player.x + dx, player.y + dy, player.w, player.h);
        assert!(!fixed.overlaps(ground) || (fixed.bottom() - ground.top()).abs() < 0.01);
    }

    #[test]
    fn sweep_prevents_vertical_tunnel() {
        let player = Aabb::new(0.0, 0.0, 10.0, 10.0);
        let ground = Aabb::new(0.0, 20.0, 40.0, 10.0);
        // Would jump from above ground to below in one step without sweep.
        let (_dx, dy, landed) = player.sweep_move(0.0, 40.0, ground);
        assert!(landed);
        assert!((player.bottom() + dy - ground.top()).abs() < 0.05);
    }
}
