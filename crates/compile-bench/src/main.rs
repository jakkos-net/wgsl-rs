//! Compile-time benchmark fixture for big `#[wgsl]` shader modules
//! (wgsl-rs#221).
//!
//! Vendors the psrdnoise3 port from the mvp repro
//! <https://github.com/jakkos-net/wgsl_rs_big_ctor>: a ~900-line
//! `#[wgsl]` mod whose macro expansion is a ~12,000-line
//! `__wgsl_module_ctor` function, costing ~14s of LLVM time per
//! incremental rebuild at `opt-level=1`.
//!
//! Driver: `cargo xtask bench` (crates/xtask/src/bench.rs) mutates this
//! file, times `cargo build -p compile-bench`, and restores the file
//! afterwards. Two mutation anchors live here and must be preserved:
//!
//! * the `BENCH_TICK` const inside the mod (in-mode literal edits)
//! * the file top (above-mode line insertion)
//!
//! Do not add other `#[wgsl]` mods or move this one; the bench measures
//! this file as-is. If a bench run is interrupted before it restores
//! the source, run `git checkout crates/compile-bench/src/main.rs`.

fn main() {
    std::hint::black_box(&psrd3::WGSL_SOURCE);
}

// psrdnoise (c) 2021 Stefan Gustavson and Ian McEwan
// Published under the MIT license.
// https://github.com/stegu/psrdnoise/
// ported from https://github.com/stegu/psrdnoise/blob/419175a270862ce7ae692038fafafb42ec0427e9/src/psrdnoise3.wgsl
// Rust port for wgsl-rs by jakkos-net: https://github.com/jakkos-net/wgsl_rs_big_ctor

// I tried to keep it as close to the original WGSL as possible, this leads to
// some warnings about breaking style guides etc, silence them
#[allow(
    unused_mut,
    non_snake_case,
    unused_parens,
    dead_code,
    clippy::too_many_arguments,
    clippy::needless_return,
    clippy::excessive_precision
)]
#[wgsl_rs::wgsl]
mod psrd3 {
    use wgsl_rs::std::*;

    // Incremental-compile anchor for `cargo xtask bench`: the driver
    // rewrites the literal below between iterations. Same byte length,
    // different tokens, so the proc-macro expansion fingerprint changes
    // without shifting any spans.
    pub const BENCH_TICK: f32 = 0.0;

    // HELPERS
    pub fn mat3x3f_(
        a: f32,
        b: f32,
        c: f32,
        d: f32,
        e: f32,
        f: f32,
        g: f32,
        h: f32,
        i: f32,
    ) -> Mat3x3f {
        mat3x3f(vec3f(a, b, c), vec3f(d, e, f), vec3f(g, h, i))
    }

    pub fn any_v3f_gtz(v: Vec3f) -> bool {
        v.x > 0. || v.y > 0. || v.z > 0.
    }

    pub fn v4f_splat(f: f32) -> Vec4f {
        vec4f(f, f, f, f)
    }

    // PORT
    pub struct NG3 {
        pub noise: f32,
        pub gradient: Vec3f,
    }

    pub fn mod289v4f(i: Vec4f) -> Vec4f {
        return i - floor(i / 289.0) * 289.0;
    }

    pub fn permute289v4f(i: Vec4f) -> Vec4f {
        let mut im: Vec4f = mod289v4f(i);
        return mod289v4f((im * 34.0 + 10.0) * im);
    }

    pub fn psrnoise3(x: Vec3f, p: Vec3f, alpha: f32) -> f32 {
        let M = mat3x3f_(0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0);
        let Mi = mat3x3f_(-0.5, 0.5, 0.5, 0.5, -0.5, 0.5, 0.5, 0.5, -0.5);

        let mut uvw: Vec3f;
        let mut i0: Vec3f;
        let mut i1: Vec3f;
        let mut i2: Vec3f;
        let mut i3: Vec3f;
        let mut f0: Vec3f;
        let mut gt_: Vec3f;
        let mut lt_: Vec3f;
        let mut gt: Vec3f;
        let mut lt: Vec3f;
        let mut o1: Vec3f;
        let mut o2: Vec3f;
        let mut v0: Vec3f;
        let mut v1: Vec3f;
        let mut v2: Vec3f;
        let mut v3: Vec3f;
        let mut x0: Vec3f;
        let mut x1: Vec3f;
        let mut x2: Vec3f;
        let mut x3: Vec3f;

        uvw = M * x;
        i0 = floor(uvw);
        f0 = uvw - i0;
        gt_ = step(f0.xyx(), f0.yzz());
        lt_ = 1.0 - gt_;
        gt = vec3f(lt_.z, gt_.x, gt_.y);
        lt = vec3f(lt_.x, lt_.y, gt_.z);
        o1 = min(gt, lt);
        o2 = max(gt, lt);
        i1 = i0 + o1;
        i2 = i0 + o2;
        i3 = i0 + vec3f(1.0, 1.0, 1.0);
        v0 = Mi * i0;
        v1 = Mi * i1;
        v2 = Mi * i2;
        v3 = Mi * i3;
        x0 = x - v0;
        x1 = x - v1;
        x2 = x - v2;
        x3 = x - v3;

        let mut vx: Vec4f;
        let mut vy: Vec4f;
        let mut vz: Vec4f;

        if (any_v3f_gtz(p)) {
            vx = vec4f(v0.x, v1.x, v2.x, v3.x);
            vy = vec4f(v0.y, v1.y, v2.y, v3.y);
            vz = vec4f(v0.z, v1.z, v2.z, v3.z);
            if (p.x > 0.0) {
                vx = vx - floor(vx / p.x) * p.x;
            }
            if (p.y > 0.0) {
                vy = vy - floor(vy / p.y) * p.y;
            }
            if (p.z > 0.0) {
                vz = vz - floor(vz / p.z) * p.z;
            }
            i0 = floor(M * vec3f(vx.x, vy.x, vz.x) + 0.5);
            i1 = floor(M * vec3f(vx.y, vy.y, vz.y) + 0.5);
            i2 = floor(M * vec3f(vx.z, vy.z, vz.z) + 0.5);
            i3 = floor(M * vec3f(vx.w, vy.w, vz.w) + 0.5);
        }

        let mut hash: Vec4f;
        let mut theta: Vec4f;
        let mut sz: Vec4f;
        let mut psi: Vec4f;
        let mut St: Vec4f;
        let mut Ct: Vec4f;
        let mut sz_: Vec4f;

        hash = permute289v4f(
            permute289v4f(
                permute289v4f(vec4f(i0.z, i1.z, i2.z, i3.z)) + vec4f(i0.y, i1.y, i2.y, i3.y),
            ) + vec4f(i0.x, i1.x, i2.x, i3.x),
        );
        theta = hash * 3.883222077;
        sz = hash * -0.006920415 + 0.996539792;
        psi = hash * 0.108705628;
        Ct = cos(theta);
        St = sin(theta);
        sz_ = sqrt(1.0 - sz * sz);

        let mut gx: Vec4f;
        let mut gy: Vec4f;
        let mut gz: Vec4f;
        let mut px: Vec4f;
        let mut py: Vec4f;
        let mut pz: Vec4f;
        let mut Sp: Vec4f;
        let mut Cp: Vec4f;
        let mut Ctp: Vec4f;
        let mut qx: Vec4f;
        let mut qy: Vec4f;
        let mut qz: Vec4f;
        let mut Sa: Vec4f;
        let mut Ca: Vec4f;

        if (alpha != 0.0) {
            px = Ct * sz_;
            py = St * sz_;
            pz = sz;
            Sp = sin(psi);
            Cp = cos(psi);
            Ctp = St * Sp - Ct * Cp;
            qx = mix(Ctp * St, Sp, sz);
            qy = mix(-Ctp * Ct, Cp, sz);
            qz = -(py * Cp + px * Sp);
            Sa = v4f_splat(sin(alpha));
            Ca = v4f_splat(cos(alpha));
            gx = Ca * px + Sa * qx;
            gy = Ca * py + Sa * qy;
            gz = Ca * pz + Sa * qz;
        } else {
            gx = Ct * sz_;
            gy = St * sz_;
            gz = sz;
        }

        let mut g0: Vec3f;
        let mut g1: Vec3f;
        let mut g2: Vec3f;
        let mut g3: Vec3f;
        let mut w: Vec4f;
        let mut w2: Vec4f;
        let mut w3: Vec4f;
        let mut gdotx: Vec4f;
        let mut n: f32;

        g0 = vec3f(gx.x, gy.x, gz.x);
        g1 = vec3f(gx.y, gy.y, gz.y);
        g2 = vec3f(gx.z, gy.z, gz.z);
        g3 = vec3f(gx.w, gy.w, gz.w);
        w = 0.5 - vec4f(dot(x0, x0), dot(x1, x1), dot(x2, x2), dot(x3, x3));
        w = max(w, vec4f(0.0, 0.0, 0.0, 0.0));
        w2 = w * w;
        w3 = w2 * w;
        gdotx = vec4f(dot(g0, x0), dot(g1, x1), dot(g2, x2), dot(g3, x3));
        n = dot(w3, gdotx);

        return 39.5 * n;
    }

    pub fn psnoise3(x: Vec3f, p: Vec3f) -> f32 {
        let M = mat3x3f_(0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0);
        let Mi = mat3x3f_(-0.5, 0.5, 0.5, 0.5, -0.5, 0.5, 0.5, 0.5, -0.5);

        let mut uvw: Vec3f;
        let mut i0: Vec3f;
        let mut i1: Vec3f;
        let mut i2: Vec3f;
        let mut i3: Vec3f;
        let mut f0: Vec3f;
        let mut gt_: Vec3f;
        let mut lt_: Vec3f;
        let mut gt: Vec3f;
        let mut lt: Vec3f;
        let mut o1: Vec3f;
        let mut o2: Vec3f;
        let mut v0: Vec3f;
        let mut v1: Vec3f;
        let mut v2: Vec3f;
        let mut v3: Vec3f;
        let mut x0: Vec3f;
        let mut x1: Vec3f;
        let mut x2: Vec3f;
        let mut x3: Vec3f;

        uvw = M * x;
        i0 = floor(uvw);
        f0 = uvw - i0;
        gt_ = step(f0.xyx(), f0.yzz());
        lt_ = 1.0 - gt_;
        gt = vec3f(lt_.z, gt_.x, gt_.y);
        lt = vec3f(lt_.x, lt_.y, gt_.z);
        o1 = min(gt, lt);
        o2 = max(gt, lt);
        i1 = i0 + o1;
        i2 = i0 + o2;
        i3 = i0 + vec3f(1.0, 1.0, 1.0);
        v0 = Mi * i0;
        v1 = Mi * i1;
        v2 = Mi * i2;
        v3 = Mi * i3;
        x0 = x - v0;
        x1 = x - v1;
        x2 = x - v2;
        x3 = x - v3;

        let mut vx: Vec4f;
        let mut vy: Vec4f;
        let mut vz: Vec4f;

        if (any_v3f_gtz(p)) {
            vx = vec4f(v0.x, v1.x, v2.x, v3.x);
            vy = vec4f(v0.y, v1.y, v2.y, v3.y);
            vz = vec4f(v0.z, v1.z, v2.z, v3.z);
            if (p.x > 0.0) {
                vx = vx - floor(vx / p.x) * p.x;
            }
            if (p.y > 0.0) {
                vy = vy - floor(vy / p.y) * p.y;
            }
            if (p.z > 0.0) {
                vz = vz - floor(vz / p.z) * p.z;
            }
            i0 = floor(M * vec3f(vx.x, vy.x, vz.x) + 0.5);
            i1 = floor(M * vec3f(vx.y, vy.y, vz.y) + 0.5);
            i2 = floor(M * vec3f(vx.z, vy.z, vz.z) + 0.5);
            i3 = floor(M * vec3f(vx.w, vy.w, vz.w) + 0.5);
        }

        let mut hash: Vec4f;
        let mut theta: Vec4f;
        let mut sz: Vec4f;
        // let mut psi: Vec4f;
        let mut St: Vec4f;
        let mut Ct: Vec4f;
        let mut sz_: Vec4f;

        hash = permute289v4f(
            permute289v4f(
                permute289v4f(vec4f(i0.z, i1.z, i2.z, i3.z)) + vec4f(i0.y, i1.y, i2.y, i3.y),
            ) + vec4f(i0.x, i1.x, i2.x, i3.x),
        );
        theta = hash * 3.883222077;
        sz = hash * -0.006920415 + 0.996539792;
        // psi = hash * 0.108705628;
        Ct = cos(theta);
        St = sin(theta);
        sz_ = sqrt(1.0 - sz * sz);

        let mut gx: Vec4f;
        let mut gy: Vec4f;
        let mut gz: Vec4f;

        gx = Ct * sz_;
        gy = St * sz_;
        gz = sz;

        let mut g0: Vec3f;
        let mut g1: Vec3f;
        let mut g2: Vec3f;
        let mut g3: Vec3f;
        let mut w: Vec4f;
        let mut w2: Vec4f;
        let mut w3: Vec4f;
        let mut gdotx: Vec4f;
        let mut n: f32;

        g0 = vec3f(gx.x, gy.x, gz.x);
        g1 = vec3f(gx.y, gy.y, gz.y);
        g2 = vec3f(gx.z, gy.z, gz.z);
        g3 = vec3f(gx.w, gy.w, gz.w);
        w = 0.5 - vec4f(dot(x0, x0), dot(x1, x1), dot(x2, x2), dot(x3, x3));
        w = max(w, vec4f(0.0, 0.0, 0.0, 0.0));
        w2 = w * w;
        w3 = w2 * w;
        gdotx = vec4f(dot(g0, x0), dot(g1, x1), dot(g2, x2), dot(g3, x3));
        n = dot(w3, gdotx);

        return 39.5 * n;
    }

    pub fn srnoise3(x: Vec3f, alpha: f32) -> f32 {
        let M = mat3x3f_(0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0);
        let Mi = mat3x3f_(-0.5, 0.5, 0.5, 0.5, -0.5, 0.5, 0.5, 0.5, -0.5);

        let mut uvw: Vec3f;
        let mut i0: Vec3f;
        let mut i1: Vec3f;
        let mut i2: Vec3f;
        let mut i3: Vec3f;
        let mut f0: Vec3f;
        let mut gt_: Vec3f;
        let mut lt_: Vec3f;
        let mut gt: Vec3f;
        let mut lt: Vec3f;
        let mut o1: Vec3f;
        let mut o2: Vec3f;
        let mut v0: Vec3f;
        let mut v1: Vec3f;
        let mut v2: Vec3f;
        let mut v3: Vec3f;
        let mut x0: Vec3f;
        let mut x1: Vec3f;
        let mut x2: Vec3f;
        let mut x3: Vec3f;

        uvw = M * x;
        i0 = floor(uvw);
        f0 = uvw - i0;
        gt_ = step(f0.xyx(), f0.yzz());
        lt_ = 1.0 - gt_;
        gt = vec3f(lt_.z, gt_.x, gt_.y);
        lt = vec3f(lt_.x, lt_.y, gt_.z);
        o1 = min(gt, lt);
        o2 = max(gt, lt);
        i1 = i0 + o1;
        i2 = i0 + o2;
        i3 = i0 + vec3f(1.0, 1.0, 1.0);
        v0 = Mi * i0;
        v1 = Mi * i1;
        v2 = Mi * i2;
        v3 = Mi * i3;
        x0 = x - v0;
        x1 = x - v1;
        x2 = x - v2;
        x3 = x - v3;

        let mut hash: Vec4f;
        let mut theta: Vec4f;
        let mut sz: Vec4f;
        let mut psi: Vec4f;
        let mut St: Vec4f;
        let mut Ct: Vec4f;
        let mut sz_: Vec4f;

        hash = permute289v4f(
            permute289v4f(
                permute289v4f(vec4f(i0.z, i1.z, i2.z, i3.z)) + vec4f(i0.y, i1.y, i2.y, i3.y),
            ) + vec4f(i0.x, i1.x, i2.x, i3.x),
        );
        theta = hash * 3.883222077;
        sz = hash * -0.006920415 + 0.996539792;
        psi = hash * 0.108705628;
        Ct = cos(theta);
        St = sin(theta);
        sz_ = sqrt(1.0 - sz * sz);

        let mut gx: Vec4f;
        let mut gy: Vec4f;
        let mut gz: Vec4f;
        let mut px: Vec4f;
        let mut py: Vec4f;
        let mut pz: Vec4f;
        let mut Sp: Vec4f;
        let mut Cp: Vec4f;
        let mut Ctp: Vec4f;
        let mut qx: Vec4f;
        let mut qy: Vec4f;
        let mut qz: Vec4f;
        let mut Sa: Vec4f;
        let mut Ca: Vec4f;

        if (alpha != 0.0) {
            px = Ct * sz_;
            py = St * sz_;
            pz = sz;
            Sp = sin(psi);
            Cp = cos(psi);
            Ctp = St * Sp - Ct * Cp;
            qx = mix(Ctp * St, Sp, sz);
            qy = mix(-Ctp * Ct, Cp, sz);
            qz = -(py * Cp + px * Sp);
            Sa = v4f_splat(sin(alpha));
            Ca = v4f_splat(cos(alpha));
            gx = Ca * px + Sa * qx;
            gy = Ca * py + Sa * qy;
            gz = Ca * pz + Sa * qz;
        } else {
            gx = Ct * sz_;
            gy = St * sz_;
            gz = sz;
        }

        let mut g0: Vec3f;
        let mut g1: Vec3f;
        let mut g2: Vec3f;
        let mut g3: Vec3f;
        let mut w: Vec4f;
        let mut w2: Vec4f;
        let mut w3: Vec4f;
        let mut gdotx: Vec4f;
        let mut n: f32;

        g0 = vec3f(gx.x, gy.x, gz.x);
        g1 = vec3f(gx.y, gy.y, gz.y);
        g2 = vec3f(gx.z, gy.z, gz.z);
        g3 = vec3f(gx.w, gy.w, gz.w);
        w = 0.5 - vec4f(dot(x0, x0), dot(x1, x1), dot(x2, x2), dot(x3, x3));
        w = max(w, vec4f(0.0, 0.0, 0.0, 0.0));
        w2 = w * w;
        w3 = w2 * w;
        gdotx = vec4f(dot(g0, x0), dot(g1, x1), dot(g2, x2), dot(g3, x3));
        n = dot(w3, gdotx);

        return 39.5 * n;
    }

    pub fn snoise3(x: Vec3f) -> f32 {
        let M = mat3x3f_(0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0);
        let Mi = mat3x3f_(-0.5, 0.5, 0.5, 0.5, -0.5, 0.5, 0.5, 0.5, -0.5);

        let mut uvw: Vec3f;
        let mut i0: Vec3f;
        let mut i1: Vec3f;
        let mut i2: Vec3f;
        let mut i3: Vec3f;
        let mut f0: Vec3f;
        let mut gt_: Vec3f;
        let mut lt_: Vec3f;
        let mut gt: Vec3f;
        let mut lt: Vec3f;
        let mut o1: Vec3f;
        let mut o2: Vec3f;
        let mut v0: Vec3f;
        let mut v1: Vec3f;
        let mut v2: Vec3f;
        let mut v3: Vec3f;
        let mut x0: Vec3f;
        let mut x1: Vec3f;
        let mut x2: Vec3f;
        let mut x3: Vec3f;

        uvw = M * x;
        i0 = floor(uvw);
        f0 = uvw - i0;
        gt_ = step(f0.xyx(), f0.yzz());
        lt_ = 1.0 - gt_;
        gt = vec3f(lt_.z, gt_.x, gt_.y);
        lt = vec3f(lt_.x, lt_.y, gt_.z);
        o1 = min(gt, lt);
        o2 = max(gt, lt);
        i1 = i0 + o1;
        i2 = i0 + o2;
        i3 = i0 + vec3f(1.0, 1.0, 1.0);
        v0 = Mi * i0;
        v1 = Mi * i1;
        v2 = Mi * i2;
        v3 = Mi * i3;
        x0 = x - v0;
        x1 = x - v1;
        x2 = x - v2;
        x3 = x - v3;

        let mut hash: Vec4f;
        let mut theta: Vec4f;
        let mut sz: Vec4f;
        // let mut psi: Vec4f;
        let mut St: Vec4f;
        let mut Ct: Vec4f;
        let mut sz_: Vec4f;

        hash = permute289v4f(
            permute289v4f(
                permute289v4f(vec4f(i0.z, i1.z, i2.z, i3.z)) + vec4f(i0.y, i1.y, i2.y, i3.y),
            ) + vec4f(i0.x, i1.x, i2.x, i3.x),
        );
        theta = hash * 3.883222077;
        sz = hash * -0.006920415 + 0.996539792;
        // psi = hash * 0.108705628;
        Ct = cos(theta);
        St = sin(theta);
        sz_ = sqrt(1.0 - sz * sz);

        let mut gx: Vec4f;
        let mut gy: Vec4f;
        let mut gz: Vec4f;

        gx = Ct * sz_;
        gy = St * sz_;
        gz = sz;

        let mut g0: Vec3f;
        let mut g1: Vec3f;
        let mut g2: Vec3f;
        let mut g3: Vec3f;
        let mut w: Vec4f;
        let mut w2: Vec4f;
        let mut w3: Vec4f;
        let mut gdotx: Vec4f;
        let mut n: f32;

        g0 = vec3f(gx.x, gy.x, gz.x);
        g1 = vec3f(gx.y, gy.y, gz.y);
        g2 = vec3f(gx.z, gy.z, gz.z);
        g3 = vec3f(gx.w, gy.w, gz.w);
        w = 0.5 - vec4f(dot(x0, x0), dot(x1, x1), dot(x2, x2), dot(x3, x3));
        w = max(w, vec4f(0.0, 0.0, 0.0, 0.0));
        w2 = w * w;
        w3 = w2 * w;
        gdotx = vec4f(dot(g0, x0), dot(g1, x1), dot(g2, x2), dot(g3, x3));
        n = dot(w3, gdotx);

        return 39.5 * n;
    }

    // Versions computing their gradient and returning a struct

    pub fn psrdnoise3(x: Vec3f, p: Vec3f, alpha: f32) -> NG3 {
        let M = mat3x3f_(0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0);
        let Mi = mat3x3f_(-0.5, 0.5, 0.5, 0.5, -0.5, 0.5, 0.5, 0.5, -0.5);

        let mut uvw: Vec3f;
        let mut i0: Vec3f;
        let mut i1: Vec3f;
        let mut i2: Vec3f;
        let mut i3: Vec3f;
        let mut f0: Vec3f;
        let mut gt_: Vec3f;
        let mut lt_: Vec3f;
        let mut gt: Vec3f;
        let mut lt: Vec3f;
        let mut o1: Vec3f;
        let mut o2: Vec3f;
        let mut v0: Vec3f;
        let mut v1: Vec3f;
        let mut v2: Vec3f;
        let mut v3: Vec3f;
        let mut x0: Vec3f;
        let mut x1: Vec3f;
        let mut x2: Vec3f;
        let mut x3: Vec3f;

        uvw = M * x;
        i0 = floor(uvw);
        f0 = uvw - i0;
        gt_ = step(f0.xyx(), f0.yzz());
        lt_ = 1.0 - gt_;
        gt = vec3f(lt_.z, gt_.x, gt_.y);
        lt = vec3f(lt_.x, lt_.y, gt_.z);
        o1 = min(gt, lt);
        o2 = max(gt, lt);
        i1 = i0 + o1;
        i2 = i0 + o2;
        i3 = i0 + vec3f(1.0, 1.0, 1.0);
        v0 = Mi * i0;
        v1 = Mi * i1;
        v2 = Mi * i2;
        v3 = Mi * i3;
        x0 = x - v0;
        x1 = x - v1;
        x2 = x - v2;
        x3 = x - v3;

        let mut vx: Vec4f;
        let mut vy: Vec4f;
        let mut vz: Vec4f;

        if (any_v3f_gtz(p)) {
            vx = vec4f(v0.x, v1.x, v2.x, v3.x);
            vy = vec4f(v0.y, v1.y, v2.y, v3.y);
            vz = vec4f(v0.z, v1.z, v2.z, v3.z);
            if (p.x > 0.0) {
                vx = vx - floor(vx / p.x) * p.x;
            }
            if (p.y > 0.0) {
                vy = vy - floor(vy / p.y) * p.y;
            }
            if (p.z > 0.0) {
                vz = vz - floor(vz / p.z) * p.z;
            }
            i0 = floor(M * vec3f(vx.x, vy.x, vz.x) + 0.5);
            i1 = floor(M * vec3f(vx.y, vy.y, vz.y) + 0.5);
            i2 = floor(M * vec3f(vx.z, vy.z, vz.z) + 0.5);
            i3 = floor(M * vec3f(vx.w, vy.w, vz.w) + 0.5);
        }

        let mut hash: Vec4f;
        let mut theta: Vec4f;
        let mut sz: Vec4f;
        let mut psi: Vec4f;
        let mut St: Vec4f;
        let mut Ct: Vec4f;
        let mut sz_: Vec4f;

        hash = permute289v4f(
            permute289v4f(
                permute289v4f(vec4f(i0.z, i1.z, i2.z, i3.z)) + vec4f(i0.y, i1.y, i2.y, i3.y),
            ) + vec4f(i0.x, i1.x, i2.x, i3.x),
        );
        theta = hash * 3.883222077;
        sz = hash * -0.006920415 + 0.996539792;
        psi = hash * 0.108705628;
        Ct = cos(theta);
        St = sin(theta);
        sz_ = sqrt(1.0 - sz * sz);

        let mut gx: Vec4f;
        let mut gy: Vec4f;
        let mut gz: Vec4f;
        let mut px: Vec4f;
        let mut py: Vec4f;
        let mut pz: Vec4f;
        let mut Sp: Vec4f;
        let mut Cp: Vec4f;
        let mut Ctp: Vec4f;
        let mut qx: Vec4f;
        let mut qy: Vec4f;
        let mut qz: Vec4f;
        let mut Sa: Vec4f;
        let mut Ca: Vec4f;

        if (alpha != 0.0) {
            px = Ct * sz_;
            py = St * sz_;
            pz = sz;
            Sp = sin(psi);
            Cp = cos(psi);
            Ctp = St * Sp - Ct * Cp;
            qx = mix(Ctp * St, Sp, sz);
            qy = mix(-Ctp * Ct, Cp, sz);
            qz = -(py * Cp + px * Sp);
            Sa = v4f_splat(sin(alpha));
            Ca = v4f_splat(cos(alpha));
            gx = Ca * px + Sa * qx;
            gy = Ca * py + Sa * qy;
            gz = Ca * pz + Sa * qz;
        } else {
            gx = Ct * sz_;
            gy = St * sz_;
            gz = sz;
        }

        let mut g0: Vec3f;
        let mut g1: Vec3f;
        let mut g2: Vec3f;
        let mut g3: Vec3f;
        let mut w: Vec4f;
        let mut w2: Vec4f;
        let mut w3: Vec4f;
        let mut gdotx: Vec4f;
        let mut n: f32;

        g0 = vec3f(gx.x, gy.x, gz.x);
        g1 = vec3f(gx.y, gy.y, gz.y);
        g2 = vec3f(gx.z, gy.z, gz.z);
        g3 = vec3f(gx.w, gy.w, gz.w);
        w = 0.5 - vec4f(dot(x0, x0), dot(x1, x1), dot(x2, x2), dot(x3, x3));
        w = max(w, vec4f(0.0, 0.0, 0.0, 0.0));
        w2 = w * w;
        w3 = w2 * w;
        gdotx = vec4f(dot(g0, x0), dot(g1, x1), dot(g2, x2), dot(g3, x3));
        n = 39.5 * dot(w3, gdotx);

        let mut dw: Vec4f = -6.0 * w2 * gdotx;
        let mut dn0: Vec3f = w3.x * g0 + dw.x * x0;
        let mut dn1: Vec3f = w3.y * g1 + dw.y * x1;
        let mut dn2: Vec3f = w3.z * g2 + dw.z * x2;
        let mut dn3: Vec3f = w3.w * g3 + dw.w * x3;
        let mut g: Vec3f = 39.5 * (dn0 + dn1 + dn2 + dn3);

        return NG3 {
            noise: n,
            gradient: g,
        };
    }

    pub fn psdnoise3(x: Vec3f, p: Vec3f) -> NG3 {
        let M = mat3x3f_(0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0);
        let Mi = mat3x3f_(-0.5, 0.5, 0.5, 0.5, -0.5, 0.5, 0.5, 0.5, -0.5);

        let mut uvw: Vec3f;
        let mut i0: Vec3f;
        let mut i1: Vec3f;
        let mut i2: Vec3f;
        let mut i3: Vec3f;
        let mut f0: Vec3f;
        let mut gt_: Vec3f;
        let mut lt_: Vec3f;
        let mut gt: Vec3f;
        let mut lt: Vec3f;
        let mut o1: Vec3f;
        let mut o2: Vec3f;
        let mut v0: Vec3f;
        let mut v1: Vec3f;
        let mut v2: Vec3f;
        let mut v3: Vec3f;
        let mut x0: Vec3f;
        let mut x1: Vec3f;
        let mut x2: Vec3f;
        let mut x3: Vec3f;

        uvw = M * x;
        i0 = floor(uvw);
        f0 = uvw - i0;
        gt_ = step(f0.xyx(), f0.yzz());
        lt_ = 1.0 - gt_;
        gt = vec3f(lt_.z, gt_.x, gt_.y);
        lt = vec3f(lt_.x, lt_.y, gt_.z);
        o1 = min(gt, lt);
        o2 = max(gt, lt);
        i1 = i0 + o1;
        i2 = i0 + o2;
        i3 = i0 + vec3f(1.0, 1.0, 1.0);
        v0 = Mi * i0;
        v1 = Mi * i1;
        v2 = Mi * i2;
        v3 = Mi * i3;
        x0 = x - v0;
        x1 = x - v1;
        x2 = x - v2;
        x3 = x - v3;

        let mut vx: Vec4f;
        let mut vy: Vec4f;
        let mut vz: Vec4f;

        if (any_v3f_gtz(p)) {
            vx = vec4f(v0.x, v1.x, v2.x, v3.x);
            vy = vec4f(v0.y, v1.y, v2.y, v3.y);
            vz = vec4f(v0.z, v1.z, v2.z, v3.z);
            if (p.x > 0.0) {
                vx = vx - floor(vx / p.x) * p.x;
            }
            if (p.y > 0.0) {
                vy = vy - floor(vy / p.y) * p.y;
            }
            if (p.z > 0.0) {
                vz = vz - floor(vz / p.z) * p.z;
            }
            i0 = floor(M * vec3f(vx.x, vy.x, vz.x) + 0.5);
            i1 = floor(M * vec3f(vx.y, vy.y, vz.y) + 0.5);
            i2 = floor(M * vec3f(vx.z, vy.z, vz.z) + 0.5);
            i3 = floor(M * vec3f(vx.w, vy.w, vz.w) + 0.5);
        }

        let mut hash: Vec4f;
        let mut theta: Vec4f;
        let mut sz: Vec4f;
        // let mut psi: Vec4f;
        let mut St: Vec4f;
        let mut Ct: Vec4f;
        let mut sz_: Vec4f;

        hash = permute289v4f(
            permute289v4f(
                permute289v4f(vec4f(i0.z, i1.z, i2.z, i3.z)) + vec4f(i0.y, i1.y, i2.y, i3.y),
            ) + vec4f(i0.x, i1.x, i2.x, i3.x),
        );
        theta = hash * 3.883222077;
        sz = hash * -0.006920415 + 0.996539792;
        // psi = hash * 0.108705628;
        Ct = cos(theta);
        St = sin(theta);
        sz_ = sqrt(1.0 - sz * sz);

        let mut gx: Vec4f;
        let mut gy: Vec4f;
        let mut gz: Vec4f;

        gx = Ct * sz_;
        gy = St * sz_;
        gz = sz;

        let mut g0: Vec3f;
        let mut g1: Vec3f;
        let mut g2: Vec3f;
        let mut g3: Vec3f;
        let mut w: Vec4f;
        let mut w2: Vec4f;
        let mut w3: Vec4f;
        let mut gdotx: Vec4f;
        let mut n: f32;

        g0 = vec3f(gx.x, gy.x, gz.x);
        g1 = vec3f(gx.y, gy.y, gz.y);
        g2 = vec3f(gx.z, gy.z, gz.z);
        g3 = vec3f(gx.w, gy.w, gz.w);
        w = 0.5 - vec4f(dot(x0, x0), dot(x1, x1), dot(x2, x2), dot(x3, x3));
        w = max(w, vec4f(0.0, 0.0, 0.0, 0.0));
        w2 = w * w;
        w3 = w2 * w;
        gdotx = vec4f(dot(g0, x0), dot(g1, x1), dot(g2, x2), dot(g3, x3));
        n = 39.5 * dot(w3, gdotx);

        let mut dw: Vec4f = -6.0 * w2 * gdotx;
        let mut dn0: Vec3f = w3.x * g0 + dw.x * x0;
        let mut dn1: Vec3f = w3.y * g1 + dw.y * x1;
        let mut dn2: Vec3f = w3.z * g2 + dw.z * x2;
        let mut dn3: Vec3f = w3.w * g3 + dw.w * x3;
        let mut g: Vec3f = 39.5 * (dn0 + dn1 + dn2 + dn3);

        return NG3 {
            noise: n,
            gradient: g,
        };
    }

    pub fn srdnoise3(x: Vec3f, alpha: f32) -> NG3 {
        let M = mat3x3f_(0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0);
        let Mi = mat3x3f_(-0.5, 0.5, 0.5, 0.5, -0.5, 0.5, 0.5, 0.5, -0.5);

        let mut uvw: Vec3f;
        let mut i0: Vec3f;
        let mut i1: Vec3f;
        let mut i2: Vec3f;
        let mut i3: Vec3f;
        let mut f0: Vec3f;
        let mut gt_: Vec3f;
        let mut lt_: Vec3f;
        let mut gt: Vec3f;
        let mut lt: Vec3f;
        let mut o1: Vec3f;
        let mut o2: Vec3f;
        let mut v0: Vec3f;
        let mut v1: Vec3f;
        let mut v2: Vec3f;
        let mut v3: Vec3f;
        let mut x0: Vec3f;
        let mut x1: Vec3f;
        let mut x2: Vec3f;
        let mut x3: Vec3f;

        uvw = M * x;
        i0 = floor(uvw);
        f0 = uvw - i0;
        gt_ = step(f0.xyx(), f0.yzz());
        lt_ = 1.0 - gt_;
        gt = vec3f(lt_.z, gt_.x, gt_.y);
        lt = vec3f(lt_.x, lt_.y, gt_.z);
        o1 = min(gt, lt);
        o2 = max(gt, lt);
        i1 = i0 + o1;
        i2 = i0 + o2;
        i3 = i0 + vec3f(1.0, 1.0, 1.0);
        v0 = Mi * i0;
        v1 = Mi * i1;
        v2 = Mi * i2;
        v3 = Mi * i3;
        x0 = x - v0;
        x1 = x - v1;
        x2 = x - v2;
        x3 = x - v3;

        let mut hash: Vec4f;
        let mut theta: Vec4f;
        let mut sz: Vec4f;
        let mut psi: Vec4f;
        let mut St: Vec4f;
        let mut Ct: Vec4f;
        let mut sz_: Vec4f;

        hash = permute289v4f(
            permute289v4f(
                permute289v4f(vec4f(i0.z, i1.z, i2.z, i3.z)) + vec4f(i0.y, i1.y, i2.y, i3.y),
            ) + vec4f(i0.x, i1.x, i2.x, i3.x),
        );
        theta = hash * 3.883222077;
        sz = hash * -0.006920415 + 0.996539792;
        psi = hash * 0.108705628;
        Ct = cos(theta);
        St = sin(theta);
        sz_ = sqrt(1.0 - sz * sz);

        let mut gx: Vec4f;
        let mut gy: Vec4f;
        let mut gz: Vec4f;
        let mut px: Vec4f;
        let mut py: Vec4f;
        let mut pz: Vec4f;
        let mut Sp: Vec4f;
        let mut Cp: Vec4f;
        let mut Ctp: Vec4f;
        let mut qx: Vec4f;
        let mut qy: Vec4f;
        let mut qz: Vec4f;
        let mut Sa: Vec4f;
        let mut Ca: Vec4f;

        if (alpha != 0.0) {
            px = Ct * sz_;
            py = St * sz_;
            pz = sz;
            Sp = sin(psi);
            Cp = cos(psi);
            Ctp = St * Sp - Ct * Cp;
            qx = mix(Ctp * St, Sp, sz);
            qy = mix(-Ctp * Ct, Cp, sz);
            qz = -(py * Cp + px * Sp);
            Sa = v4f_splat(sin(alpha));
            Ca = v4f_splat(cos(alpha));
            gx = Ca * px + Sa * qx;
            gy = Ca * py + Sa * qy;
            gz = Ca * pz + Sa * qz;
        } else {
            gx = Ct * sz_;
            gy = St * sz_;
            gz = sz;
        }

        let mut g0: Vec3f;
        let mut g1: Vec3f;
        let mut g2: Vec3f;
        let mut g3: Vec3f;
        let mut w: Vec4f;
        let mut w2: Vec4f;
        let mut w3: Vec4f;
        let mut gdotx: Vec4f;
        let mut n: f32;

        g0 = vec3f(gx.x, gy.x, gz.x);
        g1 = vec3f(gx.y, gy.y, gz.y);
        g2 = vec3f(gx.z, gy.z, gz.z);
        g3 = vec3f(gx.w, gy.w, gz.w);
        w = 0.5 - vec4f(dot(x0, x0), dot(x1, x1), dot(x2, x2), dot(x3, x3));
        w = max(w, vec4f(0.0, 0.0, 0.0, 0.0));
        w2 = w * w;
        w3 = w2 * w;
        gdotx = vec4f(dot(g0, x0), dot(g1, x1), dot(g2, x2), dot(g3, x3));
        n = 39.5 * dot(w3, gdotx);

        let mut dw: Vec4f = -6.0 * w2 * gdotx;
        let mut dn0: Vec3f = w3.x * g0 + dw.x * x0;
        let mut dn1: Vec3f = w3.y * g1 + dw.y * x1;
        let mut dn2: Vec3f = w3.z * g2 + dw.z * x2;
        let mut dn3: Vec3f = w3.w * g3 + dw.w * x3;
        let mut g: Vec3f = 39.5 * (dn0 + dn1 + dn2 + dn3);

        return NG3 {
            noise: n,
            gradient: g,
        };
    }

    pub fn sdnoise3(x: Vec3f) -> NG3 {
        let M = mat3x3f_(0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0);
        let Mi = mat3x3f_(-0.5, 0.5, 0.5, 0.5, -0.5, 0.5, 0.5, 0.5, -0.5);

        let mut uvw: Vec3f;
        let mut i0: Vec3f;
        let mut i1: Vec3f;
        let mut i2: Vec3f;
        let mut i3: Vec3f;
        let mut f0: Vec3f;
        let mut gt_: Vec3f;
        let mut lt_: Vec3f;
        let mut gt: Vec3f;
        let mut lt: Vec3f;
        let mut o1: Vec3f;
        let mut o2: Vec3f;
        let mut v0: Vec3f;
        let mut v1: Vec3f;
        let mut v2: Vec3f;
        let mut v3: Vec3f;
        let mut x0: Vec3f;
        let mut x1: Vec3f;
        let mut x2: Vec3f;
        let mut x3: Vec3f;

        uvw = M * x;
        i0 = floor(uvw);
        f0 = uvw - i0;
        gt_ = step(f0.xyx(), f0.yzz());
        lt_ = 1.0 - gt_;
        gt = vec3f(lt_.z, gt_.x, gt_.y);
        lt = vec3f(lt_.x, lt_.y, gt_.z);
        o1 = min(gt, lt);
        o2 = max(gt, lt);
        i1 = i0 + o1;
        i2 = i0 + o2;
        i3 = i0 + vec3f(1.0, 1.0, 1.0);
        v0 = Mi * i0;
        v1 = Mi * i1;
        v2 = Mi * i2;
        v3 = Mi * i3;
        x0 = x - v0;
        x1 = x - v1;
        x2 = x - v2;
        x3 = x - v3;

        let mut hash: Vec4f;
        let mut theta: Vec4f;
        let mut sz: Vec4f;
        // let mut psi: Vec4f;
        let mut St: Vec4f;
        let mut Ct: Vec4f;
        let mut sz_: Vec4f;

        hash = permute289v4f(
            permute289v4f(
                permute289v4f(vec4f(i0.z, i1.z, i2.z, i3.z)) + vec4f(i0.y, i1.y, i2.y, i3.y),
            ) + vec4f(i0.x, i1.x, i2.x, i3.x),
        );
        theta = hash * 3.883222077;
        sz = hash * -0.006920415 + 0.996539792;
        // psi = hash * 0.108705628;
        Ct = cos(theta);
        St = sin(theta);
        sz_ = sqrt(1.0 - sz * sz);

        let mut gx: Vec4f;
        let mut gy: Vec4f;
        let mut gz: Vec4f;

        gx = Ct * sz_;
        gy = St * sz_;
        gz = sz;

        let mut g0: Vec3f;
        let mut g1: Vec3f;
        let mut g2: Vec3f;
        let mut g3: Vec3f;
        let mut w: Vec4f;
        let mut w2: Vec4f;
        let mut w3: Vec4f;
        let mut gdotx: Vec4f;
        let mut n: f32;

        g0 = vec3f(gx.x, gy.x, gz.x);
        g1 = vec3f(gx.y, gy.y, gz.y);
        g2 = vec3f(gx.z, gy.z, gz.z);
        g3 = vec3f(gx.w, gy.w, gz.w);
        w = 0.5 - vec4f(dot(x0, x0), dot(x1, x1), dot(x2, x2), dot(x3, x3));
        w = max(w, vec4f(0.0, 0.0, 0.0, 0.0));
        w2 = w * w;
        w3 = w2 * w;
        gdotx = vec4f(dot(g0, x0), dot(g1, x1), dot(g2, x2), dot(g3, x3));
        n = 39.5 * dot(w3, gdotx);

        let mut dw: Vec4f = -6.0 * w2 * gdotx;
        let mut dn0: Vec3f = w3.x * g0 + dw.x * x0;
        let mut dn1: Vec3f = w3.y * g1 + dw.y * x1;
        let mut dn2: Vec3f = w3.z * g2 + dw.z * x2;
        let mut dn3: Vec3f = w3.w * g3 + dw.w * x3;
        let mut g: Vec3f = 39.5 * (dn0 + dn1 + dn2 + dn3);

        return NG3 {
            noise: n,
            gradient: g,
        };
    }
}
