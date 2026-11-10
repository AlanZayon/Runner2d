//! Parametric geometry / obstacle / coin placement per segment intent.

use crate::collision_cat::CollisionCategory;
use crate::components::{HazardKind, PowerupKind};
use crate::config::{
    FALLING_EXIT_LIP_WIDTH, FALLING_LIP_WIDTH, FALLING_PIT_WIDTH_MAX, FALLING_PIT_WIDTH_MIN,
    FALLING_SHAFT_DEPTH_MAX, FALLING_SHAFT_DEPTH_MIN, GROUND_PLATFORM_H, GROUND_Y, LOW_BAR_HEIGHT,
    MAX_DASH_THROUGH_W, MAX_SAFE_GAP, PLAYER_H,
};
use crate::rng::XorShift64;
use crate::segment::{
    CoinSpec, DestructibleSpec, HazardSpec, PlatformSpec, PowerupSpec, SegmentDefinition,
    SegmentIntent, ShaftSpec,
};

fn ground_tint() -> (f32, f32, f32) {
    (0.25, 0.45, 0.40)
}
fn shelf_tint() -> (f32, f32, f32) {
    (0.35, 0.40, 0.55)
}
fn solid_tint() -> (f32, f32, f32) {
    (0.85, 0.25, 0.30)
}
fn lowbar_tint() -> (f32, f32, f32) {
    (0.75, 0.55, 0.15)
}

fn plat(x: f32, y: f32, w: f32, h: f32, tint: (f32, f32, f32)) -> PlatformSpec {
    PlatformSpec { x, y, w, h, tint }
}

fn haz(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    kind: HazardKind,
    tint: (f32, f32, f32),
) -> HazardSpec {
    HazardSpec {
        x,
        y,
        w,
        h,
        kind,
        tint,
        category: CollisionCategory::Normal,
        destructible: None,
    }
}

fn maybe_sparse_powerups(
    rng: &mut XorShift64,
    world_x0: f32,
    width: f32,
    gy: f32,
) -> Vec<PowerupSpec> {
    // ~18% chance of one pickup — never gates progression.
    if rng.gen_range_u32(0, 5) != 0 {
        return Vec::new();
    }
    let kinds = [
        PowerupKind::Magnet,
        PowerupKind::SlowFall,
        PowerupKind::Ghost,
        PowerupKind::SuperStrength,
    ];
    let kind = kinds[rng.choose_index(kinds.len())];
    vec![PowerupSpec {
        kind,
        x: world_x0 + rng.gen_range_f32(40.0, (width - 40.0).max(50.0)),
        y: gy - rng.gen_range_f32(70.0, 110.0),
    }]
}

fn maybe_ghost_or_destructible(
    rng: &mut XorShift64,
    world_x0: f32,
    width: f32,
    gy: f32,
) -> Option<HazardSpec> {
    // Optional side obstacle; ground route remains clear.
    match rng.gen_range_u32(0, 7) {
        0 => {
            let w = rng.gen_range_f32(28.0, 40.0);
            let x = world_x0 + rng.gen_range_f32(50.0, (width - w - 40.0).max(60.0));
            Some(HazardSpec {
                x,
                y: gy - 70.0,
                w,
                h: 70.0,
                kind: HazardKind::SolidKill,
                tint: (0.55, 0.45, 0.85),
                category: CollisionCategory::GhostPassable,
                destructible: None,
            })
        }
        1 => {
            let w = rng.gen_range_f32(32.0, 44.0);
            let x = world_x0 + rng.gen_range_f32(50.0, (width - w - 40.0).max(60.0));
            Some(HazardSpec {
                x,
                y: gy - 64.0,
                w,
                h: 64.0,
                kind: HazardKind::SolidKill,
                tint: (0.95, 0.4, 0.2),
                category: CollisionCategory::AlwaysSolid,
                destructible: Some(DestructibleSpec {
                    health: 30.0,
                    reward_coins: 2,
                    material: 1,
                }),
            })
        }
        _ => None,
    }
}

fn coin_arc(start_x: f32, span: f32, base_y: f32, arch: f32, count: usize) -> Vec<CoinSpec> {
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let t = if count <= 1 {
            0.5
        } else {
            i as f32 / (count - 1) as f32
        };
        out.push(CoinSpec {
            x: start_x + t * span,
            y: base_y - (1.0 - (t - 0.5).abs() * 2.0) * arch,
        });
    }
    out
}

fn pack(
    intent: SegmentIntent,
    family_id: u32,
    world_x0: f32,
    width: f32,
    platforms: Vec<PlatformSpec>,
    hazards: Vec<HazardSpec>,
    coins: Vec<CoinSpec>,
    powerups: Vec<PowerupSpec>,
) -> SegmentDefinition {
    pack_shaft(
        intent,
        family_id,
        world_x0,
        width,
        platforms,
        hazards,
        coins,
        powerups,
        None,
    )
}

fn pack_shaft(
    intent: SegmentIntent,
    family_id: u32,
    world_x0: f32,
    width: f32,
    platforms: Vec<PlatformSpec>,
    hazards: Vec<HazardSpec>,
    coins: Vec<CoinSpec>,
    powerups: Vec<PowerupSpec>,
    shaft: Option<ShaftSpec>,
) -> SegmentDefinition {
    SegmentDefinition {
        intent,
        template_id: family_id,
        world_x0,
        width,
        platforms,
        hazards,
        coins,
        powerups,
        shaft,
    }
}

/// Guaranteed flat recovery (explicit fallback only).
pub fn safe_recovery(world_x0: f32, width: f32) -> SegmentDefinition {
    let gy = GROUND_Y;
    let w = width.max(160.0);
    let mut coins = Vec::new();
    let n = ((w / 60.0) as usize).clamp(2, 5);
    for i in 0..n {
        coins.push(CoinSpec {
            x: world_x0 + 30.0 + i as f32 * (w - 60.0) / n.max(1) as f32,
            y: gy - 70.0,
        });
    }
    pack(
        SegmentIntent::Recovery,
        1,
        world_x0,
        w,
        vec![plat(world_x0, gy, w, GROUND_PLATFORM_H, ground_tint())],
        vec![],
        coins,
        Vec::new(),
    )
}

fn gen_recovery(rng: &mut XorShift64, world_x0: f32) -> SegmentDefinition {
    let width = rng.gen_range_f32(180.0, 260.0);
    let gy = GROUND_Y;
    let mut platforms = vec![plat(world_x0, gy, width, GROUND_PLATFORM_H, ground_tint())];
    let mut coins = Vec::new();
    let coin_n = rng.gen_range_u32(2, 4) as usize;
    for i in 0..coin_n {
        coins.push(CoinSpec {
            x: world_x0 + 35.0 + i as f32 * ((width - 70.0) / coin_n.max(1) as f32),
            y: gy - rng.gen_range_f32(60.0, 85.0),
        });
    }
    if rng.gen_range_u32(0, 1) == 1 {
        let sx = rng.gen_range_f32(40.0, (width - 80.0).max(50.0));
        platforms.push(plat(
            world_x0 + sx,
            gy - rng.gen_range_f32(70.0, 95.0),
            rng.gen_range_f32(50.0, 80.0),
            18.0,
            shelf_tint(),
        ));
    }
    let mut hazards = Vec::new();
    if let Some(h) = maybe_ghost_or_destructible(rng, world_x0, width, gy) {
        hazards.push(h);
    }
    let powerups = maybe_sparse_powerups(rng, world_x0, width, gy);
    pack(
        SegmentIntent::Recovery,
        100,
        world_x0,
        width,
        platforms,
        hazards,
        coins,
        powerups,
    )
}

fn gen_teach_jump(rng: &mut XorShift64, world_x0: f32) -> SegmentDefinition {
    let gap = rng.gen_range_f32(90.0, 115.0).min(MAX_SAFE_GAP - 15.0);
    let left_w = rng.gen_range_f32(85.0, 120.0);
    let right_w = rng.gen_range_f32(100.0, 140.0);
    let width = left_w + gap + right_w;
    let gy = GROUND_Y;
    let right_x = left_w + gap;
    let mut platforms = vec![
        plat(world_x0, gy, left_w, GROUND_PLATFORM_H, ground_tint()),
        plat(
            world_x0 + right_x,
            gy,
            right_w,
            GROUND_PLATFORM_H,
            ground_tint(),
        ),
    ];
    if rng.gen_range_u32(0, 1) == 1 {
        platforms.push(plat(
            world_x0 + left_w + gap * rng.gen_range_f32(0.25, 0.45),
            gy - rng.gen_range_f32(70.0, 95.0),
            rng.gen_range_f32(55.0, 85.0),
            18.0,
            shelf_tint(),
        ));
    }
    let coins = coin_arc(
        world_x0 + left_w - 8.0,
        gap + 16.0,
        gy - rng.gen_range_f32(85.0, 110.0),
        rng.gen_range_f32(30.0, 50.0),
        rng.gen_range_u32(4, 6) as usize,
    );
    pack(
        SegmentIntent::TeachJump,
        110,
        world_x0,
        width,
        platforms,
        vec![],
        coins,
        Vec::new(),
    )
}

fn gen_teach_dash(rng: &mut XorShift64, world_x0: f32) -> SegmentDefinition {
    let width = rng.gen_range_f32(200.0, 260.0);
    let gy = GROUND_Y;
    let bw = rng.gen_range_f32(28.0, 40.0).min(MAX_DASH_THROUGH_W);
    let bx = rng.gen_range_f32(70.0, (width - bw - 50.0).max(80.0));
    let platforms = vec![plat(world_x0, gy, width, GROUND_PLATFORM_H, ground_tint())];
    let hazards = vec![haz(
        world_x0 + bx,
        gy - 70.0,
        bw,
        70.0,
        HazardKind::SolidKill,
        solid_tint(),
    )];
    let coins = coin_arc(
        world_x0 + bx - 25.0,
        bw + 50.0,
        gy - rng.gen_range_f32(100.0, 125.0),
        35.0,
        4,
    );
    pack(
        SegmentIntent::TeachDash,
        120,
        world_x0,
        width,
        platforms,
        hazards,
        coins,
        Vec::new(),
    )
}

fn gen_teach_roll(rng: &mut XorShift64, world_x0: f32) -> SegmentDefinition {
    let width = rng.gen_range_f32(200.0, 260.0);
    let gy = GROUND_Y;
    let bar_w = rng.gen_range_f32(60.0, 90.0);
    let bx = rng.gen_range_f32(60.0, (width - bar_w - 40.0).max(70.0));
    let platforms = vec![plat(world_x0, gy, width, GROUND_PLATFORM_H, ground_tint())];
    let hazards = vec![haz(
        world_x0 + bx,
        gy - LOW_BAR_HEIGHT,
        bar_w,
        LOW_BAR_HEIGHT,
        HazardKind::LowBar,
        lowbar_tint(),
    )];
    let mut coins = Vec::new();
    let n = rng.gen_range_u32(2, 4) as usize;
    for i in 0..n {
        coins.push(CoinSpec {
            x: world_x0 + bx + 8.0 + i as f32 * (bar_w / n.max(1) as f32),
            y: gy - PLAYER_H * 0.35,
        });
    }
    pack(
        SegmentIntent::TeachRoll,
        130,
        world_x0,
        width,
        platforms,
        hazards,
        coins,
        Vec::new(),
    )
}

fn gen_jump_dash(rng: &mut XorShift64, world_x0: f32) -> SegmentDefinition {
    let gap = rng.gen_range_f32(90.0, 110.0).min(MAX_SAFE_GAP - 20.0);
    let left_w = rng.gen_range_f32(85.0, 110.0);
    let mid_w = rng.gen_range_f32(110.0, 150.0);
    let tail_w = rng.gen_range_f32(70.0, 100.0);
    let width = left_w + gap + mid_w + tail_w;
    let gy = GROUND_Y;
    let mid_x = left_w + gap;
    let rest_x = mid_x + mid_w;
    let bw = rng.gen_range_f32(28.0, 38.0);
    let hazard_off = rng.gen_range_f32(25.0, (mid_w - bw - 20.0).max(30.0));
    let platforms = vec![
        plat(world_x0, gy, left_w, GROUND_PLATFORM_H, ground_tint()),
        plat(
            world_x0 + mid_x,
            gy,
            mid_w,
            GROUND_PLATFORM_H,
            ground_tint(),
        ),
        plat(
            world_x0 + rest_x,
            gy,
            tail_w,
            GROUND_PLATFORM_H,
            ground_tint(),
        ),
    ];
    let hazards = vec![haz(
        world_x0 + mid_x + hazard_off,
        gy - 70.0,
        bw,
        70.0,
        HazardKind::SolidKill,
        solid_tint(),
    )];
    let coins = coin_arc(
        world_x0 + left_w - 5.0,
        gap + 10.0,
        gy - rng.gen_range_f32(90.0, 110.0),
        40.0,
        4,
    );
    pack(
        SegmentIntent::JumpDash,
        140,
        world_x0,
        width,
        platforms,
        hazards,
        coins,
        Vec::new(),
    )
}

fn gen_jump_roll(rng: &mut XorShift64, world_x0: f32) -> SegmentDefinition {
    let gap = rng.gen_range_f32(90.0, 110.0).min(MAX_SAFE_GAP - 20.0);
    let left_w = rng.gen_range_f32(85.0, 110.0);
    let right_w = rng.gen_range_f32(160.0, 210.0);
    let width = left_w + gap + right_w;
    let gy = GROUND_Y;
    let mid_x = left_w + gap;
    let bar_w = rng.gen_range_f32(60.0, 85.0);
    // Leave runway after landing before the low bar.
    let bar_off = rng.gen_range_f32(55.0, (right_w - bar_w - 30.0).max(60.0));
    let platforms = vec![
        plat(world_x0, gy, left_w, GROUND_PLATFORM_H, ground_tint()),
        plat(
            world_x0 + mid_x,
            gy,
            right_w,
            GROUND_PLATFORM_H,
            ground_tint(),
        ),
    ];
    let hazards = vec![haz(
        world_x0 + mid_x + bar_off,
        gy - LOW_BAR_HEIGHT,
        bar_w,
        LOW_BAR_HEIGHT,
        HazardKind::LowBar,
        lowbar_tint(),
    )];
    let coins = coin_arc(
        world_x0 + left_w - 5.0,
        gap + 10.0,
        gy - rng.gen_range_f32(90.0, 110.0),
        40.0,
        4,
    );
    pack(
        SegmentIntent::JumpRoll,
        150,
        world_x0,
        width,
        platforms,
        hazards,
        coins,
        Vec::new(),
    )
}

fn gen_risk_reward(rng: &mut XorShift64, world_x0: f32) -> SegmentDefinition {
    let gap = rng.gen_range_f32(105.0, 125.0).min(MAX_SAFE_GAP - 10.0);
    let left_w = rng.gen_range_f32(80.0, 110.0);
    let right_w = rng.gen_range_f32(100.0, 160.0);
    let width = left_w + gap + right_w;
    let gy = GROUND_Y;
    let right_x = left_w + gap;
    let platforms = vec![
        plat(world_x0, gy, left_w, GROUND_PLATFORM_H, ground_tint()),
        plat(
            world_x0 + right_x,
            gy,
            right_w,
            GROUND_PLATFORM_H,
            ground_tint(),
        ),
    ];
    let coins = coin_arc(
        world_x0 + left_w - 5.0,
        gap + 10.0,
        gy - rng.gen_range_f32(110.0, 140.0),
        rng.gen_range_f32(45.0, 60.0),
        rng.gen_range_u32(5, 7) as usize,
    );
    let mut hazards = Vec::new();
    if let Some(h) = maybe_ghost_or_destructible(rng, world_x0, width, gy) {
        // Place on the right pad so the gap route stays valid without the ability.
        let mut h = h;
        h.x = world_x0 + right_x + rng.gen_range_f32(20.0, (right_w - h.w - 10.0).max(25.0));
        hazards.push(h);
    }
    let powerups = maybe_sparse_powerups(rng, world_x0, width, gy);
    pack(
        SegmentIntent::RiskReward,
        160,
        world_x0,
        width,
        platforms,
        hazards,
        coins,
        powerups,
    )
}

fn gen_ghost_challenge(rng: &mut XorShift64, world_x0: f32) -> SegmentDefinition {
    // Safe high road + low GhostPassable corridor blocked by AlwaysSolid for non-Ghost.
    let left_w = rng.gen_range_f32(90.0, 120.0);
    let mid_w = rng.gen_range_f32(100.0, 140.0);
    let right_w = rng.gen_range_f32(90.0, 130.0);
    let gap = rng.gen_range_f32(70.0, 95.0);
    let width = left_w + gap + mid_w + 40.0 + right_w;
    let gy = GROUND_Y;
    let mid_x = world_x0 + left_w + gap;
    let right_x = mid_x + mid_w + 40.0;
    let shelf_y = gy - rng.gen_range_f32(70.0, 95.0);
    let platforms = vec![
        plat(world_x0, gy, left_w, GROUND_PLATFORM_H, ground_tint()),
        // Safe alternate route above the trap.
        plat(mid_x - 10.0, shelf_y, mid_w + 60.0, 18.0, shelf_tint()),
        plat(mid_x, gy, mid_w, GROUND_PLATFORM_H, ground_tint()),
        plat(right_x, gy, right_w, GROUND_PLATFORM_H, ground_tint()),
    ];
    let trap_w = rng.gen_range_f32(36.0, 48.0);
    let hazards = vec![
        HazardSpec {
            x: mid_x + mid_w * 0.35,
            y: gy - 72.0,
            w: trap_w,
            h: 72.0,
            kind: HazardKind::SolidKill,
            tint: (0.95, 0.2, 0.35),
            category: CollisionCategory::AlwaysSolid,
            destructible: None,
        },
        HazardSpec {
            x: mid_x + 8.0,
            y: gy - 50.0,
            w: mid_w - 16.0,
            h: 50.0,
            kind: HazardKind::SolidKill,
            tint: (0.55, 0.45, 0.85),
            category: CollisionCategory::GhostPassable,
            destructible: None,
        },
    ];
    let coins = coin_arc(
        mid_x,
        mid_w,
        gy - 40.0,
        20.0,
        rng.gen_range_u32(4, 6) as usize,
    );
    let powerups = vec![PowerupSpec {
        kind: PowerupKind::Ghost,
        x: world_x0 + left_w * 0.5,
        y: gy - 90.0,
    }];
    pack(
        SegmentIntent::GhostChallenge,
        190,
        world_x0,
        width,
        platforms,
        hazards,
        coins,
        powerups,
    )
}

fn gen_transition(rng: &mut XorShift64, world_x0: f32) -> SegmentDefinition {
    let width = rng.gen_range_f32(200.0, 280.0);
    let gy = GROUND_Y;
    let platforms = vec![plat(world_x0, gy, width, GROUND_PLATFORM_H, ground_tint())];
    let n = rng.gen_range_u32(3, 6) as usize;
    let mut coins = Vec::new();
    for i in 0..n {
        coins.push(CoinSpec {
            x: world_x0 + 30.0 + i as f32 * ((width - 60.0) / n.max(1) as f32),
            y: gy - rng.gen_range_f32(70.0, 95.0),
        });
    }
    pack(
        SegmentIntent::Transition,
        170,
        world_x0,
        width,
        platforms,
        vec![],
        coins,
        Vec::new(),
    )
}

fn gen_falling_shaft(rng: &mut XorShift64, world_x0: f32) -> SegmentDefinition {
    let gy = GROUND_Y;
    let lip = FALLING_LIP_WIDTH;
    let exit_lip = FALLING_EXIT_LIP_WIDTH;
    let pit_w = rng.gen_range_f32(FALLING_PIT_WIDTH_MIN, FALLING_PIT_WIDTH_MAX);
    let depth = rng.gen_range_f32(FALLING_SHAFT_DEPTH_MIN, FALLING_SHAFT_DEPTH_MAX);
    let floor_y = gy + depth;
    let width = lip + pit_w + exit_lip;
    let pit_x0 = world_x0 + lip;
    let pit_x1 = pit_x0 + pit_w;
    let exit_x = pit_x1;
    let wall_tint = (0.2, 0.22, 0.28);
    let floor_tint = (0.3, 0.5, 0.35);

    let wall_h = depth + GROUND_PLATFORM_H;
    let platforms = vec![
        // Entry lip at run height.
        plat(world_x0, gy, lip, GROUND_PLATFORM_H, ground_tint()),
        // Landing pad at bottom of pit.
        plat(pit_x0, floor_y, pit_w, GROUND_PLATFORM_H, floor_tint),
        // Exit lip back at run height (player teleports here on FallingExit).
        plat(exit_x, gy, exit_lip, GROUND_PLATFORM_H, ground_tint()),
        // Side walls hang into the pit (y below GROUND_Y so they are not "ground spans").
        plat(
            pit_x0 - 14.0,
            gy + GROUND_PLATFORM_H,
            14.0,
            wall_h,
            wall_tint,
        ),
        plat(pit_x1, gy + GROUND_PLATFORM_H, 14.0, wall_h, wall_tint),
    ];

    let mut hazards = Vec::new();

    // Mid-air obstacles (dodge with strafe / dash / ghost).
    let bands = rng.gen_range_u32(2, 4);
    for i in 0..bands {
        let t = (i as f32 + 1.0) / (bands as f32 + 1.0);
        let y = gy + depth * t - 40.0;
        let w = rng.gen_range_f32(28.0, 44.0);
        let x = pit_x0 + rng.gen_range_f32(20.0, (pit_w - w - 20.0).max(30.0));
        let ghost = rng.gen_range_u32(0, 3) == 0;
        hazards.push(HazardSpec {
            x,
            y,
            w,
            h: 36.0,
            kind: HazardKind::SolidKill,
            tint: if ghost {
                (0.55, 0.45, 0.85)
            } else {
                solid_tint()
            },
            category: if ghost {
                CollisionCategory::GhostPassable
            } else {
                CollisionCategory::Normal
            },
            destructible: None,
        });
    }

    // Vertical coin trail through the pit.
    let mut coins = Vec::new();
    let coin_n = rng.gen_range_u32(5, 8) as usize;
    for i in 0..coin_n {
        let t = (i as f32 + 0.5) / coin_n as f32;
        coins.push(CoinSpec {
            x: pit_x0 + pit_w * 0.5 + rng.gen_range_f32(-30.0, 30.0),
            y: gy + depth * t - 20.0,
        });
    }

    let mut powerups = Vec::new();
    if rng.gen_range_u32(0, 2) == 0 {
        let kinds = [
            PowerupKind::SlowFall,
            PowerupKind::Ghost,
            PowerupKind::Magnet,
        ];
        powerups.push(PowerupSpec {
            kind: kinds[rng.choose_index(kinds.len())],
            x: pit_x0 + pit_w * 0.5,
            y: gy + depth * 0.35,
        });
    }

    let shaft = ShaftSpec {
        pit_x0,
        pit_x1,
        floor_y,
        enter_y: gy,
        exit_x,
        exit_y: gy,
    };

    pack_shaft(
        SegmentIntent::FallingShaft,
        180,
        world_x0,
        width,
        platforms,
        hazards,
        coins,
        powerups,
        Some(shaft),
    )
}

/// Generate one segment for `intent` at `world_x0` using RNG parameters.
pub fn generate_segment(
    rng: &mut XorShift64,
    intent: SegmentIntent,
    world_x0: f32,
) -> SegmentDefinition {
    match intent {
        SegmentIntent::Recovery => gen_recovery(rng, world_x0),
        SegmentIntent::TeachJump => gen_teach_jump(rng, world_x0),
        SegmentIntent::TeachDash => gen_teach_dash(rng, world_x0),
        SegmentIntent::TeachRoll => gen_teach_roll(rng, world_x0),
        SegmentIntent::JumpDash => gen_jump_dash(rng, world_x0),
        SegmentIntent::JumpRoll => gen_jump_roll(rng, world_x0),
        SegmentIntent::RiskReward => gen_risk_reward(rng, world_x0),
        SegmentIntent::Transition => gen_transition(rng, world_x0),
        SegmentIntent::FallingShaft => gen_falling_shaft(rng, world_x0),
        SegmentIntent::GhostChallenge => gen_ghost_challenge(rng, world_x0),
    }
}

/// Compact local fingerprint for variety tests (geometry + hazards).
pub fn segment_local_fingerprint(seg: &SegmentDefinition) -> u64 {
    let mut h = seg.intent as u32 as u64;
    h = h
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(seg.width.to_bits() as u64);
    h = h
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(seg.platforms.len() as u64);
    h = h
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(seg.hazards.len() as u64);
    for p in &seg.platforms {
        h = h
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            .wrapping_add(p.x.to_bits() as u64);
        h = h
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            .wrapping_add(p.w.to_bits() as u64);
        h = h
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            .wrapping_add(p.y.to_bits() as u64);
    }
    for hz in &seg.hazards {
        let k = match hz.kind {
            HazardKind::SolidKill => 1u64,
            HazardKind::LowBar => 2u64,
        };
        h = h.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(k);
        h = h
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            .wrapping_add(hz.x.to_bits() as u64);
        h = h
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            .wrapping_add(hz.w.to_bits() as u64);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::validate_segment;

    #[test]
    fn parametric_intents_validate() {
        let mut rng = XorShift64::from_seed(12345);
        for intent in [
            SegmentIntent::Recovery,
            SegmentIntent::TeachJump,
            SegmentIntent::TeachDash,
            SegmentIntent::TeachRoll,
            SegmentIntent::JumpDash,
            SegmentIntent::JumpRoll,
            SegmentIntent::RiskReward,
            SegmentIntent::Transition,
            SegmentIntent::FallingShaft,
            SegmentIntent::GhostChallenge,
        ] {
            for _ in 0..8 {
                let seg = generate_segment(&mut rng, intent, 50.0);
                assert!(
                    validate_segment(&seg).is_ok(),
                    "{intent:?} invalid: {:?}",
                    validate_segment(&seg)
                );
            }
        }
    }

    #[test]
    fn same_seed_same_segment_params() {
        let mut a = XorShift64::from_seed(99);
        let mut b = XorShift64::from_seed(99);
        let sa = generate_segment(&mut a, SegmentIntent::TeachJump, 0.0);
        let sb = generate_segment(&mut b, SegmentIntent::TeachJump, 0.0);
        assert_eq!(segment_local_fingerprint(&sa), segment_local_fingerprint(&sb));
    }
}
