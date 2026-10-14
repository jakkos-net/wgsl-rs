//! Locks in the placement assumptions behind the renderer's uniform
//! `@interpolate(flat)` emission for integer `@location` IO
//! (see `wgsl-rs-ir/src/render.rs`, `write_inter_stage_io_attrs`).
//!
//! WGSL §12.9 allows `@interpolate` on any declaration that has a
//! `@location` attribute ("only significant on user-defined vertex
//! outputs and fragment inputs"). The renderer emits it uniformly —
//! fn args, return annotations, and struct fields — rather than
//! tracking stage roles at render time, so every placement must stay
//! acceptable to naga. These probes pin that contract: if a future naga
//! starts rejecting any of them (e.g. fragment outputs), this test
//! fails and the emission must become stage-aware.

// Every test here runs through `wgsl_rs::validate_wgsl_source`, which
// only exists with the `validation` feature.
#![cfg(feature = "validation")]

use wgsl_rs::validate_wgsl_source;

fn check(name: &str, src: &str) {
    match validate_wgsl_source(src) {
        Ok(()) => println!("{name}: OK"),
        Err(e) => panic!("{name}: naga rejected explicit flat on an integer location:\n{e}"),
    }
}

#[test]
fn interpolate_flat_accepted_on_all_location_placements() {
    // 1. Vertex INPUT (from vertex buffer).
    check(
        "vertex input",
        "@vertex fn vs(@location(0) @interpolate(flat) i: u32) -> @builtin(position) vec4f { \
         return vec4f(0.0, 0.0, 0.0, 1.0); }",
    );
    // 2. Vertex OUTPUT return.
    check(
        "vertex output return",
        "struct VOut { @builtin(position) p: vec4f, @location(0) @interpolate(flat) i: u32 } \
         @vertex fn vs(@builtin(vertex_index) _vi: u32) -> VOut { return VOut(vec4f(0.0, 0.0, \
         0.0, 1.0), 0u); }",
    );
    // 3. Fragment INPUT.
    check(
        "fragment input",
        "@fragment fn fs(@location(0) @interpolate(flat) i: u32) -> @location(0) vec4f { return \
         vec4f(0.0, 0.0, 0.0, 1.0); }",
    );
    // 4. Fragment OUTPUT return (integer render target).
    check(
        "fragment output return",
        "@fragment fn fs() -> @location(0) @interpolate(flat) u32 { return 0u; }",
    );
    // 5. Struct field used as fragment output.
    check(
        "fragment output struct field",
        "struct Out { @location(0) @interpolate(flat) i: u32 } @fragment fn fs() -> Out { return \
         Out(0u); }",
    );
    // 6. Control: fragment output WITHOUT interpolate must be fine too.
    check(
        "fragment output without interpolation (control)",
        "@fragment fn fs() -> @location(0) u32 { return 0u; }",
    );
}

/// naga 30 requires explicit flat on integer inter-stage IO; this is
/// the bug the renderer's emission fixes, kept as the negative control.
#[test]
fn fragment_input_without_flat_is_rejected() {
    let src = "@fragment fn fs(@location(0) i: u32) -> @location(0) vec4f { return vec4f(0.0, \
               0.0, 0.0, 1.0); }";
    assert!(
        validate_wgsl_source(src).is_err(),
        "naga should reject integer fragment input without @interpolate(flat)"
    );
}
