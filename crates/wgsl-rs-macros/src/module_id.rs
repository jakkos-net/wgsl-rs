//! Deterministic assignment of unique ids to `#[wgsl]` modules.
//!
//! Every expanded module gets a `u64` id baked into its `WGSL_SOURCE`
//! static. The runtime deduplicates imported sources by id (diamond
//! import graphs), so two *distinct* modules must never share an id:
//! one would be silently dropped from the assembled WGSL, and the
//! shader would fail to parse with "unknown identifier" errors.
//!
//! The id is a stable XOR of two hashes (wgsl-rs#221):
//!
//! 1. the module's **content**: a canonical text rendering of its IR (the
//!    `static_ir` emission), plus its imports, templates, instantiations, and
//!    module type params, assembled by the macro;
//! 2. the **consuming compilation unit's** identity: the Cargo package
//!    (`CARGO_PKG_NAME` + `CARGO_PKG_VERSION`) plus the target being compiled
//!    (`CARGO_CRATE_NAME`, and `CARGO_BIN_NAME` for binary targets), per the
//!    wgsl-rs#165 rationale.
//!
//! Consequences, compared to the previous per-process counter scheme:
//!
//! * **Deterministic.** The same module source yields the same id in every
//!   build, process, and machine. Adding, removing, or moving *other* `#[wgsl]`
//!   mods no longer shifts this module's id — that was the counter's failure
//!   mode: every later module got a fresh id and its build was invalidated for
//!   nothing.
//! * **Identical content shares an id, and that is fine.** Two byte-identical
//!   modules are interchangeable in source assembly; dedup keeps one, and the
//!   WGSL is the same either way.
//! * **Distinct content yields distinct ids probabilistically** within one
//!   compilation unit (64-bit FNV-1a; a collision needs ~2^-64 per pair). The
//!   old counter was *exactly* unique within a unit; that guarantee is traded
//!   for determinism, and exactness now comes from content identity rather than
//!   allocation order.
//! * **Across units**, ids are separated by hashing each unit's identity,
//!   closing the hole where byte-identical modules in different crates would
//!   dedup-drop each other while their `crate::`-relative imports resolve to
//!   different sources. Including the package version also separates two
//!   semver- incompatible copies of one package, which cargo can legally
//!   compile into one binary.
//!
//! Known residual edge: two targets of one package whose
//! `CARGO_CRATE_NAME` values alias (e.g. an integration test named
//! exactly after the lib crate) share an identity; stable cargo
//! provides no target-kind environment variable to discriminate them.
//!
//! Outside cargo these environment variables are absent and ids degrade
//! to pure content hashes: deterministic, but with no cross-unit
//! separation (identical content in different non-cargo units will
//! share an id, deduping to one interchangeable copy).

/// FNV-1a 64-bit hash, streamed over the given byte slices.
///
/// Chosen because it is small and *stable*: ids are baked into each
/// crate at its own compile time, so the hash must not vary between
/// rustc versions (`DefaultHasher` gives no such guarantee).
fn fnv1a64(slices: &[&[u8]]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &slice in slices {
        for &byte in slice {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    hash
}

/// Stable hash of the consuming compilation unit's identity.
///
/// NUL separators keep identity tuples like ("ab", "1.0", ...) and
/// ("ab1", ".0", ...) from hashing to the same stream.
fn unit_identity_hash(pkg_name: &str, pkg_version: &str, crate_name: &str, bin_name: &str) -> u64 {
    fnv1a64(&[
        pkg_name.as_bytes(),
        b"\0",
        pkg_version.as_bytes(),
        b"\0",
        crate_name.as_bytes(),
        b"\0",
        bin_name.as_bytes(),
    ])
}

/// Derive the module id for a module with the given content descriptor,
/// expanded in the compilation unit identified by `pkg_name` /
/// `pkg_version` / `crate_name` / `bin_name`.
///
/// Pure, so determinism and collision properties are unit-testable
/// directly; [`deterministic_module_id`] reads the environment and
/// delegates here.
fn module_id_for(
    pkg_name: &str,
    pkg_version: &str,
    crate_name: &str,
    bin_name: &str,
    descriptor: &str,
) -> u64 {
    unit_identity_hash(pkg_name, pkg_version, crate_name, bin_name)
        ^ fnv1a64(&[descriptor.as_bytes()])
}

/// The id for the `#[wgsl]` module whose canonical content descriptor
/// is `descriptor`, expanded in this process's consuming compilation
/// unit.
///
/// Reads the unit's identity from the cargo-provided environment
/// (`CARGO_PKG_NAME`, `CARGO_PKG_VERSION`, `CARGO_CRATE_NAME`,
/// `CARGO_BIN_NAME`). Outside cargo these are absent and ids degrade to
/// pure content hashes: deterministic, with no cross-unit separation.
///
/// Deterministic by construction: the same descriptor and environment
/// produce the same id across processes and builds, unlike the
/// per-process counter this replaces (wgsl-rs#221).
pub(crate) fn deterministic_module_id(descriptor: &str) -> u64 {
    let pkg_name = std::env::var("CARGO_PKG_NAME").unwrap_or_default();
    let pkg_version = std::env::var("CARGO_PKG_VERSION").unwrap_or_default();
    let crate_name = std::env::var("CARGO_CRATE_NAME").unwrap_or_default();
    let bin_name = std::env::var("CARGO_BIN_NAME").unwrap_or_default();
    module_id_for(&pkg_name, &pkg_version, &crate_name, &bin_name, descriptor)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FNV-1a is pinned by reference vectors so an accidental algorithm
    /// change — which would silently re-assign every module id — can't
    /// slip through unnoticed.
    #[test]
    fn fnv1a64_matches_reference_vectors() {
        assert_eq!(fnv1a64(&[b""]), 0xcbf29ce484222325);
        assert_eq!(fnv1a64(&[b"a"]), 0xaf63dc4c8601ec8c);
        assert_eq!(fnv1a64(&[b"foobar"]), 0x85944171f73967e8);
    }

    /// Determinism is the whole point (wgsl-rs#221): the same content
    /// in the same unit must produce the same id on every call, in
    /// every process. The old per-process counter could not do this.
    #[test]
    fn ids_are_deterministic_for_identical_content() {
        let a = module_id_for("my-game", "0.1.0", "my_game", "", "content");
        let b = module_id_for("my-game", "0.1.0", "my_game", "", "content");
        assert_eq!(a, b, "identical content must yield the identical id");
    }

    /// Two modules with *different content* in the *same* compilation
    /// unit must get different ids — probabilistically now (64-bit
    /// hash), where the counter scheme was exactly unique. A collision
    /// requires ~2^-64.
    #[test]
    fn ids_differ_within_a_compilation_unit() {
        let a = module_id_for("my-game", "0.1.0", "my_game", "", "content-a");
        let b = module_id_for("my-game", "0.1.0", "my_game", "", "content-b");
        assert_ne!(a, b, "distinct content must not collide");
    }

    /// The bug from wgsl-rs#165: identical content 0 of crate
    /// `shader-a` and crate `shader-b` must not share an id, or one
    /// import is dropped as a "duplicate". Package identity keeps them
    /// apart.
    #[test]
    fn ids_differ_across_crates() {
        let a = module_id_for("shader-a", "0.1.0", "shader_a", "", "same-content");
        let b = module_id_for("shader-b", "0.1.0", "shader_b", "", "same-content");
        assert_ne!(
            a, b,
            "identical content in different crates must not collide"
        );
    }

    /// The same-package/different-target hole called out in review of
    /// PR #173: a package's lib target and one of its integration-test
    /// or bin targets share `CARGO_PKG_NAME`/`CARGO_PKG_VERSION`, and
    /// identical content must still not share an id. The crate/bin name
    /// discriminators keep them apart.
    #[test]
    fn ids_differ_across_targets_of_same_package() {
        // Lib target of package `id-collision-app`: crate name matches
        // the package, no bin involved.
        let lib = module_id_for(
            "id-collision-app",
            "0.1.0",
            "id_collision_app",
            "",
            "content",
        );
        // An integration-test target compiled from
        // tests/target_collision.rs: crate name is the test's stem.
        let test = module_id_for(
            "id-collision-app",
            "0.1.0",
            "target_collision",
            "",
            "content",
        );
        // The package's own bin: `CARGO_CRATE_NAME` equals the lib's,
        // so `CARGO_BIN_NAME` is what distinguishes it.
        let bin = module_id_for(
            "id-collision-app",
            "0.1.0",
            "id_collision_app",
            "id-collision-app",
            "content",
        );
        assert_ne!(
            lib, test,
            "lib and integration-test targets must not collide"
        );
        assert_ne!(lib, bin, "lib and bin targets must not collide");
        assert_ne!(test, bin, "test and bin targets must not collide");
    }

    /// Sweep a realistic id space — the first 16 distinct contents of
    /// each of three crates, pairwise — so no sampled (crate, content)
    /// combination coincides. Cross-identity uniqueness is inherently
    /// probabilistic; this pins it for the sampled space.
    #[test]
    fn ids_differ_across_crates_for_many_contents() {
        for a in 0..16u64 {
            for b in 0..16u64 {
                for (left, right) in [
                    ("shader-a", "shader-b"),
                    ("shader-a", "shader-c"),
                    ("shader-b", "shader-c"),
                ] {
                    assert_ne!(
                        module_id_for(left, "0.1.0", left, "", &format!("content-{a}")),
                        module_id_for(right, "0.1.0", right, "", &format!("content-{b}")),
                        "{left} #{a} vs {right} #{b}"
                    );
                }
            }
        }
    }

    /// Two semver-incompatible versions of the same package can coexist
    /// in one build graph; the version must keep identical content in
    /// them apart.
    #[test]
    fn ids_differ_across_versions_of_same_crate() {
        let old = module_id_for("shader-lib", "1.0.0", "shader_lib", "", "content");
        let new = module_id_for("shader-lib", "2.0.0", "shader_lib", "", "content");
        assert_ne!(
            old, new,
            "identical content in different versions must not collide"
        );
    }

    /// Identity tuples whose concatenated byte streams would alias —
    /// ("ab", "1.0", ...) vs ("ab1", ".0", ...) — must hash
    /// differently; the NUL separators keep them distinct.
    #[test]
    fn identity_tuples_dont_alias() {
        assert_ne!(
            module_id_for("ab", "1.0", "ab", "", "content"),
            module_id_for("ab1", ".0", "ab1", "", "content")
        );
    }
}
