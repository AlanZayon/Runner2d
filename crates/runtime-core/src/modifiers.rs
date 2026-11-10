//! Temporary movement / collision modifiers derived from active abilities.

#[derive(Clone, Copy, Debug)]
pub struct MovementModifiers {
    pub gravity_scale: f32,
    pub max_fall_scale: f32,
    /// Reserved for future air strafe; auto-run keeps X fixed.
    pub air_control: f32,
}

impl Default for MovementModifiers {
    fn default() -> Self {
        Self {
            gravity_scale: 1.0,
            max_fall_scale: 1.0,
            air_control: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CollisionModifiers {
    pub ghost_active: bool,
    pub strength_active: bool,
    pub invulnerable: bool,
}
