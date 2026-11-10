#pragma once

#include "vec2.hpp"

/** Column-major 3x3 affine matrix for 2D transforms. */
struct Mat3 {
    float m[9] = {
        1, 0, 0,
        0, 1, 0,
        0, 0, 1,
    };

    static Mat3 identity() { return {}; }

    static Mat3 translation(Vec2 t) {
        Mat3 r = identity();
        r.m[6] = t.x;
        r.m[7] = t.y;
        return r;
    }

    static Mat3 scale(Vec2 s) {
        Mat3 r = identity();
        r.m[0] = s.x;
        r.m[4] = s.y;
        return r;
    }

    /** Orthographic projection: world (x right, y down) → clip [-1,1]. */
    static Mat3 ortho(float left, float right, float top, float bottom) {
        Mat3 r{};
        const float rl = right - left;
        const float bt = bottom - top;
        r.m[0] = 2.0f / rl;
        r.m[4] = 2.0f / bt;
        r.m[6] = -(right + left) / rl;
        r.m[7] = -(bottom + top) / bt;
        r.m[8] = 1.0f;
        return r;
    }

    Mat3 operator*(const Mat3& o) const {
        Mat3 r{};
        for (int c = 0; c < 3; ++c) {
            for (int row = 0; row < 3; ++row) {
                r.m[c * 3 + row] =
                    m[0 * 3 + row] * o.m[c * 3 + 0] +
                    m[1 * 3 + row] * o.m[c * 3 + 1] +
                    m[2 * 3 + row] * o.m[c * 3 + 2];
            }
        }
        return r;
    }

    Vec2 transform_point(Vec2 p) const {
        return {
            m[0] * p.x + m[3] * p.y + m[6],
            m[1] * p.x + m[4] * p.y + m[7],
        };
    }

    Mat3 inverse_affine() const {
        // Inverse of 2D affine [[A,t],[0,1]] where A is 2x2 diagonal+rotation-ish.
        const float a = m[0], b = m[3], c = m[1], d = m[4];
        const float det = a * d - b * c;
        Mat3 r = identity();
        if (det == 0.0f) {
            return r;
        }
        const float inv = 1.0f / det;
        r.m[0] = d * inv;
        r.m[3] = -b * inv;
        r.m[1] = -c * inv;
        r.m[4] = a * inv;
        const float tx = m[6], ty = m[7];
        r.m[6] = -(r.m[0] * tx + r.m[3] * ty);
        r.m[7] = -(r.m[1] * tx + r.m[4] * ty);
        return r;
    }
};
