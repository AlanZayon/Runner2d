//! Arcade feel tuning (world units = pixels, Y down, run along +X).

pub const GRAVITY: f32 = 2200.0;
pub const MAX_FALL_SPEED: f32 = 1200.0;
pub const AUTO_RUN_SPEED: f32 = 260.0;
/// Speed gained per second survived (px/s² equivalent).
pub const SPEED_RAMP_PER_SEC: f32 = 6.0;
/// Hard cap so speed does not grow forever.
pub const MAX_RUN_SPEED: f32 = 420.0;
/// Added on top of current run speed during dash.
pub const DASH_BONUS: f32 = 240.0;
/// Deprecated: horizontal strafe removed — auto-run only.
pub const STRAFE_SPEED: f32 = 0.0;
pub const JUMP_VELOCITY: f32 = -750.0;
pub const JUMP_BUFFER_SECONDS: f32 = 0.10;
/// Legacy absolute dash (prefer `run_speed + DASH_BONUS` at runtime).
pub const DASH_SPEED: f32 = 520.0;
/// Timed burst; holding the key does not extend dash.
pub const DASH_DURATION: f32 = 0.18;
pub const DASH_COOLDOWN: f32 = 0.40;
/// Timed crouch; holding the key does not extend or chain-roll.
pub const ROLL_DURATION: f32 = 0.55;
pub const ROLL_HEIGHT_SCALE: f32 = 0.5;
pub const COIN_VALUE: u32 = 10;
pub const DISTANCE_SCORE_DIV: f32 = 10.0;
pub const CAMERA_FOLLOW_LERP: f32 = 10.0;
pub const CAMERA_LOOKAHEAD: f32 = 120.0;
/// Far below FallingShaft floors so the pit is traversable.
pub const DEATH_Y: f32 = 1800.0;
pub const RESTART_DELAY: f32 = 1.0;
pub const PLAYER_SPAWN_X: f32 = 40.0;
pub const PLAYER_SPAWN_Y: f32 = 480.0;
pub const PLAYER_W: f32 = 28.0;
pub const PLAYER_H: f32 = 40.0;
pub const GROUND_Y: f32 = 560.0;
/// Legacy constant; procedural runs use `World.course_end` instead.
pub const COURSE_END: f32 = 1600.0;
/// Player / character sprite atlas (host maps to demo/sprite.png).
pub const DEFAULT_TEXTURE_ID: u32 = 1;
/// Solid white quad for tinted platforms/hazards/FX (host maps to demo/white.png).
pub const SOLID_TEXTURE_ID: u32 = 2;
pub const DASH_SUBSTEPS: u32 = 4;

/// Initial buffered segments at run start (course continues infinitely past this).
pub const SEGMENT_COUNT: usize = 8;
/// Definitions generated ahead (cheap: no entities). Larger than the spawn window
/// so the tip never races the runner; streaming decides what is live.
pub const GENERATE_AHEAD_PX: f32 = 2400.0;
/// Also keep at least this many segment definitions past the player's current index.
pub const GENERATE_AHEAD_SEGMENTS: usize = 5;
/// Spawn/despawn window in segment indices (also expanded by pixel window in streamer).
pub const STREAM_AHEAD: usize = 5;
pub const STREAM_BEHIND: usize = 1;
/// Live entity window (~2–3 viewports). Keep smaller than GENERATE_AHEAD_PX.
pub const STREAM_AHEAD_PX: f32 = 1200.0;
pub const STREAM_BEHIND_PX: f32 = 400.0;
pub const DEFAULT_SEED: u64 = 0x0055_EED1_0000_0005;
pub const GENERATOR_VERSION: u32 = 4;
pub const MAX_COURSE_ATTEMPTS: u32 = 48;
pub const MAX_SEGMENT_REROLLS: u32 = 12;

/// Conservative horizontal jump clearance at AUTO_RUN_SPEED (px).
pub const MAX_SAFE_GAP: f32 = 140.0;
/// SolidKill width that dash can clear during DASH_DURATION.
pub const MAX_DASH_THROUGH_W: f32 = 48.0;
/// LowBar height must leave roll passage under the bar top.
pub const LOW_BAR_HEIGHT: f32 = 28.0;
pub const GROUND_PLATFORM_H: f32 = 40.0;

// --- Phase 6: power-ups / abilities ---
pub const MAGNET_DURATION: f32 = 6.0;
pub const MAGNET_RADIUS: f32 = 140.0;
pub const MAGNET_FORCE: f32 = 520.0;
pub const MAGNET_MAX_SPEED: f32 = 420.0;
pub const MAGNET_DAMPING: f32 = 0.88;

pub const SLOW_FALL_DURATION: f32 = 5.0;
pub const SLOW_FALL_GRAVITY_SCALE: f32 = 0.35;
pub const SLOW_FALL_MAX_FALL_SCALE: f32 = 0.40;

pub const GHOST_DURATION: f32 = 5.0;
pub const GHOST_ALPHA: f32 = 0.55;

pub const STRENGTH_QTE_PROMPTS: u32 = 4;
pub const STRENGTH_QTE_WINDOW: f32 = 0.55;
pub const STRENGTH_DURATION_POOR: f32 = 2.0;
pub const STRENGTH_DURATION_GOOD: f32 = 3.5;
pub const STRENGTH_DURATION_GREAT: f32 = 5.0;
pub const STRENGTH_DURATION_PERFECT: f32 = 7.0;
pub const STRENGTH_DESTROY_DAMAGE: f32 = 999.0;

pub const POWERUP_PICKUP_SIZE: f32 = 36.0;
/// Dark pedestal under the glyph so pickups pop against platforms.
pub const POWERUP_PEDESTAL_RGB: (f32, f32, f32) = (0.08, 0.08, 0.12);
pub const EFFECT_POOL_CAP: usize = 192;
pub const EFFECT_GHOST_TRAIL_INTERVAL: f32 = 0.05;
pub const EFFECT_SLOW_FALL_INTERVAL: f32 = 0.08;

// --- Phase 7: Falling Mode ---
pub const FALLING_STRAFE_SPEED: f32 = 260.0;
pub const FALLING_ENTER_DURATION: f32 = 0.12;
pub const FALLING_EXIT_DURATION: f32 = 0.35;
pub const FALLING_MAX_DURATION: f32 = 8.0;
pub const FALLING_SHAFT_DEPTH_MIN: f32 = 380.0;
pub const FALLING_SHAFT_DEPTH_MAX: f32 = 520.0;
pub const FALLING_PIT_WIDTH_MIN: f32 = 160.0;
pub const FALLING_PIT_WIDTH_MAX: f32 = 220.0;
pub const FALLING_LIP_WIDTH: f32 = 90.0;
pub const FALLING_EXIT_LIP_WIDTH: f32 = 120.0;
pub const CAMERA_FALL_LOOKAHEAD: f32 = 140.0;
pub const CAMERA_FALL_X_LOOKAHEAD: f32 = 40.0;
/// Roughly one FallingShaft every N endless picks (plus anti-adjacent).
pub const FALLING_SHAFT_EVERY_N: u32 = 7;

// --- Phase 10: spatial query window (collision / pickup scans) ---
/// Half-width (px) around the player used to cull platforms/hazards/coins/powerups.
pub const COLLISION_QUERY_PAD_X: f32 = 420.0;

// Deprecated alias kept for any leftover references.
pub const MOVE_SPEED: f32 = STRAFE_SPEED;
