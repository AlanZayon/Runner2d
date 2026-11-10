#pragma once

#include "vertical_shift_abi.h"

#include <cstdint>

#ifndef __ANDROID__
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <windows.h>
#else
#ifndef VK_LEFT
#define VK_LEFT 0x25
#define VK_RIGHT 0x27
#define VK_UP 0x26
#define VK_DOWN 0x28
#define VK_SPACE 0x20
#define VK_SHIFT 0x10
#endif
#endif

/** Host-side input accumulator mapped to VsInputState each sim tick. */
struct InputState {
    float move_x = 0.0f;
    bool jump_held = false;
    bool jump_pressed_edge = false;
    bool dash_pressed_edge = false;
    bool roll_held = false;
    bool restart_pressed_edge = false;
    bool qte_pressed_edge = false;

    void on_key(int vk, bool down) {
        switch (vk) {
        case 'A':
        case VK_LEFT:
            left_held_ = down;
            break;
        case 'D':
        case VK_RIGHT:
            right_held_ = down;
            break;
        case 'W':
        case VK_UP:
        case VK_SPACE:
            if (down && !jump_held) {
                jump_pressed_edge = true;
            }
            jump_held = down;
            break;
        case 'S':
        case VK_DOWN:
            // Hold is fine for feel; sim only starts one timed roll per press (gate).
            roll_held = down;
            break;
        case 'J':
        case VK_SHIFT:
            if (down && !dash_held_) {
                dash_pressed_edge = true;
            }
            dash_held_ = down;
            break;
        case 'K':
        case 'E':
            // Super Strength QTE confirm (edge).
            if (down && !qte_held_) {
                qte_pressed_edge = true;
            }
            qte_held_ = down;
            break;
        case 'R':
            if (down && !restart_held_) {
                restart_pressed_edge = true;
            }
            restart_held_ = down;
            break;
        default:
            break;
        }
        // A/D strafe only consumed during Falling Mode (sim ignores otherwise).
        move_x = 0.0f;
        if (left_held_) {
            move_x -= 1.0f;
        }
        if (right_held_) {
            move_x += 1.0f;
        }
    }

    void consume_for_tick(VsInputState* out) {
        out->move_x = move_x;
        out->jump_pressed = jump_pressed_edge ? 1u : 0u;
        out->jump_held = jump_held ? 1u : 0u;
        out->dash_pressed = dash_pressed_edge ? 1u : 0u;
        out->roll_held = roll_held ? 1u : 0u;
        out->restart_pressed = restart_pressed_edge ? 1u : 0u;
        out->qte_pressed = qte_pressed_edge ? 1u : 0u;
        jump_pressed_edge = false;
        dash_pressed_edge = false;
        restart_pressed_edge = false;
        qte_pressed_edge = false;
    }

private:
    bool dash_held_ = false;
    bool restart_held_ = false;
    bool qte_held_ = false;
    bool left_held_ = false;
    bool right_held_ = false;
};
