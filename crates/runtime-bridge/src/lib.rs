//! C ABI implementation for the Vertical Shift C++ host.
//!
//! Panics are caught at the FFI edge so they never unwind into C++.

use runtime_core::{
    CameraState, ExtractSprite, GameEvent, InputSnapshot, QualityProfile, World,
};
use std::os::raw::c_char;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

static INITIALIZED: AtomicBool = AtomicBool::new(false);
static TICK: AtomicU64 = AtomicU64::new(0);
static WORLD: Mutex<Option<World>> = Mutex::new(None);

const VS_OK: i32 = 0;
const VS_ERR_FAIL: i32 = 1;
const VS_ERR_INVALID_ARG: i32 = 2;
const VS_ERR_NOT_INITIALIZED: i32 = 3;
const VS_ERR_ALREADY_INITIALIZED: i32 = 4;
const VS_ERR_NO_WORLD: i32 = 5;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VsInputState {
    pub move_x: f32,
    pub jump_pressed: u32,
    pub jump_held: u32,
    pub dash_pressed: u32,
    pub roll_held: u32,
    pub restart_pressed: u32,
    pub qte_pressed: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct VsRunState {
    pub alive: u32,
    pub finished: u32,
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VsSpriteInstance {
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VsCamera2D {
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
    pub viewport_w: f32,
    pub viewport_h: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VsGameEvent {
    pub kind: u32,
    pub x: f32,
    pub y: f32,
    pub data: u32,
}

fn catch_vs<F: FnOnce() -> i32 + std::panic::UnwindSafe>(f: F) -> i32 {
    match std::panic::catch_unwind(f) {
        Ok(code) => code,
        Err(_) => VS_ERR_FAIL,
    }
}

#[no_mangle]
pub extern "C" fn vs_runtime_version() -> *const c_char {
    runtime_core::RUNTIME_VERSION_C.as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn vs_abi_version_major() -> u32 {
    runtime_core::ABI_VERSION_MAJOR
}

#[no_mangle]
pub extern "C" fn vs_abi_version_minor() -> u32 {
    runtime_core::ABI_VERSION_MINOR
}

#[no_mangle]
pub extern "C" fn vs_runtime_init() -> i32 {
    catch_vs(|| {
        if INITIALIZED.swap(true, Ordering::SeqCst) {
            return VS_ERR_ALREADY_INITIALIZED;
        }
        TICK.store(0, Ordering::SeqCst);
        VS_OK
    })
}

#[no_mangle]
pub extern "C" fn vs_runtime_tick() -> u64 {
    if !INITIALIZED.load(Ordering::SeqCst) {
        return 0;
    }
    TICK.fetch_add(1, Ordering::SeqCst) + 1
}

#[no_mangle]
pub extern "C" fn vs_runtime_shutdown() -> i32 {
    catch_vs(|| {
        if let Ok(mut guard) = WORLD.lock() {
            *guard = None;
        }
        if !INITIALIZED.swap(false, Ordering::SeqCst) {
            return VS_OK;
        }
        TICK.store(0, Ordering::SeqCst);
        let _ = VS_ERR_NOT_INITIALIZED;
        VS_OK
    })
}

#[no_mangle]
pub extern "C" fn vs_world_create() -> i32 {
    catch_vs(|| {
        if !INITIALIZED.load(Ordering::SeqCst) {
            return VS_ERR_NOT_INITIALIZED;
        }
        match WORLD.lock() {
            Ok(mut guard) => {
                // Unlocked random course; reset rolls a new seed each time.
                *guard = Some(World::new());
                VS_OK
            }
            Err(_) => VS_ERR_FAIL,
        }
    })
}

#[no_mangle]
pub extern "C" fn vs_world_create_seeded(seed: u64) -> i32 {
    catch_vs(move || {
        if !INITIALIZED.load(Ordering::SeqCst) {
            return VS_ERR_NOT_INITIALIZED;
        }
        match WORLD.lock() {
            Ok(mut guard) => {
                // Locked seed for `--seed` / reproducible benches.
                *guard = Some(World::new_seeded(seed));
                VS_OK
            }
            Err(_) => VS_ERR_FAIL,
        }
    })
}

#[no_mangle]
pub extern "C" fn vs_world_destroy() -> i32 {
    catch_vs(|| {
        match WORLD.lock() {
            Ok(mut guard) => {
                *guard = None;
                VS_OK
            }
            Err(_) => VS_ERR_FAIL,
        }
    })
}

#[no_mangle]
pub extern "C" fn vs_world_reset() -> i32 {
    catch_vs(|| {
        match WORLD.lock() {
            Ok(mut guard) => {
                if let Some(world) = guard.as_mut() {
                    world.reset();
                    VS_OK
                } else {
                    VS_ERR_NO_WORLD
                }
            }
            Err(_) => VS_ERR_FAIL,
        }
    })
}

#[no_mangle]
pub extern "C" fn vs_world_set_viewport(width: f32, height: f32) -> i32 {
    catch_vs(|| {
        match WORLD.lock() {
            Ok(mut guard) => {
                if let Some(world) = guard.as_mut() {
                    world.set_viewport(width, height);
                    VS_OK
                } else {
                    VS_ERR_NO_WORLD
                }
            }
            Err(_) => VS_ERR_FAIL,
        }
    })
}

#[no_mangle]
pub extern "C" fn vs_world_tick(input: *const VsInputState, dt: f32) -> i32 {
    catch_vs(move || {
        if input.is_null() {
            return VS_ERR_INVALID_ARG;
        }
        let snap = unsafe {
            let i = &*input;
            InputSnapshot {
                move_x: i.move_x,
                jump_pressed: i.jump_pressed != 0,
                jump_held: i.jump_held != 0,
                dash_pressed: i.dash_pressed != 0,
                roll_held: i.roll_held != 0,
                restart_pressed: i.restart_pressed != 0,
                qte_pressed: i.qte_pressed != 0,
            }
        };
        match WORLD.lock() {
            Ok(mut guard) => {
                if let Some(world) = guard.as_mut() {
                    world.tick(snap, dt);
                    TICK.fetch_add(1, Ordering::SeqCst);
                    VS_OK
                } else {
                    VS_ERR_NO_WORLD
                }
            }
            Err(_) => VS_ERR_FAIL,
        }
    })
}

#[no_mangle]
pub extern "C" fn vs_world_extract(
    out: *mut VsSpriteInstance,
    cap: u32,
    out_count: *mut u32,
    camera: *mut VsCamera2D,
) -> i32 {
    catch_vs(move || {
        if out.is_null() || out_count.is_null() || camera.is_null() {
            return VS_ERR_INVALID_ARG;
        }
        match WORLD.lock() {
            Ok(mut guard) => {
                let Some(world) = guard.as_mut() else {
                    return VS_ERR_NO_WORLD;
                };
                let mut sprites: Vec<ExtractSprite> = Vec::new();
                world.extract_sprites(&mut sprites);
                let n = sprites.len().min(cap as usize) as u32;
                unsafe {
                    for i in 0..n as usize {
                        let s = &sprites[i];
                        *out.add(i) = VsSpriteInstance {
                            x: s.x,
                            y: s.y,
                            w: s.w,
                            h: s.h,
                            u0: s.u0,
                            v0: s.v0,
                            u1: s.u1,
                            v1: s.v1,
                            r: s.r,
                            g: s.g,
                            b: s.b,
                            a: s.a,
                            texture_id: s.texture_id,
                        };
                    }
                    *out_count = n;
                    let cam: CameraState = world.camera();
                    *camera = VsCamera2D {
                        x: cam.x,
                        y: cam.y,
                        zoom: cam.zoom,
                        viewport_w: cam.viewport_w,
                        viewport_h: cam.viewport_h,
                    };
                }
                VS_OK
            }
            Err(_) => VS_ERR_FAIL,
        }
    })
}

#[no_mangle]
pub extern "C" fn vs_world_get_run_state(out: *mut VsRunState) -> i32 {
    catch_vs(move || {
        if out.is_null() {
            return VS_ERR_INVALID_ARG;
        }
        match WORLD.lock() {
            Ok(guard) => {
                let Some(world) = guard.as_ref() else {
                    return VS_ERR_NO_WORLD;
                };
                let rs = world.run_state();
                unsafe {
                    *out = VsRunState {
                        alive: if rs.alive { 1 } else { 0 },
                        finished: if rs.finished { 1 } else { 0 },
                        score: rs.score,
                        coins: rs.coins,
                        distance: rs.distance,
                        player_state: rs.player_state,
                        seed: rs.seed,
                        segment_index: rs.segment_index,
                        segment_count: rs.segment_count,
                        ability_mask: rs.ability_mask,
                        qte_active: rs.qte_active,
                        qte_prompt: rs.qte_prompt,
                        falling: rs.falling,
                        falling_time: rs.falling_time,
                    };
                }
                VS_OK
            }
            Err(_) => VS_ERR_FAIL,
        }
    })
}

#[no_mangle]
pub extern "C" fn vs_world_drain_events(
    out: *mut VsGameEvent,
    cap: u32,
    count: *mut u32,
) -> i32 {
    catch_vs(move || {
        if out.is_null() || count.is_null() {
            return VS_ERR_INVALID_ARG;
        }
        match WORLD.lock() {
            Ok(mut guard) => {
                let Some(world) = guard.as_mut() else {
                    return VS_ERR_NO_WORLD;
                };
                let mut tmp = vec![
                    GameEvent {
                        kind: runtime_core::GameEventKind::Jump,
                        x: 0.0,
                        y: 0.0,
                        data: 0,
                    };
                    cap as usize
                ];
                let n = world.drain_events(&mut tmp);
                unsafe {
                    for i in 0..n {
                        let e = &tmp[i];
                        *out.add(i) = VsGameEvent {
                            kind: e.kind as u32,
                            x: e.x,
                            y: e.y,
                            data: e.data,
                        };
                    }
                    *count = n as u32;
                }
                VS_OK
            }
            Err(_) => VS_ERR_FAIL,
        }
    })
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VsDebugAabb {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[no_mangle]
pub extern "C" fn vs_world_set_quality(profile: u32) -> i32 {
    catch_vs(move || match WORLD.lock() {
        Ok(mut guard) => {
            if let Some(world) = guard.as_mut() {
                world.set_quality(QualityProfile::from_u32(profile));
                VS_OK
            } else {
                VS_ERR_NO_WORLD
            }
        }
        Err(_) => VS_ERR_FAIL,
    })
}

#[no_mangle]
pub extern "C" fn vs_world_extract_debug_aabbs(
    out: *mut VsDebugAabb,
    cap: u32,
    count: *mut u32,
) -> i32 {
    catch_vs(move || {
        if out.is_null() || count.is_null() {
            return VS_ERR_INVALID_ARG;
        }
        match WORLD.lock() {
            Ok(guard) => {
                let Some(world) = guard.as_ref() else {
                    return VS_ERR_NO_WORLD;
                };
                let mut tmp = Vec::new();
                world.extract_debug_aabbs(&mut tmp);
                let n = tmp.len().min(cap as usize) as u32;
                unsafe {
                    for i in 0..n as usize {
                        let (x, y, w, h) = tmp[i];
                        *out.add(i) = VsDebugAabb { x, y, w, h };
                    }
                    *count = n;
                }
                VS_OK
            }
            Err(_) => VS_ERR_FAIL,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn abi_world_round_trip() {
        let _ = vs_runtime_shutdown();
        assert_eq!(vs_runtime_init(), VS_OK);
        assert_eq!(vs_world_create(), VS_OK);

        let input = VsInputState {
            move_x: 0.0,
            jump_pressed: 0,
            jump_held: 0,
            dash_pressed: 0,
            roll_held: 0,
            restart_pressed: 0,
            qte_pressed: 0,
        };
        assert_eq!(vs_world_tick(&input, 1.0 / 60.0), VS_OK);

        let mut run = VsRunState::default();
        assert_eq!(vs_world_get_run_state(&mut run), VS_OK);
        assert_eq!(run.alive, 1);

        let mut buf = [VsSpriteInstance {
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
            u0: 0.0,
            v0: 0.0,
            u1: 0.0,
            v1: 0.0,
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
            texture_id: 0,
        }; 64];
        let mut count = 0u32;
        let mut cam = VsCamera2D {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
            viewport_w: 1.0,
            viewport_h: 1.0,
        };
        assert_eq!(
            vs_world_extract(buf.as_mut_ptr(), buf.len() as u32, &mut count, &mut cam),
            VS_OK
        );
        assert!(count >= 8);
        assert!(cam.viewport_w > 0.0);

        let version = unsafe { CStr::from_ptr(vs_runtime_version()) };
        assert_eq!(version.to_str().unwrap(), runtime_core::RUNTIME_VERSION);
        assert_eq!(vs_abi_version_minor(), 7);

        let mut ev = [VsGameEvent {
            kind: 0,
            x: 0.0,
            y: 0.0,
            data: 0,
        }; 32];
        let mut ev_count = 0u32;
        assert_eq!(
            vs_world_drain_events(ev.as_mut_ptr(), ev.len() as u32, &mut ev_count),
            VS_OK
        );

        assert_eq!(vs_world_destroy(), VS_OK);
        assert_eq!(vs_runtime_shutdown(), VS_OK);
    }
}
