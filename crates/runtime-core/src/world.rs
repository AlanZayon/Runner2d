//! Phase 8 runner world: procedural segments, streaming, abilities, Falling Mode, game events.

use crate::aabb::Aabb;
use crate::ability::{AbilityId, AbilityManager};
use crate::animation::AnimController;
use crate::collision_cat::{self, CollisionCategory};
use crate::components::{
    Coin, Collider, Destructible, Hazard, HazardKind, Platform, Player, PlayerState, Powerup,
    PowerupKind, Sprite, Transform, Velocity,
};
use crate::config::*;
use crate::director;
use crate::effects::EffectPool;
use crate::entity::{DenseSlot, EntityAllocator, EntityId};
use crate::events::{EventQueue, GameEvent, GameEventKind};
use crate::falling::{FallingState, RunMode};
use crate::profile::{elapsed_ns, TickProfile};
use crate::segment::{DestructibleSpec, GeneratedCourse, SegmentDefinition, SegmentIntent};
use crate::streamer::SegmentStreamer;
use std::time::Instant;

/// Host quality profiles (§16) — caps secondary effects only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum QualityProfile {
    Low = 0,
    Balanced = 1,
    High = 2,
    Ultra = 3,
}

impl QualityProfile {
    pub fn from_u32(v: u32) -> Self {
        match v {
            0 => Self::Low,
            2 => Self::High,
            3 => Self::Ultra,
            _ => Self::Balanced,
        }
    }

    pub fn effect_cap(self) -> usize {
        match self {
            Self::Low => 24,
            Self::Balanced => 64,
            Self::High => 96,
            Self::Ultra => EFFECT_POOL_CAP,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct InputSnapshot {
    pub move_x: f32,
    pub jump_pressed: bool,
    pub jump_held: bool,
    pub dash_pressed: bool,
    pub roll_held: bool,
    pub restart_pressed: bool,
    pub qte_pressed: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct CameraState {
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
    pub viewport_w: f32,
    pub viewport_h: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
            viewport_w: 432.0,
            viewport_h: 768.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ExtractSprite {
    pub x: f32,
    pub y: f32,
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

#[derive(Clone, Copy, Debug, Default)]
pub struct RunState {
    pub alive: bool,
    pub finished: bool,
    pub score: u32,
    pub coins: u32,
    pub distance: f32,
    pub player_state: u32,
    pub seed: u64,
    pub segment_index: u32,
    pub segment_count: u32,
    pub ability_mask: u32,
    pub qte_active: u32,
    pub qte_prompt: u32,
    pub falling: u32,
    pub falling_time: f32,
}

pub struct World {
    entities: EntityAllocator,
    transforms: DenseSlot<Transform>,
    velocities: DenseSlot<Velocity>,
    colliders: DenseSlot<Collider>,
    sprites: DenseSlot<Sprite>,
    players: DenseSlot<Player>,
    platforms: DenseSlot<Platform>,
    hazards: DenseSlot<Hazard>,
    coins: DenseSlot<Coin>,
    powerups: DenseSlot<Powerup>,
    destructibles: DenseSlot<Destructible>,
    /// Entity → segment index ownership for streaming despawn.
    segment_of: DenseSlot<u32>,
    player_id: EntityId,
    camera: CameraState,
    seed: u64,
    /// When true, `reset` regenerates the same seed (CLI `--seed` / tests).
    seed_locked: bool,
    segments: Vec<SegmentDefinition>,
    /// Far edge of generated buffer (grows forever in endless mode).
    course_end: f32,
    streamer: SegmentStreamer,
    /// When true, streamer does not despawn (headless full spawn).
    streaming_enabled: bool,
    /// Endless run: generate beyond the initial buffer.
    endless: bool,
    /// Seconds survived this run (drives speed ramp).
    run_time: f32,
    /// Current auto-run speed (capped).
    run_speed: f32,
    abilities: AbilityManager,
    effects: EffectPool,
    run_mode: RunMode,
    falling: Option<FallingState>,
    events: EventQueue,
    prev_grounded: bool,
    prev_qte_active: bool,
    profile: TickProfile,
    anim: AnimController,
    quality: QualityProfile,
}

impl World {
    /// Play session: random course; restart rolls a new seed (connectivity still validated).
    pub fn new() -> Self {
        Self::new_with_seed(crate::rng::random_seed(), false)
    }

    /// Locked seed: same layout on every `reset` (repro / `--seed`).
    pub fn new_seeded(seed: u64) -> Self {
        Self::new_with_seed(seed, true)
    }

    fn new_with_seed(seed: u64, seed_locked: bool) -> Self {
        let course = director::generate_course(seed);
        Self::from_course(course, true, seed_locked)
    }

    /// Spawn every segment up front (headless path proof).
    pub fn from_course_fully_spawned(course: &GeneratedCourse) -> Self {
        Self::from_course(course.clone(), false, true)
    }

    fn from_course(course: GeneratedCourse, streaming: bool, seed_locked: bool) -> Self {
        let mut world = Self {
            entities: EntityAllocator::new(),
            transforms: DenseSlot::new(),
            velocities: DenseSlot::new(),
            colliders: DenseSlot::new(),
            sprites: DenseSlot::new(),
            players: DenseSlot::new(),
            platforms: DenseSlot::new(),
            hazards: DenseSlot::new(),
            coins: DenseSlot::new(),
            powerups: DenseSlot::new(),
            destructibles: DenseSlot::new(),
            segment_of: DenseSlot::new(),
            player_id: EntityId::INVALID,
            camera: CameraState::default(),
            seed: course.seed,
            seed_locked,
            course_end: course.course_end,
            streamer: SegmentStreamer::new(course.segments.len()),
            segments: course.segments,
            streaming_enabled: streaming,
            // Play / streaming worlds are endless; headless full-spawn proves a finite buffer.
            endless: streaming,
            run_time: 0.0,
            run_speed: AUTO_RUN_SPEED,
            abilities: AbilityManager::default(),
            effects: EffectPool::new(),
            run_mode: RunMode::Run,
            falling: None,
            events: EventQueue::new(),
            prev_grounded: false,
            prev_qte_active: false,
            profile: TickProfile::default(),
            anim: AnimController::default(),
            quality: QualityProfile::Balanced,
        };
        world.effects.set_active_cap(world.quality.effect_cap());
        world.spawn_player();
        if streaming {
            // Prefill far ahead so the tip never lags behind the runner.
            world.ensure_segments_ahead();
            world.sync_streamer();
        } else {
            for i in 0..world.segments.len() {
                world.spawn_segment_entities(i);
                world.streamer.mark_active(i, true);
            }
        }
        world
    }

    pub fn reset(&mut self) {
        if self.seed_locked {
            *self = Self::new_with_seed(self.seed, true);
        } else {
            *self = Self::new_with_seed(crate::rng::random_seed(), false);
        }
    }

    pub fn seed_locked(&self) -> bool {
        self.seed_locked
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn course_end(&self) -> f32 {
        self.course_end
    }

    pub fn run_speed(&self) -> f32 {
        self.run_speed
    }

    pub fn run_time(&self) -> f32 {
        self.run_time
    }

    pub fn is_endless(&self) -> bool {
        self.endless
    }

    pub fn segments(&self) -> &[SegmentDefinition] {
        &self.segments
    }

    pub fn drain_events(&mut self, out: &mut [GameEvent]) -> usize {
        self.events.drain_into(out)
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    pub fn profile(&self) -> TickProfile {
        self.profile
    }

    pub fn take_profile(&mut self) -> TickProfile {
        let p = self.profile;
        self.profile.clear();
        p
    }

    pub fn reset_profile(&mut self) {
        self.profile.clear();
    }

    /// Bench helper: remove lethal interactables so long ticks stay alive.
    pub fn clear_hazards_and_powerups(&mut self) {
        let clear: Vec<EntityId> = self
            .hazards
            .iter()
            .map(|(id, _)| id)
            .chain(self.powerups.iter().map(|(id, _)| id))
            .collect();
        for id in clear {
            self.despawn_entity(id);
        }
    }

    fn push_event(&mut self, kind: GameEventKind, x: f32, y: f32, data: u32) {
        self.events.push(GameEvent { kind, x, y, data });
    }

    fn player_xy(&self) -> (f32, f32) {
        self.transforms
            .get(self.player_id)
            .map(|t| (t.x, t.y))
            .unwrap_or((0.0, 0.0))
    }

    fn update_run_speed(&mut self, dt: f32) {
        self.run_time += dt;
        self.run_speed = (AUTO_RUN_SPEED + self.run_time * SPEED_RAMP_PER_SEC).min(MAX_RUN_SPEED);
    }

    fn append_segment(&mut self) {
        let index = self.segments.len();
        let world_x0 = self
            .segments
            .last()
            .map(|s| s.world_x1())
            .unwrap_or(-40.0);
        let prev = self.segments.last().map(|s| s.intent);
        let mut seg = director::generate_next_segment(self.seed, index, prev, world_x0);
        // Ensure seam continuity; if invalid, place a short recovery bridge.
        let mut ok = crate::validate::validate_segment(&seg).is_ok();
        if ok {
            if let Some(last) = self.segments.last() {
                // Validate only the seam pair without cloning the whole course.
                let pair = [last.clone(), seg.clone()];
                if crate::validate::validate_continuity(&pair).is_err() {
                    ok = false;
                }
            }
        }
        if !ok {
            seg = crate::generate::safe_recovery(world_x0, 220.0);
        }
        self.course_end = seg.world_x1();
        self.segments.push(seg);
        self.streamer.push_inactive();
    }

    fn ensure_segments_ahead(&mut self) {
        if !self.endless {
            return;
        }
        let px = self
            .transforms
            .get(self.player_id)
            .map(|t| t.x)
            .unwrap_or(0.0);
        let cur = SegmentStreamer::segment_index_at(&self.segments, px);
        // Distance + index: never let the generated tip lag the runner.
        let mut guard = 0;
        loop {
            let tip = self
                .segments
                .last()
                .map(|s| s.world_x1())
                .unwrap_or(-40.0);
            let ahead_segs = self.segments.len().saturating_sub(cur + 1);
            if tip >= px + GENERATE_AHEAD_PX && ahead_segs >= GENERATE_AHEAD_SEGMENTS {
                break;
            }
            self.append_segment();
            guard += 1;
            if guard > 128 {
                break;
            }
        }
    }

    pub fn entity_count_approx(&self) -> usize {
        self.sprites.iter().count()
    }

    pub fn player_transform(&self) -> Option<Transform> {
        self.transforms.get(self.player_id).copied()
    }

    pub fn player_collider(&self) -> Option<Collider> {
        self.colliders.get(self.player_id).copied()
    }

    pub fn player_grounded(&self) -> bool {
        self.players
            .get(self.player_id)
            .map(|p| p.grounded)
            .unwrap_or(false)
    }

    pub fn platform_aabbs(&self) -> Vec<Aabb> {
        self.platforms
            .iter()
            .filter_map(|(id, _)| {
                let t = self.transforms.get(id)?;
                let c = self.colliders.get(id)?;
                Some(Aabb::new(t.x, t.y, c.w, c.h))
            })
            .collect()
    }

    pub fn hazard_aabbs(&self) -> Vec<(f32, f32, f32, f32, HazardKind)> {
        self.hazards
            .iter()
            .filter_map(|(id, h)| {
                let t = self.transforms.get(id)?;
                let c = self.colliders.get(id)?;
                Some((t.x, t.y, c.w, c.h, h.kind))
            })
            .collect()
    }

    pub fn set_viewport(&mut self, w: f32, h: f32) {
        self.camera.viewport_w = w.max(1.0);
        self.camera.viewport_h = h.max(1.0);
    }

    pub fn camera(&self) -> CameraState {
        self.camera
    }

    pub fn player_id(&self) -> EntityId {
        self.player_id
    }

    pub fn run_state(&self) -> RunState {
        let p = self.players.get(self.player_id);
        let coins = p.map(|p| p.score_coins).unwrap_or(0);
        let max_x = p.map(|p| p.max_x).unwrap_or(0.0);
        let distance = max_x.max(0.0);
        let distance_score = (distance / DISTANCE_SCORE_DIV).floor() as u32;
        let px = self.transforms.get(self.player_id).map(|t| t.x).unwrap_or(0.0);
        let seg_i = SegmentStreamer::segment_index_at(&self.segments, px) as u32;
        RunState {
            alive: p.map(|p| p.state != PlayerState::Dead).unwrap_or(false),
            finished: p.map(|p| p.finished).unwrap_or(false),
            score: coins * COIN_VALUE + distance_score,
            coins,
            distance,
            player_state: p.map(|p| p.state as u32).unwrap_or(0),
            seed: self.seed,
            segment_index: seg_i,
            segment_count: self.segments.len() as u32,
            ability_mask: self.abilities.active_mask(),
            qte_active: if self.abilities.qte_active() { 1 } else { 0 },
            qte_prompt: self
                .abilities
                .qte
                .as_ref()
                .map(|q| q.prompt_index())
                .unwrap_or(0),
            falling: if self.run_mode.is_falling_like() { 1 } else { 0 },
            falling_time: self.falling.map(|f| f.mode_timer).unwrap_or(0.0),
        }
    }

    pub fn run_mode(&self) -> RunMode {
        self.run_mode
    }

    pub fn abilities(&self) -> &AbilityManager {
        &self.abilities
    }

    pub fn abilities_mut(&mut self) -> &mut AbilityManager {
        &mut self.abilities
    }

    #[allow(dead_code)]
    pub(crate) fn spawn_platform(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        tint: (f32, f32, f32),
    ) -> EntityId {
        self.spawn_platform_tagged(x, y, w, h, tint, None)
    }

    fn spawn_platform_tagged(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        tint: (f32, f32, f32),
        seg: Option<u32>,
    ) -> EntityId {
        let id = self.entities.spawn();
        self.transforms.insert(id, Transform { x, y });
        self.colliders.insert(id, Collider { w, h });
        self.sprites.insert(
            id,
            Sprite::tinted(w, h, tint.0, tint.1, tint.2, SOLID_TEXTURE_ID),
        );
        self.platforms.insert(id, Platform);
        if let Some(s) = seg {
            self.segment_of.insert(id, s);
        }
        id
    }

    #[allow(dead_code)]
    pub(crate) fn spawn_hazard(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        kind: HazardKind,
        tint: (f32, f32, f32),
    ) -> EntityId {
        self.spawn_hazard_full(
            x,
            y,
            w,
            h,
            kind,
            tint,
            CollisionCategory::Normal,
            None,
            None,
        )
    }

    fn spawn_hazard_full(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        kind: HazardKind,
        tint: (f32, f32, f32),
        category: CollisionCategory,
        destructible: Option<DestructibleSpec>,
        seg: Option<u32>,
    ) -> EntityId {
        let id = self.entities.spawn();
        self.transforms.insert(id, Transform { x, y });
        self.colliders.insert(id, Collider { w, h });
        self.sprites.insert(
            id,
            Sprite::tinted(w, h, tint.0, tint.1, tint.2, SOLID_TEXTURE_ID),
        );
        self.hazards.insert(id, Hazard { kind, category });
        if let Some(d) = destructible {
            self.destructibles.insert(
                id,
                Destructible {
                    health: d.health,
                    reward_coins: d.reward_coins,
                    material: d.material,
                },
            );
        }
        if let Some(s) = seg {
            self.segment_of.insert(id, s);
        }
        id
    }

    #[allow(dead_code)] // used by Phase 6 unit tests
    pub(crate) fn spawn_powerup(&mut self, kind: PowerupKind, x: f32, y: f32) -> EntityId {
        self.spawn_powerup_tagged(kind, x, y, None)
    }

    fn spawn_powerup_tagged(
        &mut self,
        kind: PowerupKind,
        x: f32,
        y: f32,
        seg: Option<u32>,
    ) -> EntityId {
        let id = self.entities.spawn();
        let (w, h) = kind.pickup_size();
        // Pedestal only — glyph parts are drawn in extract for a readable silhouette.
        let ped = POWERUP_PEDESTAL_RGB;
        self.transforms.insert(id, Transform { x, y });
        self.colliders.insert(id, Collider { w, h });
        self.sprites.insert(
            id,
            Sprite::tinted(w, h, ped.0, ped.1, ped.2, SOLID_TEXTURE_ID),
        );
        self.powerups.insert(id, Powerup { kind });
        if let Some(seg_i) = seg {
            self.segment_of.insert(id, seg_i);
        }
        id
    }

    #[allow(dead_code)]
    pub(crate) fn spawn_coin(&mut self, x: f32, y: f32) -> EntityId {
        self.spawn_coin_tagged(x, y, None)
    }

    fn spawn_coin_tagged(&mut self, x: f32, y: f32, seg: Option<u32>) -> EntityId {
        let id = self.entities.spawn();
        let s = 18.0;
        self.transforms.insert(id, Transform { x, y });
        self.colliders.insert(id, Collider { w: s, h: s });
        self.sprites.insert(
            id,
            Sprite::tinted(s, s, 1.0, 0.85, 0.2, SOLID_TEXTURE_ID),
        );
        self.coins.insert(id, Coin { value: 1 });
        if let Some(seg_i) = seg {
            self.segment_of.insert(id, seg_i);
        }
        id
    }

    fn spawn_player(&mut self) {
        let player = self.entities.spawn();
        self.transforms.insert(
            player,
            Transform {
                x: PLAYER_SPAWN_X,
                y: PLAYER_SPAWN_Y,
            },
        );
        self.velocities.insert(player, Velocity::default());
        self.colliders.insert(
            player,
            Collider {
                w: PLAYER_W,
                h: PLAYER_H,
            },
        );
        self.sprites.insert(
            player,
            Sprite::tinted(PLAYER_W, PLAYER_H, 0.95, 0.55, 0.20, DEFAULT_TEXTURE_ID),
        );
        let mut p = Player::default();
        p.base_h = PLAYER_H;
        self.players.insert(player, p);
        self.player_id = player;

        self.camera.x = PLAYER_SPAWN_X + PLAYER_W * 0.5 + CAMERA_LOOKAHEAD * 0.3;
        self.camera.y = PLAYER_SPAWN_Y + PLAYER_H * 0.5;
    }

    fn spawn_segment_entities(&mut self, index: usize) {
        let Some(seg) = self.segments.get(index).cloned() else {
            return;
        };
        let tag = Some(index as u32);
        for p in &seg.platforms {
            self.spawn_platform_tagged(p.x, p.y, p.w, p.h, p.tint, tag);
        }
        for h in &seg.hazards {
            self.spawn_hazard_full(
                h.x,
                h.y,
                h.w,
                h.h,
                h.kind,
                h.tint,
                h.category,
                h.destructible,
                tag,
            );
        }
        for c in &seg.coins {
            self.spawn_coin_tagged(c.x, c.y, tag);
        }
        for pu in &seg.powerups {
            self.spawn_powerup_tagged(pu.kind, pu.x, pu.y, tag);
        }
    }

    fn despawn_segment_entities(&mut self, index: usize) {
        let tag = index as u32;
        let ids: Vec<EntityId> = self
            .segment_of
            .iter()
            .filter_map(|(id, s)| (*s == tag).then_some(id))
            .collect();
        for id in ids {
            self.despawn_entity(id);
        }
    }

    fn sync_streamer(&mut self) {
        let px = self
            .transforms
            .get(self.player_id)
            .map(|t| t.x)
            .unwrap_or(0.0);
        let (spawn, despawn) = self.streamer.plan(&self.segments, px);
        for i in spawn {
            self.spawn_segment_entities(i);
            self.streamer.mark_active(i, true);
        }
        for i in despawn {
            self.despawn_segment_entities(i);
            self.streamer.mark_active(i, false);
        }
    }

    pub fn tick(&mut self, input: InputSnapshot, dt: f32) {
        let dt = dt.clamp(0.0, 0.05);
        self.profile.ticks = self.profile.ticks.saturating_add(1);
        let id = self.player_id;
        if !self.entities.is_alive(id) {
            return;
        }

        // Dead: wait for restart
        if self.players.get(id).map(|p| p.state == PlayerState::Dead).unwrap_or(false) {
            if let Some(p) = self.players.get_mut(id) {
                p.death_timer += dt;
            }
            let auto = self
                .players
                .get(id)
                .map(|p| p.death_timer >= RESTART_DELAY)
                .unwrap_or(false);
            if input.restart_pressed || auto {
                // Preserve profile across reset for Phase 10 benches.
                let saved = self.profile;
                self.reset();
                self.profile = saved;
            }
            return;
        }

        if input.restart_pressed {
            let saved = self.profile;
            self.reset();
            self.profile = saved;
            return;
        }

        self.update_run_speed(dt);
        // Generate + spawn BEFORE movement so the runner never enters unloaded void.
        {
            let t0 = Instant::now();
            if self.streaming_enabled {
                self.ensure_segments_ahead();
                self.sync_streamer();
            }
            self.profile.streamer_ns += elapsed_ns(t0);
        }

        {
            let t0 = Instant::now();
            self.collect_powerups();
            let _ = self.abilities.tick_qte(dt, input.qte_pressed);
            self.abilities.tick(dt);
            let qte_now = self.abilities.qte_active();
            if qte_now && !self.prev_qte_active {
                let (x, y) = self.player_xy();
                let prompt = self
                    .abilities
                    .qte
                    .as_ref()
                    .map(|q| q.prompt_index())
                    .unwrap_or(0);
                self.push_event(GameEventKind::QteTick, x, y, prompt);
            }
            self.prev_qte_active = qte_now;

            self.update_falling_mode(dt);
            self.apply_player_control(input, dt);
            self.profile.control_ns += elapsed_ns(t0);
        }

        let dashing = self
            .players
            .get(id)
            .map(|p| p.state == PlayerState::Dashing)
            .unwrap_or(false);
        let steps = if dashing { DASH_SUBSTEPS } else { 1 };
        let step_dt = dt / steps as f32;
        for _ in 0..steps {
            let t0 = Instant::now();
            self.integrate_player_swept(step_dt);
            self.integrate_non_player_velocities(step_dt);
            self.profile.integrate_ns += elapsed_ns(t0);
            let t1 = Instant::now();
            self.resolve_player_collisions();
            self.profile.collision_ns += elapsed_ns(t1);
        }

        let grounded = self
            .players
            .get(id)
            .map(|p| p.grounded)
            .unwrap_or(false);
        if grounded && !self.prev_grounded {
            let (x, y) = self.player_xy();
            self.push_event(GameEventKind::Land, x, y, 0);
        }
        self.prev_grounded = grounded;

        {
            let t0 = Instant::now();
            self.apply_magnet(dt);
            self.resolve_hazards_and_coins();
            self.apply_super_strength_damage();
            self.profile.hazards_ns += elapsed_ns(t0);
        }

        {
            let t0 = Instant::now();
            self.tick_ability_effects(dt);
            self.effects.tick(dt);
            self.update_player_visual_state(dt);
            self.update_score_progress();
            self.check_falling_transitions();
            self.check_fall_death();
            if !self.endless {
                self.check_finish();
            }
            self.profile.other_ns += elapsed_ns(t0);
        }
        // Catch up again after displacement (dash / hitch / large dt).
        {
            let t0 = Instant::now();
            if self.streaming_enabled {
                self.ensure_segments_ahead();
                self.sync_streamer();
            }
            self.profile.streamer_ns += elapsed_ns(t0);
        }
        self.update_camera(dt);
    }

    fn apply_player_control(&mut self, input: InputSnapshot, dt: f32) {
        let id = self.player_id;
        let grounded = self.players.get(id).map(|p| p.grounded).unwrap_or(false);
        let state = self.players.get(id).map(|p| p.state).unwrap_or(PlayerState::Running);

        if let Some(p) = self.players.get_mut(id) {
            p.dash_cooldown = (p.dash_cooldown - dt).max(0.0);
            if input.jump_pressed {
                p.jump_buffer = JUMP_BUFFER_SECONDS;
            } else {
                p.jump_buffer = (p.jump_buffer - dt).max(0.0);
            }
            if matches!(
                p.state,
                PlayerState::Dashing | PlayerState::Rolling
            ) {
                p.state_timer -= dt;
                if p.state_timer <= 0.0 {
                    self.end_special_state(id, grounded);
                }
            }
        }

        // Re-read after possible state end
        let state = self.players.get(id).map(|p| p.state).unwrap_or(state);

        // Start dash (edge press → fixed DASH_DURATION; hold does not extend)
        if input.dash_pressed
            && !matches!(state, PlayerState::Dashing | PlayerState::Dead)
            && self
                .players
                .get(id)
                .map(|p| p.dash_cooldown <= 0.0)
                .unwrap_or(false)
        {
            if let Some(p) = self.players.get_mut(id) {
                p.state = PlayerState::Dashing;
                p.state_timer = DASH_DURATION;
                p.dash_cooldown = DASH_COOLDOWN;
                p.jump_buffer = 0.0;
            }
            self.restore_standing_collider(id);
            let (x, y) = self.player_xy();
            self.push_event(GameEventKind::Dash, x, y, 0);
        }

        // Roll: one timed crouch per press. Hold does not extend or chain.
        // May start airborne so Jump→Roll into a LowBar stays viable.
        if !input.roll_held {
            if let Some(p) = self.players.get_mut(id) {
                p.roll_gate = false;
            }
        }
        let state = self.players.get(id).map(|p| p.state).unwrap_or(state);
        let roll_gate = self.players.get(id).map(|p| p.roll_gate).unwrap_or(false);
        if input.roll_held
            && !roll_gate
            && !matches!(
                state,
                PlayerState::Dashing | PlayerState::Rolling | PlayerState::Dead
            )
        {
            self.begin_roll(id);
            if let Some(p) = self.players.get_mut(id) {
                p.roll_gate = true;
            }
            let (x, y) = self.player_xy();
            self.push_event(GameEventKind::Roll, x, y, 0);
        }

        let state = self.players.get(id).map(|p| p.state).unwrap_or(PlayerState::Running);
        let run_speed = self.run_speed;
        let move_mod = self.abilities.movement_modifiers();
        let rolling = state == PlayerState::Rolling;
        // Roll bypasses Slow Fall so the player can still drop quickly.
        let g_scale = if rolling { 1.0 } else { move_mod.gravity_scale };
        let fall_scale = if rolling {
            1.0
        } else {
            move_mod.max_fall_scale
        };
        let _ = move_mod.air_control;
        let falling_strafe = self.run_mode.uses_strafe();
        let (shaft_lo, shaft_hi) = self
            .falling
            .map(|f| (f.shaft_x0 + 2.0, f.shaft_x1 - PLAYER_W - 2.0))
            .unwrap_or((f32::NEG_INFINITY, f32::INFINITY));
        let mut jumped = false;
        {
            let grounded = self.players.get(id).map(|p| p.grounded).unwrap_or(false);
            let can_jump = grounded
                && !matches!(state, PlayerState::Dashing | PlayerState::Rolling)
                && self
                    .players
                    .get(id)
                    .map(|p| p.jump_buffer > 0.0)
                    .unwrap_or(false);
            if let Some(vel) = self.velocities.get_mut(id) {
                match state {
                    PlayerState::Dashing => {
                        if falling_strafe {
                            // Dash boosts horizontal intent during Falling Mode.
                            let dir = if input.move_x.abs() > 0.1 {
                                input.move_x.signum()
                            } else {
                                1.0
                            };
                            vel.x = dir * (FALLING_STRAFE_SPEED + DASH_BONUS * 0.5);
                        } else {
                            vel.x = run_speed + DASH_BONUS;
                        }
                        // Keep vertical from gravity lightly
                        vel.y += GRAVITY * g_scale * dt * 0.35;
                    }
                    PlayerState::Dead => {}
                    _ => {
                        if falling_strafe {
                            vel.x = input.move_x.clamp(-1.0, 1.0) * FALLING_STRAFE_SPEED;
                        } else if self.run_mode == RunMode::FallingExit {
                            // Hold still horizontally while snapping to exit lip.
                            vel.x = 0.0;
                        } else {
                            // Auto-run only — no strafe / no backpedal.
                            let _ = input.move_x;
                            vel.x = run_speed;
                        }
                        vel.y += GRAVITY * g_scale * dt;
                    }
                }
                let max_fall = MAX_FALL_SPEED * fall_scale;
                if vel.y > max_fall {
                    vel.y = max_fall;
                }

                if can_jump {
                    vel.y = JUMP_VELOCITY;
                    jumped = true;
                } else if !input.jump_held && vel.y < 0.0 && state != PlayerState::Dashing {
                    vel.y *= 0.55;
                }
            }
            if jumped {
                if let Some(p) = self.players.get_mut(id) {
                    p.grounded = false;
                    p.jump_buffer = 0.0;
                    p.state = PlayerState::Jumping;
                }
            }
        }
        if jumped {
            let (jx, jy) = self.player_xy();
            self.push_event(GameEventKind::Jump, jx, jy, 0);
        }

        // Clamp strafe inside the shaft walls.
        if falling_strafe {
            if let Some(t) = self.transforms.get_mut(id) {
                t.x = t.x.clamp(shaft_lo, shaft_hi);
            }
        }

        // Sync Running/Jumping/Falling when not in special
        if let Some(p) = self.players.get_mut(id) {
            if !matches!(
                p.state,
                PlayerState::Dashing | PlayerState::Rolling | PlayerState::Dead
            ) {
                let vy = self.velocities.get(id).map(|v| v.y).unwrap_or(0.0);
                p.state = if p.grounded {
                    PlayerState::Running
                } else if vy < 0.0 {
                    PlayerState::Jumping
                } else {
                    PlayerState::Falling
                };
            }
        }
    }

    fn update_falling_mode(&mut self, dt: f32) {
        match self.run_mode {
            RunMode::FallingEnter => {
                if let Some(f) = self.falling.as_mut() {
                    f.timer += dt;
                    f.mode_timer += dt;
                    if f.enter_done() {
                        self.run_mode = RunMode::Falling;
                        if let Some(f) = self.falling.as_mut() {
                            f.timer = 0.0;
                        }
                    }
                }
            }
            RunMode::Falling => {
                if let Some(f) = self.falling.as_mut() {
                    f.mode_timer += dt;
                }
            }
            RunMode::FallingExit => {
                if let Some(f) = self.falling.as_mut() {
                    f.timer += dt;
                    let t = (f.timer / FALLING_EXIT_DURATION).clamp(0.0, 1.0);
                    let id = self.player_id;
                    let (ex, ey) = (f.exit_x + 8.0, f.exit_y - PLAYER_H - 2.0);
                    if let Some(tr) = self.transforms.get_mut(id) {
                        tr.x += (ex - tr.x) * t.max(0.15);
                        tr.y += (ey - tr.y) * t.max(0.15);
                    }
                    if let Some(v) = self.velocities.get_mut(id) {
                        v.x = 0.0;
                        v.y = 0.0;
                    }
                    if f.exit_done() {
                        if let Some(tr) = self.transforms.get_mut(id) {
                            tr.x = ex;
                            tr.y = ey;
                        }
                        self.run_mode = RunMode::Run;
                        self.falling = None;
                        if let Some(p) = self.players.get_mut(id) {
                            p.grounded = true;
                            p.state = PlayerState::Running;
                        }
                    }
                }
            }
            RunMode::Run => {}
        }
    }

    fn try_begin_falling(&mut self) {
        if self.run_mode != RunMode::Run {
            return;
        }
        let id = self.player_id;
        let Some(t) = self.transforms.get(id).copied() else {
            return;
        };
        let grounded = self.players.get(id).map(|p| p.grounded).unwrap_or(false);
        if grounded {
            return;
        }
        let px = t.x + PLAYER_W * 0.5;
        let seg_i = SegmentStreamer::segment_index_at(&self.segments, px);
        let Some(seg) = self.segments.get(seg_i) else {
            return;
        };
        if seg.intent != SegmentIntent::FallingShaft {
            return;
        }
        let Some(shaft) = seg.shaft else {
            return;
        };
        if px < shaft.pit_x0 || px > shaft.pit_x1 {
            return;
        }
        // Over the pit and airborne → enter Falling Mode.
        self.falling = Some(FallingState::new(
            shaft.pit_x0,
            shaft.pit_x1,
            shaft.floor_y,
            shaft.enter_y,
            shaft.exit_x,
            shaft.exit_y,
        ));
        self.run_mode = RunMode::FallingEnter;
        self.effects.spawn_burst(t.x, t.y, 0.4, 0.85, 1.0, 8);
        self.push_event(GameEventKind::FallEnter, t.x, t.y, 0);
    }

    fn check_falling_transitions(&mut self) {
        self.try_begin_falling();

        if self.run_mode != RunMode::Falling {
            return;
        }
        let id = self.player_id;
        let grounded = self.players.get(id).map(|p| p.grounded).unwrap_or(false);
        let Some(f) = self.falling else {
            return;
        };
        let on_floor = grounded
            && self
                .transforms
                .get(id)
                .map(|t| (t.y + PLAYER_H - f.floor_y).abs() < 12.0 || t.y + PLAYER_H >= f.floor_y - 4.0)
                .unwrap_or(false);

        if on_floor || f.timed_out() {
            let t = self.transforms.get(id).copied();
            if let Some(tr) = t {
                self.effects
                    .spawn_burst(tr.x, tr.y, 0.3, 1.0, 0.5, 10);
                self.push_event(GameEventKind::FallExit, tr.x, tr.y, 0);
            }
            if let Some(fs) = self.falling.as_mut() {
                fs.timer = 0.0;
            }
            self.run_mode = RunMode::FallingExit;
            self.restore_standing_collider(id);
        }
    }

    fn begin_roll(&mut self, id: EntityId) {
        let base_h = self.players.get(id).map(|p| p.base_h).unwrap_or(PLAYER_H);
        let new_h = base_h * ROLL_HEIGHT_SCALE;
        if let Some(c) = self.colliders.get_mut(id) {
            let old_h = c.h;
            c.h = new_h;
            if let Some(t) = self.transforms.get_mut(id) {
                t.y += old_h - new_h; // keep feet planted
            }
        }
        if let Some(s) = self.sprites.get_mut(id) {
            s.h = new_h;
        }
        if let Some(p) = self.players.get_mut(id) {
            p.state = PlayerState::Rolling;
            p.state_timer = ROLL_DURATION;
        }
    }

    fn end_special_state(&mut self, id: EntityId, grounded: bool) {
        self.restore_standing_collider(id);
        if let Some(p) = self.players.get_mut(id) {
            p.state = if grounded {
                PlayerState::Running
            } else {
                PlayerState::Falling
            };
            p.state_timer = 0.0;
        }
    }

    fn restore_standing_collider(&mut self, id: EntityId) {
        let base_h = self.players.get(id).map(|p| p.base_h).unwrap_or(PLAYER_H);
        if let Some(c) = self.colliders.get_mut(id) {
            let old_h = c.h;
            if (old_h - base_h).abs() > 0.1 {
                c.h = base_h;
                if let Some(t) = self.transforms.get_mut(id) {
                    t.y -= base_h - old_h;
                }
            }
        }
        if let Some(s) = self.sprites.get_mut(id) {
            s.h = base_h;
            s.w = PLAYER_W;
        }
    }

    fn integrate_non_player_velocities(&mut self, dt: f32) {
        let player = self.player_id;
        let ids: Vec<EntityId> = self
            .velocities
            .iter()
            .map(|(id, _)| id)
            .filter(|id| self.entities.is_alive(*id) && *id != player)
            .collect();
        for id in ids {
            let (vx, vy) = match self.velocities.get(id) {
                Some(v) => (v.x, v.y),
                None => continue,
            };
            if let Some(t) = self.transforms.get_mut(id) {
                t.x += vx * dt;
                t.y += vy * dt;
            }
        }
    }

    /// Integrate player with swept anti-tunnel on fast falls (§21). Discrete resolve follows.
    fn integrate_player_swept(&mut self, dt: f32) {
        let id = self.player_id;
        if !self.entities.is_alive(id) {
            return;
        }
        if self.players.get(id).map(|p| p.state == PlayerState::Dead).unwrap_or(false) {
            return;
        }
        let (vx, vy) = match self.velocities.get(id) {
            Some(v) => (v.x, v.y),
            None => return,
        };
        let dx = vx * dt;
        let mut dy = vy * dt;
        let Some(body) = self.player_aabb(id) else {
            return;
        };
        let (x0, x1) = self.query_x_window();
        // Only engage sweep when falling fast enough to tunnel through thin platforms.
        if dy > 8.0 {
            for (pid, _) in self.platforms.iter() {
                let Some(t) = self.transforms.get(pid) else {
                    continue;
                };
                let Some(c) = self.colliders.get(pid) else {
                    continue;
                };
                if t.x + c.w < x0 || t.x > x1 {
                    continue;
                }
                let solid = Aabb::new(t.x, t.y, c.w, c.h);
                let (_mx, my, landed) = body.sweep_move(dx, dy, solid);
                if landed && my < dy {
                    dy = my;
                    if let Some(vel) = self.velocities.get_mut(id) {
                        vel.y = 0.0;
                    }
                    if let Some(p) = self.players.get_mut(id) {
                        p.grounded = true;
                    }
                    break;
                }
            }
        }
        if let Some(t) = self.transforms.get_mut(id) {
            t.x += dx;
            t.y += dy;
        }
    }

    fn player_aabb(&self, id: EntityId) -> Option<Aabb> {
        let t = self.transforms.get(id)?;
        let c = self.colliders.get(id)?;
        Some(Aabb::new(t.x, t.y, c.w, c.h))
    }

    fn query_x_window(&self) -> (f32, f32) {
        let px = self
            .transforms
            .get(self.player_id)
            .map(|t| t.x)
            .unwrap_or(0.0);
        (px - COLLISION_QUERY_PAD_X, px + COLLISION_QUERY_PAD_X + PLAYER_W)
    }

    fn resolve_player_collisions(&mut self) {
        let id = self.player_id;
        if !self.entities.is_alive(id) {
            return;
        }
        if self.players.get(id).map(|p| p.state == PlayerState::Dead).unwrap_or(false) {
            return;
        }
        let (x0, x1) = self.query_x_window();
        let mut grounded = false;
        // Broadphase: only platforms overlapping the player's X window (Phase 10).
        let platforms: Vec<Aabb> = self
            .platforms
            .iter()
            .filter_map(|(pid, _)| {
                let t = self.transforms.get(pid)?;
                let c = self.colliders.get(pid)?;
                if t.x + c.w < x0 || t.x > x1 {
                    return None;
                }
                Some(Aabb::new(t.x, t.y, c.w, c.h))
            })
            .collect();

        for _ in 0..3 {
            let Some(mut body) = self.player_aabb(id) else {
                return;
            };
            for solid in &platforms {
                let (dx, dy, landed) = body.resolve_against(*solid);
                if dx == 0.0 && dy == 0.0 {
                    continue;
                }
                body.x += dx;
                body.y += dy;
                if let Some(t) = self.transforms.get_mut(id) {
                    t.x = body.x;
                    t.y = body.y;
                }
                if let Some(vel) = self.velocities.get_mut(id) {
                    // Never zero auto-run X from wall nudge on platforms — only vertical.
                    if dy != 0.0 {
                        if landed {
                            vel.y = 0.0;
                            grounded = true;
                        } else if dy > 0.0 {
                            vel.y = 0.0;
                        }
                    }
                }
            }
        }

        if let Some(player) = self.players.get_mut(id) {
            player.grounded = grounded;
        }
    }

    fn collect_powerups(&mut self) {
        let id = self.player_id;
        let Some(body) = self.player_aabb(id) else {
            return;
        };
        let (x0, x1) = self.query_x_window();
        let hits: Vec<(EntityId, PowerupKind)> = self
            .powerups
            .iter()
            .filter_map(|(pid, pu)| {
                let t = self.transforms.get(pid)?;
                let c = self.colliders.get(pid)?;
                if t.x + c.w < x0 || t.x > x1 {
                    return None;
                }
                let aabb = Aabb::new(t.x, t.y, c.w, c.h);
                body.overlaps(aabb).then_some((pid, pu.kind))
            })
            .collect();
        for (pid, kind) in hits {
            self.abilities.activate(kind.ability());
            let tint = kind.tint();
            let (ex, ey) = self
                .transforms
                .get(pid)
                .map(|t| (t.x, t.y))
                .unwrap_or((0.0, 0.0));
            self.effects
                .spawn_burst(ex, ey, tint.0, tint.1, tint.2, 6);
            self.push_event(GameEventKind::AbilityOn, ex, ey, kind.ability() as u32);
            self.despawn_entity(pid);
        }
    }

    fn apply_magnet(&mut self, dt: f32) {
        if !self.abilities.is_active(AbilityId::Magnet) {
            return;
        }
        let id = self.player_id;
        let Some(pt) = self.transforms.get(id).copied() else {
            return;
        };
        let Some(pc) = self.colliders.get(id).copied() else {
            return;
        };
        let cx = pt.x + pc.w * 0.5;
        let cy = pt.y + pc.h * 0.5;
        let r2 = MAGNET_RADIUS * MAGNET_RADIUS;

        let coin_ids: Vec<EntityId> = self.coins.iter().map(|(cid, _)| cid).collect();
        for cid in coin_ids {
            let Some(ct) = self.transforms.get(cid).copied() else {
                continue;
            };
            let Some(cc) = self.colliders.get(cid).copied() else {
                continue;
            };
            let ox = ct.x + cc.w * 0.5;
            let oy = ct.y + cc.h * 0.5;
            let dx = cx - ox;
            let dy = cy - oy;
            let d2 = dx * dx + dy * dy;
            if d2 > r2 || d2 < 1.0 {
                continue;
            }
            let dist = d2.sqrt();
            let nx = dx / dist;
            let ny = dy / dist;
            if self.velocities.get(cid).is_none() {
                self.velocities.insert(cid, Velocity::default());
            }
            if let Some(v) = self.velocities.get_mut(cid) {
                v.x = (v.x + nx * MAGNET_FORCE * dt) * MAGNET_DAMPING;
                v.y = (v.y + ny * MAGNET_FORCE * dt) * MAGNET_DAMPING;
                let spd = (v.x * v.x + v.y * v.y).sqrt();
                if spd > MAGNET_MAX_SPEED {
                    let s = MAGNET_MAX_SPEED / spd;
                    v.x *= s;
                    v.y *= s;
                }
            }
            if (self.effects.live_count() & 3) == 0 {
                self.effects.spawn_magnet_pull(ox, oy);
            }
        }
    }

    fn apply_super_strength_damage(&mut self) {
        let col = self.abilities.collision_modifiers();
        if !col.strength_active {
            return;
        }
        let id = self.player_id;
        let Some(body) = self.player_aabb(id) else {
            return;
        };
        let hits: Vec<EntityId> = self
            .destructibles
            .iter()
            .filter_map(|(hid, _)| {
                let t = self.transforms.get(hid)?;
                let c = self.colliders.get(hid)?;
                let aabb = Aabb::new(t.x, t.y, c.w, c.h);
                body.overlaps(aabb).then_some(hid)
            })
            .collect();
        for hid in hits {
            let reward = self
                .destructibles
                .get(hid)
                .map(|d| d.reward_coins)
                .unwrap_or(0);
            if let Some(d) = self.destructibles.get_mut(hid) {
                d.health -= STRENGTH_DESTROY_DAMAGE;
            }
            let dead = self
                .destructibles
                .get(hid)
                .map(|d| d.health <= 0.0)
                .unwrap_or(false);
            if dead {
                let (ex, ey) = self
                    .transforms
                    .get(hid)
                    .map(|t| (t.x, t.y))
                    .unwrap_or((0.0, 0.0));
                self.effects.spawn_burst(ex, ey, 1.0, 0.4, 0.15, 10);
                self.push_event(GameEventKind::Destroy, ex, ey, reward);
                if let Some(p) = self.players.get_mut(id) {
                    p.score_coins = p.score_coins.saturating_add(reward);
                }
                self.despawn_entity(hid);
            }
        }
    }

    fn tick_ability_effects(&mut self, dt: f32) {
        let id = self.player_id;
        let Some(t) = self.transforms.get(id).copied() else {
            return;
        };
        if self.abilities.is_active(AbilityId::Ghost) {
            self.effects
                .tick_ghost_trail(dt, t.x, t.y, EFFECT_GHOST_TRAIL_INTERVAL);
        }
        if self.abilities.is_active(AbilityId::SlowFall) {
            let rolling = self
                .players
                .get(id)
                .map(|p| p.state == PlayerState::Rolling)
                .unwrap_or(false);
            if !rolling {
                self.effects
                    .tick_slow_fall(dt, t.x, t.y, EFFECT_SLOW_FALL_INTERVAL);
            }
        }
    }

    fn resolve_hazards_and_coins(&mut self) {
        let id = self.player_id;
        let Some(body) = self.player_aabb(id) else {
            return;
        };
        let state = self.players.get(id).map(|p| p.state).unwrap_or(PlayerState::Running);
        if state == PlayerState::Dead {
            return;
        }
        let col = self.abilities.collision_modifiers();
        let (x0, x1) = self.query_x_window();

        let hazard_hits: Vec<(EntityId, HazardKind, CollisionCategory)> = self
            .hazards
            .iter()
            .filter_map(|(hid, h)| {
                let t = self.transforms.get(hid)?;
                let c = self.colliders.get(hid)?;
                if t.x + c.w < x0 || t.x > x1 {
                    return None;
                }
                let aabb = Aabb::new(t.x, t.y, c.w, c.h);
                if body.overlaps(aabb) {
                    Some((hid, h.kind, h.category))
                } else {
                    None
                }
            })
            .collect();

        for (_hid, kind, category) in hazard_hits {
            if !collision_cat::hazard_applies(category, col.ghost_active) {
                continue;
            }
            if col.invulnerable {
                continue;
            }
            // Destructibles without strength still kill via SolidKill unless dashed.
            let lethal = match kind {
                HazardKind::SolidKill => state != PlayerState::Dashing,
                HazardKind::LowBar => state != PlayerState::Rolling,
            };
            if lethal {
                self.kill_player();
                return;
            }
        }

        let coin_hits: Vec<EntityId> = self
            .coins
            .iter()
            .filter_map(|(cid, _)| {
                let t = self.transforms.get(cid)?;
                let c = self.colliders.get(cid)?;
                if t.x + c.w < x0 || t.x > x1 {
                    return None;
                }
                let aabb = Aabb::new(t.x, t.y, c.w, c.h);
                body.overlaps(aabb).then_some(cid)
            })
            .collect();

        for cid in coin_hits {
            let value = self.coins.get(cid).map(|c| c.value).unwrap_or(1);
            if let Some(p) = self.players.get_mut(id) {
                p.score_coins = p.score_coins.saturating_add(value);
            }
            let (cx, cy) = self
                .transforms
                .get(cid)
                .map(|t| (t.x, t.y))
                .unwrap_or((0.0, 0.0));
            self.effects.spawn_burst(cx, cy, 1.0, 0.85, 0.2, 4);
            self.push_event(GameEventKind::Coin, cx, cy, value);
            self.despawn_entity(cid);
        }
    }

    fn despawn_entity(&mut self, id: EntityId) {
        self.transforms.remove(id);
        self.velocities.remove(id);
        self.colliders.remove(id);
        self.sprites.remove(id);
        self.platforms.remove(id);
        self.hazards.remove(id);
        self.coins.remove(id);
        self.powerups.remove(id);
        self.destructibles.remove(id);
        self.segment_of.remove(id);
        self.players.remove(id);
        self.entities.despawn(id);
    }

    fn kill_player(&mut self) {
        let id = self.player_id;
        let (x, y) = self.player_xy();
        if let Some(v) = self.velocities.get_mut(id) {
            v.x = 0.0;
            v.y = 0.0;
        }
        if let Some(p) = self.players.get_mut(id) {
            p.state = PlayerState::Dead;
            p.death_timer = 0.0;
            p.grounded = false;
        }
        self.abilities.clear();
        self.effects.clear();
        self.run_mode = RunMode::Run;
        self.falling = None;
        self.restore_standing_collider(id);
        self.push_event(GameEventKind::Death, x, y, 0);
    }

    fn check_fall_death(&mut self) {
        let id = self.player_id;
        let Some(t) = self.transforms.get(id) else {
            return;
        };
        if t.y > DEATH_Y
            && self
                .players
                .get(id)
                .map(|p| p.state != PlayerState::Dead)
                .unwrap_or(false)
        {
            self.kill_player();
        }
    }

    fn check_finish(&mut self) {
        let id = self.player_id;
        let Some(t) = self.transforms.get(id) else {
            return;
        };
        if t.x >= self.course_end {
            if let Some(p) = self.players.get_mut(id) {
                if !p.finished && p.state != PlayerState::Dead {
                    p.finished = true;
                }
            }
        }
    }

    fn update_score_progress(&mut self) {
        let id = self.player_id;
        let x = self.transforms.get(id).map(|t| t.x).unwrap_or(0.0);
        if let Some(p) = self.players.get_mut(id) {
            if x > p.max_x {
                p.max_x = x;
            }
        }
    }

    fn update_player_visual_state(&mut self, dt: f32) {
        let id = self.player_id;
        let state = self.players.get(id).map(|p| p.state).unwrap_or(PlayerState::Running);
        let grounded = self.players.get(id).map(|p| p.grounded).unwrap_or(false);
        self.anim.sync_from_player_state(state, grounded, dt);
        let (u0, v0, u1, v1) = self.anim.uv_rect();
        let mut r_g_b = match state {
            PlayerState::Dashing => (0.2, 0.9, 1.0),
            PlayerState::Rolling => (1.0, 0.9, 0.2),
            PlayerState::Dead => (0.45, 0.45, 0.45),
            PlayerState::Jumping | PlayerState::Falling => (0.95, 0.55, 0.20),
            PlayerState::Running => (0.95, 0.55, 0.20),
        };
        let mut a = 1.0_f32;
        if self.abilities.is_active(AbilityId::Ghost) {
            r_g_b = (0.7, 0.75, 1.0);
            a = GHOST_ALPHA;
        } else if self.abilities.is_active(AbilityId::SuperStrength) {
            r_g_b = (1.0, 0.45, 0.2);
        } else if self.abilities.is_active(AbilityId::SlowFall) {
            r_g_b = (0.55, 0.85, 1.0);
        } else if self.abilities.is_active(AbilityId::Magnet) {
            r_g_b = (1.0, 0.9, 0.3);
        }
        if let Some(s) = self.sprites.get_mut(id) {
            s.r = r_g_b.0;
            s.g = r_g_b.1;
            s.b = r_g_b.2;
            s.a = a;
            s.u0 = u0;
            s.v0 = v0;
            s.u1 = u1;
            s.v1 = v1;
        }
        if state == PlayerState::Dashing && self.quality != QualityProfile::Low {
            let (x, y) = self.player_xy();
            self.effects.spawn_dash_trail(x, y);
            self.effects.spawn_speed_lines(x, y);
        }
    }

    pub fn set_quality(&mut self, q: QualityProfile) {
        self.quality = q;
        self.effects.set_active_cap(q.effect_cap());
    }

    pub fn quality(&self) -> QualityProfile {
        self.quality
    }

    pub fn anim_clip_index(&self) -> u32 {
        self.anim.clip as u32
    }

    /// Debug draw AABBs: player + nearby platforms/hazards (x,y,w,h packed).
    pub fn extract_debug_aabbs(&self, out: &mut Vec<(f32, f32, f32, f32)>) {
        out.clear();
        if let Some(b) = self.player_aabb(self.player_id) {
            out.push((b.x, b.y, b.w, b.h));
        }
        let (x0, x1) = self.query_x_window();
        for (pid, _) in self.platforms.iter() {
            if let (Some(t), Some(c)) = (self.transforms.get(pid), self.colliders.get(pid)) {
                if t.x + c.w < x0 || t.x > x1 {
                    continue;
                }
                out.push((t.x, t.y, c.w, c.h));
            }
        }
        for (hid, _) in self.hazards.iter() {
            if let (Some(t), Some(c)) = (self.transforms.get(hid), self.colliders.get(hid)) {
                if t.x + c.w < x0 || t.x > x1 {
                    continue;
                }
                out.push((t.x, t.y, c.w, c.h));
            }
        }
    }

    fn update_camera(&mut self, dt: f32) {
        let id = self.player_id;
        let Some(t) = self.transforms.get(id) else {
            return;
        };
        let Some(c) = self.colliders.get(id) else {
            return;
        };
        let (target_x, target_y) = if self.run_mode.is_falling_like() {
            (
                t.x + c.w * 0.5 + CAMERA_FALL_X_LOOKAHEAD,
                t.y + c.h * 0.5 + CAMERA_FALL_LOOKAHEAD,
            )
        } else {
            (
                t.x + c.w * 0.5 + CAMERA_LOOKAHEAD * 0.5,
                t.y + c.h * 0.5,
            )
        };
        let alpha = 1.0 - (-CAMERA_FOLLOW_LERP * dt).exp();
        self.camera.x += (target_x - self.camera.x) * alpha;
        self.camera.y += (target_y - self.camera.y) * alpha;
    }

    pub fn extract_sprites(&mut self, out: &mut Vec<ExtractSprite>) {
        let t0 = Instant::now();
        out.clear();
        for (id, sprite) in self.sprites.iter() {
            if !self.entities.is_alive(id) {
                continue;
            }
            let Some(t) = self.transforms.get(id) else {
                continue;
            };
            // Power-ups bob visually (hitbox stays fixed) so they read against scenery.
            let bob = if self.powerups.get(id).is_some() {
                (self.run_time * 6.0).sin() * 4.0
            } else {
                0.0
            };
            out.push(ExtractSprite {
                x: t.x,
                y: t.y + bob,
                w: sprite.w,
                h: sprite.h,
                u0: sprite.u0,
                v0: sprite.v0,
                u1: sprite.u1,
                v1: sprite.v1,
                r: sprite.r,
                g: sprite.g,
                b: sprite.b,
                a: sprite.a,
                texture_id: sprite.texture_id,
            });
            // Glyph: Magnet=U, SlowFall=parachute, Ghost=eyes, Strength=cross.
            if let Some(pu) = self.powerups.get(id) {
                for &(ox, oy, w, h, r, g, b) in pu.kind.glyph_parts() {
                    out.push(ExtractSprite {
                        x: t.x + ox,
                        y: t.y + oy + bob,
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
                        texture_id: SOLID_TEXTURE_ID,
                    });
                }
            }
        }
        for p in self.effects.iter_live() {
            out.push(ExtractSprite {
                x: p.x,
                y: p.y,
                w: p.w,
                h: p.h,
                u0: 0.0,
                v0: 0.0,
                u1: 1.0,
                v1: 1.0,
                r: p.r,
                g: p.g,
                b: p.b,
                a: p.a,
                texture_id: SOLID_TEXTURE_ID,
            });
        }
        self.profile.extract_ns += elapsed_ns(t0);
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn land(world: &mut World) {
        // Isolate controller tests from sparse Phase-6 hazards/powerups.
        let clear: Vec<EntityId> = world
            .hazards
            .iter()
            .map(|(id, _)| id)
            .chain(world.powerups.iter().map(|(id, _)| id))
            .collect();
        for id in clear {
            world.despawn_entity(id);
        }
        world.abilities.clear();
        world.effects.clear();

        let id = world.player_id;
        if let Some(t) = world.transforms.get_mut(id) {
            t.x = 120.0;
            t.y = GROUND_Y - PLAYER_H - 2.0;
        }
        if let Some(v) = world.velocities.get_mut(id) {
            *v = Velocity::default();
        }
        if let Some(p) = world.players.get_mut(id) {
            p.state = PlayerState::Running;
            p.death_timer = 0.0;
        }
        let input = InputSnapshot::default();
        for _ in 0..20 {
            world.tick(input, 1.0 / 60.0);
        }
        // Keep player in safe corridor for mechanic tests.
        if let Some(t) = world.transforms.get_mut(id) {
            t.x = 120.0;
            t.y = GROUND_Y - PLAYER_H - 2.0;
        }
    }

    #[test]
    fn auto_run_moves_forward() {
        let mut world = World::new();
        land(&mut world);
        let x0 = world.transforms.get(world.player_id).unwrap().x;
        let input = InputSnapshot::default();
        for _ in 0..30 {
            world.tick(input, 1.0 / 60.0);
        }
        let x1 = world.transforms.get(world.player_id).unwrap().x;
        assert!(x1 > x0 + 50.0, "expected auto-run progress, {x0} -> {x1}");
    }

    #[test]
    fn dash_boosts_speed() {
        let mut world = World::new();
        land(&mut world);
        let mut input = InputSnapshot::default();
        input.dash_pressed = true;
        world.tick(input, 1.0 / 60.0);
        let vx = world.velocities.get(world.player_id).unwrap().x;
        assert!(vx >= world.run_speed() + DASH_BONUS - 1.0);
        assert_eq!(
            world.players.get(world.player_id).unwrap().state,
            PlayerState::Dashing
        );
    }

    #[test]
    fn roll_shrinks_collider() {
        let mut world = World::new();
        land(&mut world);
        let h0 = world.colliders.get(world.player_id).unwrap().h;
        let mut input = InputSnapshot::default();
        input.roll_held = true;
        world.tick(input, 1.0 / 60.0);
        let h1 = world.colliders.get(world.player_id).unwrap().h;
        assert!(h1 < h0);
        assert_eq!(
            world.players.get(world.player_id).unwrap().state,
            PlayerState::Rolling
        );
    }

    #[test]
    fn roll_hold_does_not_chain_and_ends() {
        let mut world = World::new();
        land(&mut world);
        let mut input = InputSnapshot::default();
        input.roll_held = true;
        world.tick(input, 1.0 / 60.0);
        assert_eq!(
            world.players.get(world.player_id).unwrap().state,
            PlayerState::Rolling
        );
        // Hold through the full duration — must end, must not re-enter while held.
        let steps = (ROLL_DURATION * 60.0).ceil() as u32 + 5;
        for _ in 0..steps {
            world.tick(input, 1.0 / 60.0);
        }
        assert_ne!(
            world.players.get(world.player_id).unwrap().state,
            PlayerState::Rolling
        );
        assert!(world.players.get(world.player_id).unwrap().roll_gate);
    }

    #[test]
    fn auto_run_ignores_backpedal() {
        let mut world = World::new();
        land(&mut world);
        let mut input = InputSnapshot::default();
        input.move_x = -1.0;
        world.tick(input, 1.0 / 60.0);
        let vx = world.velocities.get(world.player_id).unwrap().x;
        assert!((vx - world.run_speed()).abs() < 0.1);
    }

    #[test]
    fn speed_ramps_then_caps() {
        let mut world = World::new_seeded(42);
        assert!((world.run_speed() - AUTO_RUN_SPEED).abs() < 0.1);
        let input = InputSnapshot::default();
        // ~30s at 60Hz exceeds the ramp to MAX_RUN_SPEED.
        for _ in 0..1800 {
            // park in safe zone so we don't die while ramping
            if let Some(t) = world.transforms.get_mut(world.player_id) {
                t.x = 80.0;
                t.y = GROUND_Y - PLAYER_H - 2.0;
            }
            world.tick(input, 1.0 / 60.0);
        }
        assert!(world.run_speed() > AUTO_RUN_SPEED + 20.0);
        assert!(world.run_speed() <= MAX_RUN_SPEED + 0.01);
        let uncapped = AUTO_RUN_SPEED + world.run_time() * SPEED_RAMP_PER_SEC;
        assert!(uncapped > MAX_RUN_SPEED);
        assert!((world.run_speed() - MAX_RUN_SPEED).abs() < 0.01);
    }

    #[test]
    fn endless_generates_beyond_initial_buffer() {
        let mut world = World::new_seeded(42);
        let initial = world.segments().len();
        let initial_tip = world.course_end();
        assert!(world.is_endless());
        let target = initial_tip + 100.0;
        if let Some(t) = world.transforms.get_mut(world.player_id) {
            t.x = target;
        }
        world.ensure_segments_ahead();
        world.sync_streamer();
        assert!(
            world.segments().len() > initial,
            "expected more than {initial} segments"
        );
        assert!(
            world.course_end() >= target + GENERATE_AHEAD_PX - 1.0,
            "tip {} should stay ahead of player {}",
            world.course_end(),
            target
        );
        assert!(crate::validate::validate_continuity(world.segments()).is_ok());
        // Platforms must be live near the player (not just definitions).
        let has_ground_near = world.platform_aabbs().iter().any(|p| {
            (p.y - GROUND_Y).abs() < 3.0 && p.x <= target + 50.0 && p.x + p.w >= target - 50.0
        });
        assert!(has_ground_near, "expected spawned ground near player");
    }

    #[test]
    fn endless_does_not_finish() {
        let mut world = World::new_seeded(42);
        let target = world.course_end() + 500.0;
        if let Some(t) = world.transforms.get_mut(world.player_id) {
            t.x = target;
        }
        world.players.get_mut(world.player_id).unwrap().finished = false;
        let input = InputSnapshot::default();
        world.tick(input, 1.0 / 60.0);
        assert!(!world.run_state().finished);
    }

    #[test]
    fn coin_pickup_increments_score() {
        let mut world = World::new_seeded(42);
        land(&mut world);
        let pid = world.player_id;
        let (px, py) = {
            let t = world.transforms.get(pid).unwrap();
            (t.x, t.y)
        };
        world.spawn_coin(px, py);
        let before = world.players.get(pid).unwrap().score_coins;
        world.resolve_hazards_and_coins();
        let after = world.players.get(pid).unwrap().score_coins;
        assert!(after > before);
        assert!(world.run_state().score >= COIN_VALUE);
    }

    #[test]
    fn solid_hazard_kills_unless_dashing() {
        let mut world = World::new();
        land(&mut world);
        let pid = world.player_id;
        let (px, py) = {
            let t = world.transforms.get(pid).unwrap();
            (t.x, t.y)
        };
        world.spawn_hazard(px, py, 40.0, 40.0, HazardKind::SolidKill, (1.0, 0.0, 0.0));
        world.resolve_hazards_and_coins();
        assert_eq!(world.players.get(pid).unwrap().state, PlayerState::Dead);

        let mut world = World::new();
        land(&mut world);
        let pid = world.player_id;
        let mut input = InputSnapshot::default();
        input.dash_pressed = true;
        world.tick(input, 1.0 / 60.0);
        let (px, py) = {
            let t = world.transforms.get(pid).unwrap();
            (t.x, t.y)
        };
        world.spawn_hazard(px, py, 40.0, 40.0, HazardKind::SolidKill, (1.0, 0.0, 0.0));
        world.resolve_hazards_and_coins();
        assert_ne!(world.players.get(pid).unwrap().state, PlayerState::Dead);
    }

    #[test]
    fn reset_clears_score() {
        let mut world = World::new();
        if let Some(p) = world.players.get_mut(world.player_id) {
            p.score_coins = 5;
            p.max_x = 500.0;
        }
        world.reset();
        let rs = world.run_state();
        assert_eq!(rs.coins, 0);
        assert!(rs.distance < 100.0);
    }

    #[test]
    fn finish_when_past_course_end_finite_only() {
        // Headless / finite buffer worlds can still mark finish.
        let course = director::generate_course_geometry(42);
        let mut world = World::from_course_fully_spawned(&course);
        assert!(!world.endless);
        let end = world.course_end();
        if let Some(t) = world.transforms.get_mut(world.player_id) {
            t.x = end + 10.0;
        }
        world.check_finish();
        assert!(world.players.get(world.player_id).unwrap().finished);
        assert!(world.run_state().finished);
    }

    #[test]
    fn streamer_despawns_behind() {
        let mut world = World::new_seeded(42);
        assert!(world.streaming_enabled);
        let initial = world.entity_count_approx();
        let target_x = world.course_end() * 0.7;
        // Advance far into the course so early segments despawn.
        if let Some(t) = world.transforms.get_mut(world.player_id) {
            t.x = target_x;
        }
        for _ in 0..5 {
            world.sync_streamer();
        }
        let after = world.entity_count_approx();
        // Windowed spawn should stay bounded relative to full course.
        let full = World::from_course_fully_spawned(&director::generate_course_geometry(42));
        assert!(
            after <= full.entity_count_approx(),
            "streamed={after} full={} initial={initial}",
            full.entity_count_approx()
        );
        assert!(world.streamer.active_count() <= STREAM_AHEAD + STREAM_BEHIND + 12);
    }

    #[test]
    fn endless_stream_entity_budget_stays_bounded() {
        let mut world = World::new_seeded(42);
        let mut peak = 0usize;
        for step in 0..2000 {
            if let Some(t) = world.transforms.get_mut(world.player_id) {
                t.x = 40.0 + step as f32 * 8.0;
                t.y = GROUND_Y - PLAYER_H - 2.0;
            }
            world.ensure_segments_ahead();
            world.sync_streamer();
            peak = peak.max(world.streamer.active_count());
            peak = peak.max(world.entity_count_approx() / 4); // rough live budget
        }
        // Live segments must not grow with course length (~2000 steps of defs).
        assert!(
            world.streamer.active_count() <= STREAM_AHEAD + STREAM_BEHIND + 12,
            "active={} course={}",
            world.streamer.active_count(),
            world.segments().len()
        );
        assert!(world.segments().len() > 40);
        assert!(peak > 0);
    }

    #[test]
    fn endless_long_run_never_runs_out_of_ground_defs() {
        let mut world = World::new_seeded(42);
        let input = InputSnapshot::default();
        let mut max_x = 0.0_f32;
        // Simulate ~45s of auto-run without requiring the player to clear hazards:
        // keep them glued to generated ground while streaming advances.
        for _ in 0..2700 {
            world.ensure_segments_ahead();
            world.sync_streamer();
            let px = world
                .transforms
                .get(world.player_id)
                .map(|t| t.x)
                .unwrap_or(0.0);
            let tip = world
                .segments()
                .last()
                .map(|s| s.world_x1())
                .unwrap_or(0.0);
            assert!(
                tip >= px + GENERATE_AHEAD_PX - 1.0,
                "tip {tip} lagged player {px}"
            );
            // Ground coverage in definitions for the next screen.
            let probe = px + 80.0;
            let in_shaft_pit = world.segments().iter().any(|s| {
                s.shaft
                    .map(|sh| probe >= sh.pit_x0 && probe <= sh.pit_x1)
                    .unwrap_or(false)
            });
            if !in_shaft_pit {
                let covered = world.segments().iter().any(|s| {
                    s.platforms.iter().any(|p| {
                        (p.y - GROUND_Y).abs() < 3.0 && p.x <= probe && p.x + p.w >= probe
                    })
                });
                // Allow intentional gaps: if not covered, a platform must exist within MAX_SAFE_GAP.
                if !covered {
                    let nearest = world
                        .segments()
                        .iter()
                        .flat_map(|s| s.platforms.iter())
                        .filter(|p| {
                            (p.y - GROUND_Y).abs() < 3.0 && p.x + p.w >= probe - MAX_SAFE_GAP
                        })
                        .map(|p| p.x)
                        .filter(|&x| x >= probe)
                        .fold(f32::MAX, f32::min);
                    assert!(
                        nearest - probe <= MAX_SAFE_GAP + 1.0,
                        "void at {probe}: next ground at {nearest}"
                    );
                }
            }
            // Step player forward at current run speed (ignore collisions for this stress test).
            let spd = world.run_speed();
            if let Some(t) = world.transforms.get_mut(world.player_id) {
                t.x += spd / 60.0;
                t.y = GROUND_Y - PLAYER_H - 2.0;
                max_x = t.x.max(max_x);
            }
            world.update_run_speed(1.0 / 60.0);
            let _ = input;
        }
        assert!(max_x > 8000.0, "expected long travel, got {max_x}");
        assert!(world.segments().len() > SEGMENT_COUNT + 10);
    }

    #[test]
    fn reset_keeps_seed_when_locked() {
        let mut world = World::new_seeded(77);
        assert!(world.seed_locked());
        assert_eq!(world.seed(), 77);
        world.reset();
        assert_eq!(world.seed(), 77);
    }

    #[test]
    fn reset_rerolls_seed_when_unlocked() {
        let mut world = World::new();
        assert!(!world.seed_locked());
        let first = world.seed();
        // Force a different clock sample.
        std::thread::sleep(std::time::Duration::from_millis(2));
        world.reset();
        let second = world.seed();
        assert_ne!(first, second, "unlocked reset should roll a new seed");
        assert!(world.segments().len() >= 2);
        assert!(crate::validate::validate_continuity(world.segments()).is_ok());
    }

    #[test]
    fn jump_still_works() {
        let mut world = World::new_seeded(42);
        land(&mut world);
        let mut jump = InputSnapshot::default();
        jump.jump_pressed = true;
        jump.jump_held = true;
        world.tick(jump, 1.0 / 60.0);
        jump.jump_pressed = false;
        for _ in 0..5 {
            world.tick(jump, 1.0 / 60.0);
        }
        assert!(world.velocities.get(world.player_id).unwrap().y < 0.0);
    }

    #[test]
    fn magnet_attracts_and_collects_coin() {
        let mut world = World::new_seeded(42);
        land(&mut world);
        let pid = world.player_id;
        let (px, py) = {
            let t = world.transforms.get(pid).unwrap();
            (t.x, t.y)
        };
        world.abilities.activate(AbilityId::Magnet);
        let coin = world.spawn_coin(px + 80.0, py);
        let x0 = world.transforms.get(coin).unwrap().x;
        let input = InputSnapshot::default();
        for _ in 0..45 {
            world.tick(input, 1.0 / 60.0);
        }
        // Either collected or moved significantly toward the player.
        let still = world.entities.is_alive(coin);
        if still {
            let x1 = world.transforms.get(coin).unwrap().x;
            assert!(x1 < x0 - 5.0, "magnet should pull coin {x0} -> {x1}");
        } else {
            assert!(world.players.get(pid).unwrap().score_coins >= 1);
        }
    }

    #[test]
    fn magnet_off_does_not_pull() {
        let mut world = World::new_seeded(42);
        land(&mut world);
        let pid = world.player_id;
        let (px, py) = {
            let t = world.transforms.get(pid).unwrap();
            (t.x, t.y)
        };
        let coin = world.spawn_coin(px + 90.0, py);
        let x0 = world.transforms.get(coin).unwrap().x;
        // Hold player still-ish by resetting x each frame after tick... just tick few frames.
        let input = InputSnapshot::default();
        world.tick(input, 1.0 / 60.0);
        assert!(world.entities.is_alive(coin));
        let x1 = world.transforms.get(coin).unwrap().x;
        assert!((x1 - x0).abs() < 0.01);
    }

    #[test]
    fn slow_fall_reduces_fall_speed() {
        let mut world = World::new_seeded(42);
        land(&mut world);
        let pid = world.player_id;
        if let Some(t) = world.transforms.get_mut(pid) {
            t.y = GROUND_Y - PLAYER_H - 200.0;
        }
        if let Some(p) = world.players.get_mut(pid) {
            p.grounded = false;
            p.state = PlayerState::Falling;
        }
        world.abilities.activate(AbilityId::SlowFall);
        let input = InputSnapshot::default();
        world.tick(input, 1.0 / 60.0);
        let vy_slow = world.velocities.get(pid).unwrap().y;
        assert!(vy_slow > 0.0);
        assert!(vy_slow < MAX_FALL_SPEED * SLOW_FALL_MAX_FALL_SCALE + 1.0);

        // Rolling ignores slow-fall cap.
        world.begin_roll(pid);
        world.tick(input, 1.0 / 60.0);
        let vy_roll = world.velocities.get(pid).unwrap().y;
        assert!(vy_roll >= vy_slow - 0.1);
    }

    #[test]
    fn ghost_passes_ghost_passable_hazard() {
        let mut world = World::new_seeded(42);
        land(&mut world);
        let pid = world.player_id;
        let (px, py) = {
            let t = world.transforms.get(pid).unwrap();
            (t.x, t.y)
        };
        world.spawn_hazard_full(
            px,
            py,
            40.0,
            40.0,
            HazardKind::SolidKill,
            (0.5, 0.4, 0.8),
            CollisionCategory::GhostPassable,
            None,
            None,
        );
        world.abilities.activate(AbilityId::Ghost);
        world.resolve_hazards_and_coins();
        assert_ne!(
            world.players.get(pid).unwrap().state,
            PlayerState::Dead,
            "ghost should ignore GhostPassable"
        );
    }

    #[test]
    fn ghost_passable_kills_without_ghost() {
        let mut world = World::new_seeded(42);
        land(&mut world);
        let pid = world.player_id;
        let (px, py) = {
            let t = world.transforms.get(pid).unwrap();
            (t.x, t.y)
        };
        world.spawn_hazard_full(
            px,
            py,
            40.0,
            40.0,
            HazardKind::SolidKill,
            (0.5, 0.4, 0.8),
            CollisionCategory::GhostPassable,
            None,
            None,
        );
        world.resolve_hazards_and_coins();
        assert_eq!(world.players.get(pid).unwrap().state, PlayerState::Dead);
    }

    #[test]
    fn always_solid_kills_even_with_ghost() {
        let mut world = World::new_seeded(42);
        land(&mut world);
        let pid = world.player_id;
        let (px, py) = {
            let t = world.transforms.get(pid).unwrap();
            (t.x, t.y)
        };
        world.abilities.activate(AbilityId::Ghost);
        world.spawn_hazard_full(
            px,
            py,
            40.0,
            40.0,
            HazardKind::SolidKill,
            (0.9, 0.2, 0.2),
            CollisionCategory::AlwaysSolid,
            None,
            None,
        );
        world.resolve_hazards_and_coins();
        assert_eq!(world.players.get(pid).unwrap().state, PlayerState::Dead);
    }

    #[test]
    fn strength_qte_ratings_map_to_durations() {
        use crate::config::{
            STRENGTH_DURATION_GOOD, STRENGTH_DURATION_GREAT, STRENGTH_DURATION_PERFECT,
            STRENGTH_DURATION_POOR,
        };
        use crate::qte::QteRating;
        let mut m = AbilityManager::default();
        m.activate_strength_from_rating(QteRating::Poor);
        assert!((m.remaining(AbilityId::SuperStrength) - STRENGTH_DURATION_POOR).abs() < 0.01);
        m.activate_strength_from_rating(QteRating::Good);
        assert!((m.remaining(AbilityId::SuperStrength) - STRENGTH_DURATION_GOOD).abs() < 0.01);
        m.activate_strength_from_rating(QteRating::Great);
        assert!((m.remaining(AbilityId::SuperStrength) - STRENGTH_DURATION_GREAT).abs() < 0.01);
        m.activate_strength_from_rating(QteRating::Perfect);
        assert!((m.remaining(AbilityId::SuperStrength) - STRENGTH_DURATION_PERFECT).abs() < 0.01);
    }

    #[test]
    fn strength_destroys_destructible_with_reward() {
        let mut world = World::new_seeded(42);
        land(&mut world);
        let pid = world.player_id;
        let (px, py) = {
            let t = world.transforms.get(pid).unwrap();
            (t.x, t.y)
        };
        let _hid = world.spawn_hazard_full(
            px,
            py,
            40.0,
            40.0,
            HazardKind::SolidKill,
            (1.0, 0.3, 0.1),
            CollisionCategory::AlwaysSolid,
            Some(DestructibleSpec {
                health: 10.0,
                reward_coins: 3,
                material: 1,
            }),
            None,
        );
        // Without strength: lethal.
        world.resolve_hazards_and_coins();
        assert_eq!(world.players.get(pid).unwrap().state, PlayerState::Dead);

        let mut world = World::new_seeded(42);
        land(&mut world);
        let pid = world.player_id;
        let (px, py) = {
            let t = world.transforms.get(pid).unwrap();
            (t.x, t.y)
        };
        let hid = world.spawn_hazard_full(
            px,
            py,
            40.0,
            40.0,
            HazardKind::SolidKill,
            (1.0, 0.3, 0.1),
            CollisionCategory::AlwaysSolid,
            Some(DestructibleSpec {
                health: 10.0,
                reward_coins: 3,
                material: 1,
            }),
            None,
        );
        world
            .abilities
            .refresh(AbilityId::SuperStrength, 5.0);
        let before = world.players.get(pid).unwrap().score_coins;
        world.apply_super_strength_damage();
        assert!(!world.entities.is_alive(hid));
        assert!(world.players.get(pid).unwrap().score_coins >= before + 3);
        // Invulnerable to leftover hazards path.
        world.resolve_hazards_and_coins();
        assert_ne!(world.players.get(pid).unwrap().state, PlayerState::Dead);
    }

    #[test]
    fn ability_expiry_and_reset_clear() {
        let mut world = World::new_seeded(42);
        world.abilities.activate(AbilityId::Magnet);
        world.abilities.refresh(AbilityId::Ghost, MAGNET_DURATION + 2.0);
        assert!(world.abilities.is_active(AbilityId::Magnet));
        world.abilities.tick(MAGNET_DURATION + 0.1);
        assert!(!world.abilities.is_active(AbilityId::Magnet));
        assert!(world.abilities.is_active(AbilityId::Ghost));
        world.reset();
        assert_eq!(world.abilities.active_mask(), 0);
        assert!(!world.abilities.qte_active());
    }

    #[test]
    fn effect_pool_capped_under_spam() {
        let mut world = World::new_seeded(42);
        for _ in 0..500 {
            world.effects.spawn_burst(0.0, 0.0, 1.0, 1.0, 1.0, 12);
        }
        assert!(world.effects.live_count() <= EFFECT_POOL_CAP);
    }

    #[test]
    fn powerup_pickup_activates_ability() {
        let mut world = World::new_seeded(42);
        land(&mut world);
        let pid = world.player_id;
        let (px, py) = {
            let t = world.transforms.get(pid).unwrap();
            (t.x, t.y)
        };
        world.spawn_powerup(PowerupKind::SlowFall, px, py);
        let input = InputSnapshot::default();
        world.tick(input, 1.0 / 60.0);
        assert!(world.abilities.is_active(AbilityId::SlowFall));
    }

    fn force_falling_shaft_world() -> World {
        use crate::generate::generate_segment;
        use crate::rng::XorShift64;
        let mut rng = XorShift64::from_seed(99);
        let shaft = generate_segment(&mut rng, SegmentIntent::FallingShaft, 0.0);
        assert!(shaft.shaft.is_some());
        let course = GeneratedCourse {
            seed: 99,
            generator_version: GENERATOR_VERSION,
            course_end: shaft.world_x1() - 40.0,
            segments: vec![shaft],
        };
        World::from_course_fully_spawned(&course)
    }

    #[test]
    fn falling_mode_strafe_moves_x() {
        let mut world = force_falling_shaft_world();
        // Clear air hazards so the strafe test is isolated.
        let hz: Vec<_> = world.hazards.iter().map(|(id, _)| id).collect();
        for id in hz {
            world.despawn_entity(id);
        }
        let shaft = world.segments()[0].shaft.unwrap();
        let pid = world.player_id;
        if let Some(t) = world.transforms.get_mut(pid) {
            t.x = (shaft.pit_x0 + shaft.pit_x1) * 0.5 - PLAYER_W * 0.5;
            t.y = shaft.enter_y + 60.0;
        }
        if let Some(p) = world.players.get_mut(pid) {
            p.grounded = false;
            p.state = PlayerState::Falling;
        }
        world.try_begin_falling();
        assert!(world.run_mode().is_falling_like());
        // Advance past FallingEnter.
        let mut input = InputSnapshot::default();
        for _ in 0..10 {
            world.tick(input, 1.0 / 60.0);
        }
        assert_eq!(world.run_mode(), RunMode::Falling);
        let x0 = world.transforms.get(pid).unwrap().x;
        input.move_x = -1.0;
        for _ in 0..20 {
            world.tick(input, 1.0 / 60.0);
        }
        let x1 = world.transforms.get(pid).unwrap().x;
        assert!(x1 < x0 - 5.0, "strafe left expected {x0} -> {x1}");
        assert!(world.players.get(pid).unwrap().state != PlayerState::Dead);
    }

    #[test]
    fn falling_mode_no_autorun() {
        let mut world = force_falling_shaft_world();
        let hz: Vec<_> = world.hazards.iter().map(|(id, _)| id).collect();
        for id in hz {
            world.despawn_entity(id);
        }
        let shaft = world.segments()[0].shaft.unwrap();
        let pid = world.player_id;
        if let Some(t) = world.transforms.get_mut(pid) {
            t.x = (shaft.pit_x0 + shaft.pit_x1) * 0.5 - PLAYER_W * 0.5;
            t.y = shaft.enter_y + 40.0;
        }
        if let Some(p) = world.players.get_mut(pid) {
            p.grounded = false;
        }
        world.try_begin_falling();
        let input = InputSnapshot::default();
        for _ in 0..12 {
            world.tick(input, 1.0 / 60.0);
        }
        assert_eq!(world.run_mode(), RunMode::Falling);
        // Direct velocity check: no auto-run X while Falling with move_x=0.
        world.apply_player_control(input, 1.0 / 60.0);
        let vx = world.velocities.get(pid).unwrap().x;
        assert!(
            vx.abs() < 1.0,
            "expected zero horizontal without strafe, got {vx}"
        );
    }

    #[test]
    fn falling_camera_looks_down() {
        let mut world = force_falling_shaft_world();
        let shaft = world.segments()[0].shaft.unwrap();
        let pid = world.player_id;
        if let Some(t) = world.transforms.get_mut(pid) {
            t.x = (shaft.pit_x0 + shaft.pit_x1) * 0.5;
            t.y = shaft.enter_y + 80.0;
        }
        if let Some(p) = world.players.get_mut(pid) {
            p.grounded = false;
        }
        world.try_begin_falling();
        let input = InputSnapshot::default();
        for _ in 0..20 {
            world.tick(input, 1.0 / 60.0);
        }
        let py = world.transforms.get(pid).unwrap().y;
        let cam_y = world.camera().y;
        assert!(
            cam_y > py,
            "camera should look ahead downward: cam={cam_y} player={py}"
        );
        assert_eq!(world.run_state().falling, 1);
    }

    #[test]
    fn falling_land_exits_to_run() {
        let mut world = force_falling_shaft_world();
        let shaft = world.segments()[0].shaft.unwrap();
        let pid = world.player_id;
        // Place on floor pad already grounded.
        if let Some(t) = world.transforms.get_mut(pid) {
            t.x = (shaft.pit_x0 + shaft.pit_x1) * 0.5 - PLAYER_W * 0.5;
            t.y = shaft.floor_y - PLAYER_H - 2.0;
        }
        world.falling = Some(crate::falling::FallingState::new(
            shaft.pit_x0,
            shaft.pit_x1,
            shaft.floor_y,
            shaft.enter_y,
            shaft.exit_x,
            shaft.exit_y,
        ));
        world.run_mode = RunMode::Falling;
        if let Some(p) = world.players.get_mut(pid) {
            p.grounded = true;
            p.state = PlayerState::Running;
        }
        let input = InputSnapshot::default();
        world.check_falling_transitions();
        assert_eq!(world.run_mode(), RunMode::FallingExit);
        for _ in 0..40 {
            world.tick(input, 1.0 / 60.0);
        }
        assert_eq!(world.run_mode(), RunMode::Run);
        assert_eq!(world.run_state().falling, 0);
        // Auto-run restored.
        let x0 = world.transforms.get(pid).unwrap().x;
        for _ in 0..30 {
            world.tick(input, 1.0 / 60.0);
        }
        let x1 = world.transforms.get(pid).unwrap().x;
        assert!(x1 > x0 + 20.0);
    }

    #[test]
    fn falling_timeout_forces_exit() {
        let mut world = force_falling_shaft_world();
        let shaft = world.segments()[0].shaft.unwrap();
        let pid = world.player_id;
        if let Some(t) = world.transforms.get_mut(pid) {
            t.x = (shaft.pit_x0 + shaft.pit_x1) * 0.5;
            t.y = shaft.enter_y + 100.0;
        }
        world.falling = Some(crate::falling::FallingState::new(
            shaft.pit_x0,
            shaft.pit_x1,
            shaft.floor_y,
            shaft.enter_y,
            shaft.exit_x,
            shaft.exit_y,
        ));
        world.run_mode = RunMode::Falling;
        if let Some(f) = world.falling.as_mut() {
            f.mode_timer = FALLING_MAX_DURATION + 0.1;
        }
        if let Some(p) = world.players.get_mut(pid) {
            p.grounded = false;
        }
        world.check_falling_transitions();
        assert_eq!(world.run_mode(), RunMode::FallingExit);
    }

    #[test]
    fn falling_shaft_segment_validates() {
        use crate::generate::generate_segment;
        use crate::rng::XorShift64;
        let mut rng = XorShift64::from_seed(7);
        for _ in 0..12 {
            let seg = generate_segment(&mut rng, SegmentIntent::FallingShaft, 100.0);
            assert!(crate::validate::validate_segment(&seg).is_ok());
            assert!(seg.shaft.is_some());
        }
    }

    #[test]
    fn endless_can_append_falling_shaft() {
        let mut world = World::new_seeded(12345);
        let mut saw = false;
        for _ in 0..80 {
            world.append_segment();
            if world
                .segments()
                .iter()
                .any(|s| s.intent == SegmentIntent::FallingShaft)
            {
                saw = true;
                break;
            }
        }
        assert!(saw, "expected FallingShaft in endless append");
    }
}
