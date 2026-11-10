//! Vertical Shift runtime-core — Phase 12 spec compliance (ABI 1.7).

pub mod aabb;
pub mod ability;
pub mod animation;
pub mod collision_cat;
pub mod components;
pub mod config;
pub mod director;
pub mod effects;
pub mod entity;
pub mod events;
pub mod falling;
pub mod generate;
pub mod headless;
pub mod modifiers;
pub mod profile;
pub mod qte;
pub mod rng;
pub mod segment;
pub mod streamer;
pub mod templates;
pub mod validate;
pub mod world;

pub use aabb::Aabb;
pub use ability::{AbilityId, AbilityManager};
pub use animation::{AnimClip, AnimController};
pub use collision_cat::CollisionCategory;
pub use components::*;
pub use director::{difficulty_tier, generate_course, DifficultyTier};
pub use entity::{DenseSlot, EntityAllocator, EntityId};
pub use components::{HazardKind, PlayerState, PowerupKind};
pub use events::{EventQueue, GameEvent, GameEventKind};
pub use falling::RunMode;
pub use profile::TickProfile;
pub use qte::QteRating;
pub use segment::{GeneratedCourse, SegmentDefinition, SegmentIntent};
pub use world::{CameraState, ExtractSprite, InputSnapshot, QualityProfile, RunState, World};

/// Semantic version string (Rust side).
pub const RUNTIME_VERSION: &str = "0.12.0-phase12";

/// NUL-terminated version for the C ABI.
pub const RUNTIME_VERSION_C: &[u8] = b"0.12.0-phase12\0";

/// ABI major version. Bump on incompatible FFI changes.
pub const ABI_VERSION_MAJOR: u32 = 1;

/// ABI minor version. Bump on compatible FFI additions.
pub const ABI_VERSION_MINOR: u32 = 7;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_non_empty() {
        assert!(!RUNTIME_VERSION.is_empty());
        assert_eq!(
            RUNTIME_VERSION.as_bytes(),
            &RUNTIME_VERSION_C[..RUNTIME_VERSION_C.len() - 1]
        );
        assert_eq!(*RUNTIME_VERSION_C.last().unwrap(), 0);
        assert!(ABI_VERSION_MAJOR >= 1);
        assert_eq!(ABI_VERSION_MINOR, 7);
    }
}
