//! Phase 10 microbenches — run with:
//! `cargo bench -p runtime-core --bench sim_bench`

use runtime_core::{InputSnapshot, World};
use std::time::Instant;

fn bench_ticks(label: &str, ticks: u32, seed: u64) {
    let mut world = World::new_seeded(seed);
    world.set_viewport(432.0, 768.0);
    world.clear_hazards_and_powerups();
    world.reset_profile();
    let input = InputSnapshot::default();
    let t0 = Instant::now();
    for i in 0..ticks {
        world.tick(input, 1.0 / 60.0);
        if i % 30 == 0 {
            world.clear_hazards_and_powerups();
        }
    }
    let wall_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let p = world.take_profile();
    let ticks_f = p.ticks.max(1) as f64;
    let total = p.total_ns().max(1) as f64;
    println!(
        "{label}: wall={wall_ms:.3}ms ticks={} avg_tick={:.3}µs | control={:.1}% integrate={:.1}% collision={:.1}% hazards={:.1}% streamer={:.1}% other={:.1}%",
        p.ticks,
        (wall_ms * 1000.0) / ticks_f,
        100.0 * p.control_ns as f64 / total,
        100.0 * p.integrate_ns as f64 / total,
        100.0 * p.collision_ns as f64 / total,
        100.0 * p.hazards_ns as f64 / total,
        100.0 * p.streamer_ns as f64 / total,
        100.0 * p.other_ns as f64 / total,
    );
}

fn bench_extract(label: &str, ticks: u32, seed: u64) {
    let mut world = World::new_seeded(seed);
    world.set_viewport(432.0, 768.0);
    world.clear_hazards_and_powerups();
    let input = InputSnapshot::default();
    for _ in 0..120 {
        world.tick(input, 1.0 / 60.0);
    }
    world.reset_profile();
    let mut buf = Vec::with_capacity(512);
    let t0 = Instant::now();
    for _ in 0..ticks {
        world.extract_sprites(&mut buf);
    }
    let wall_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let p = world.take_profile();
    println!(
        "{label}: wall={wall_ms:.3}ms extracts={ticks} sprites~{} extract_ns_total={} avg={:.3}µs",
        buf.len(),
        p.extract_ns,
        (p.extract_ns as f64 / ticks.max(1) as f64) / 1000.0,
    );
}

fn main() {
    bench_ticks("warmup", 60, 42);
    bench_ticks("tick_seed42_3k", 3000, 42);
    bench_ticks("tick_seed7_3k", 3000, 7);
    bench_extract("extract_2k", 2000, 42);
}
