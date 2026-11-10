#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum PlayerState {
    Running = 0,
    Jumping = 1,
    Falling = 2,
    Dashing = 3,
    Rolling = 4,
    Dead = 5,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::Running
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HazardKind {
    /// Kills on overlap unless dashing.
    SolidKill,
    /// Kills unless rolling (crouched).
    LowBar,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum PowerupKind {
    Magnet = 0,
    SlowFall = 1,
    Ghost = 2,
    SuperStrength = 3,
}

impl PowerupKind {
    pub fn ability(self) -> crate::ability::AbilityId {
        match self {
            PowerupKind::Magnet => crate::ability::AbilityId::Magnet,
            PowerupKind::SlowFall => crate::ability::AbilityId::SlowFall,
            PowerupKind::Ghost => crate::ability::AbilityId::Ghost,
            PowerupKind::SuperStrength => crate::ability::AbilityId::SuperStrength,
        }
    }

    pub fn tint(self) -> (f32, f32, f32) {
        match self {
            PowerupKind::Magnet => (1.0, 0.85, 0.15),
            PowerupKind::SlowFall => (0.35, 0.9, 1.0),
            PowerupKind::Ghost => (0.75, 0.45, 1.0),
            PowerupKind::SuperStrength => (1.0, 0.25, 0.15),
        }
    }

    /// Pickup hitbox size (same for fairness); silhouette differs in extract.
    pub fn pickup_size(self) -> (f32, f32) {
        (36.0, 36.0)
    }

    /// Extra quads drawn on top of the pedestal so each power-up is readable
    /// in playtests: (ox, oy, w, h, r, g, b) relative to entity transform.
    pub fn glyph_parts(self) -> &'static [(f32, f32, f32, f32, f32, f32, f32)] {
        match self {
            // Magnet = yellow horseshoe / U
            PowerupKind::Magnet => &[
                (4.0, 6.0, 8.0, 22.0, 1.0, 0.9, 0.1),
                (24.0, 6.0, 8.0, 22.0, 1.0, 0.9, 0.1),
                (4.0, 24.0, 28.0, 8.0, 1.0, 0.85, 0.05),
            ],
            // Slow Fall = cyan parachute (wide canopy + cord + payload)
            PowerupKind::SlowFall => &[
                (2.0, 4.0, 32.0, 10.0, 0.25, 0.95, 1.0),
                (10.0, 14.0, 16.0, 8.0, 0.4, 0.85, 1.0),
                (16.0, 22.0, 4.0, 10.0, 0.9, 0.95, 1.0),
                (12.0, 30.0, 12.0, 6.0, 0.2, 0.75, 0.95),
            ],
            // Ghost = purple body + two bright eyes
            PowerupKind::Ghost => &[
                (6.0, 4.0, 24.0, 26.0, 0.7, 0.4, 1.0),
                (8.0, 28.0, 6.0, 6.0, 0.65, 0.35, 0.95),
                (22.0, 28.0, 6.0, 6.0, 0.65, 0.35, 0.95),
                (10.0, 10.0, 6.0, 6.0, 1.0, 1.0, 1.0),
                (20.0, 10.0, 6.0, 6.0, 1.0, 1.0, 1.0),
            ],
            // Super Strength = red fist / heavy cross
            PowerupKind::SuperStrength => &[
                (14.0, 2.0, 8.0, 32.0, 1.0, 0.2, 0.1),
                (2.0, 14.0, 32.0, 8.0, 1.0, 0.25, 0.1),
                (12.0, 12.0, 12.0, 12.0, 1.0, 0.7, 0.2),
            ],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Destructible {
    pub health: f32,
    pub reward_coins: u32,
    pub material: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Collider {
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Sprite {
    pub w: f32,
    pub h: f32,
    pub u0: f32,
    pub v0: f32,
    pub u1: f32,
    pub v1: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub texture_id: u32,
}

impl Sprite {
    pub fn tinted(w: f32, h: f32, r: f32, g: f32, b: f32, texture_id: u32) -> Self {
        Self {
            w,
            h,
            u0: 0.0,
            v0: 0.0,
            u1: 1.0,
            v1: 1.0,
            r,
            g,
            b,
            a: 1.0,
            texture_id,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Player {
    pub grounded: bool,
    pub jump_buffer: f32,
    pub state: PlayerState,
    pub state_timer: f32,
    pub dash_cooldown: f32,
    pub death_timer: f32,
    pub base_h: f32,
    pub score_coins: u32,
    pub max_x: f32,
    pub finished: bool,
    /// Latches while roll key is held after a roll starts — blocks re-roll until release.
    pub roll_gate: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            grounded: false,
            jump_buffer: 0.0,
            state: PlayerState::Running,
            state_timer: 0.0,
            dash_cooldown: 0.0,
            death_timer: 0.0,
            base_h: crate::config::PLAYER_H,
            score_coins: 0,
            max_x: 0.0,
            finished: false,
            roll_gate: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Platform;

#[derive(Clone, Copy, Debug)]
pub struct Hazard {
    pub kind: HazardKind,
    pub category: crate::collision_cat::CollisionCategory,
}

#[derive(Clone, Copy, Debug)]
pub struct Coin {
    pub value: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct Powerup {
    pub kind: PowerupKind,
}
