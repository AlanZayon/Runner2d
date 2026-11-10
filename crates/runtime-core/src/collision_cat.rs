//! Collision categories for Ghost and solid obstacles.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum CollisionCategory {
    #[default]
    Normal = 0,
    /// Pass-through when Ghost is active; solid/lethal otherwise.
    GhostPassable = 1,
    /// Solid/lethal without Ghost; pass-through when Ghost is active.
    GhostRequired = 2,
    /// Always collides / lethal regardless of Ghost.
    AlwaysSolid = 3,
}

/// Whether a solid body should physically resolve against the player.
pub fn blocks_player(category: CollisionCategory, ghost_active: bool) -> bool {
    match category {
        CollisionCategory::AlwaysSolid | CollisionCategory::Normal => true,
        CollisionCategory::GhostPassable | CollisionCategory::GhostRequired => !ghost_active,
    }
}

/// Whether a hazard overlap should apply lethal rules.
pub fn hazard_applies(category: CollisionCategory, ghost_active: bool) -> bool {
    match category {
        CollisionCategory::AlwaysSolid | CollisionCategory::Normal => true,
        CollisionCategory::GhostPassable | CollisionCategory::GhostRequired => !ghost_active,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ghost_passable_only_when_ghost() {
        assert!(blocks_player(CollisionCategory::GhostPassable, false));
        assert!(!blocks_player(CollisionCategory::GhostPassable, true));
        assert!(hazard_applies(CollisionCategory::GhostPassable, false));
        assert!(!hazard_applies(CollisionCategory::GhostPassable, true));
    }

    #[test]
    fn always_solid_ignores_ghost() {
        assert!(blocks_player(CollisionCategory::AlwaysSolid, true));
        assert!(hazard_applies(CollisionCategory::AlwaysSolid, true));
    }
}
