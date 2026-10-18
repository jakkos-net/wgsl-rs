//! Subcommand that measures incremental compile time of big `#[wgsl]`
//! shader mods (wgsl-rs#221).
//!
//! This is a build-time regression bench, not a runtime benchmark. It
//! measures what a shader author feels: the wall-clock cost of
//! `cargo build` after touching a crate that contains a large
//! `#[wgsl]` mod. The measured crate is `compile-bench`, which vendors
//! the ~900-line psrdnoise3 port from the mvp repo linked in the
//! issue; at `opt-level=1` its ~12,000-line `__wgsl_module_ctor`
//! expansion cost ~14s of LLVM time per rebuild (legacy mode).
//!
//! The bench measures **both macro emission paths**:
//!
//! * `legacy`: `--no-default-features` builds the constructor-function
//!   emission, the pre-fix codegen (kept as the rollback off-state of the
//!   default-on `fast-compile` feature).
//! * `fast`: the default build, with the IR emitted as static `static_ir` data
//!   plus a slim conversion shim.
//!
//! Per mode and mutation mode, the bench runs:
//!
//! 1. one first build (the cold number if the cache is empty)
//! 2. a few iterations: mutate the shader source deterministically, then time
//!    an isolated `cargo build -p compile-bench`
//! 3. restores the original source, leaving the tree clean
//!
//! Two mutation modes, straight from the issue's retrigger list:
//!
//! * `inside`: rewrite a literal in the `BENCH_TICK` anchor const inside the
//!   mod. The edit keeps byte length (no span shift) but changes tokens, so the
//!   proc-macro expansion fingerprint changes and the crate fully recompiles. A
//!   comment-only edit would leave the token stream identical and rustc's
//!   incremental fingerprints would skip the work we want to measure.
//! * `above`: insert a comment line at the top of the file on odd iterations
//!   and remove it on even ones. Inserting a line shifts the spans of
//!   everything below, including the mod, invalidating the expansion even
//!   though its tokens are unchanged.
//!
//! Builds are pinned to isolated `-p compile-bench` invocations so
//! workspace-wide feature unification cannot blur the emission modes
//! (one crate keeping wgsl-rs's defaults turns `fast-compile` on for
//! the whole graph). The driver sets
//! `CARGO_PROFILE_DEV_OPT_LEVEL=1` for its own invocations only: the
//! issue reproduces at `opt-level=1` and the workspace root dev
//! profile stays untouched.
//!
//! Timings are machine-dependent and noisy, so this is a relative
//! tool (baseline versus after a fix, same machine). CI only checks
//! that the bench crate compiles.

use std::{path::PathBuf, time::Instant};

/// Incremental rebuilds measured per mutation mode and emission mode.
const ITERATIONS: usize = 3;

/// Anchor const in `crates/compile-bench/src/main.rs` whose literal the
/// `inside` mode rewrites between iterations. Keep in sync with the
/// bench shader.
const TICK_CONST: &str = "pub const BENCH_TICK: f32 = ";

/// Line the `above` mode inserts at the top of the bench shader on odd
/// iterations (and whose absence defines even iterations).
const SPACER_LINE: &str = "// __bench_spacer__";

#[derive(clap::Subcommand)]
pub enum BenchAction {
    /// Time first and incremental rebuilds of `compile-bench` in both
    /// emission modes.
    Run,
}

impl BenchAction {
    pub fn run(&self) {
        let shader = shader_path();
        let original =
            std::fs::read_to_string(&shader).expect("could not read the bench shader source");
        println!("== wgsl-rs compile bench (issue #221) ==");
        println!("shader: {}", shader.display());
        println!("profile: dev with opt-level=1 (scoped to this driver)");
        println!();

        for bench_mode in [BenchMode::Legacy, BenchMode::Fast] {
            println!("== {} emission ==", bench_mode.name());
            let Some(first) = timed_build(bench_mode) else {
                restore(&shader, &original);
                exit_failed();
            };
            println!("  first build (cold if cache empty): {first:.2}s");

            for mutation_mode in [MutationMode::Inside, MutationMode::Above] {
                println!(
                    "  {} mutation mode ({}):",
                    mutation_mode.name(),
                    mutation_mode.description()
                );
                let mut times = Vec::new();
                for iteration in 1..=ITERATIONS {
                    let mutated = mutate(&original, mutation_mode, iteration);
                    std::fs::write(&shader, mutated)
                        .expect("could not write the mutated shader source");
                    let Some(t) = timed_build(bench_mode) else {
                        restore(&shader, &original);
                        exit_failed();
                    };
                    times.push(t);
                    println!("    iteration {iteration}: {t:.2}s");
                }
                println!("    median incremental: {:.2}s", median(&times));
            }
        }

        restore(&shader, &original);
        println!("\nshader source restored; tree is clean");
    }
}

/// Which macro emission path the bench crate compiles with.
#[derive(Clone, Copy)]
enum BenchMode {
    /// The default-on `fast-compile` feature: static `static_ir` data
    /// plus a slim conversion shim.
    Fast,
    /// `--no-default-features`: the legacy constructor-function
    /// emission (the rollback off-state).
    Legacy,
}

impl BenchMode {
    fn name(self) -> &'static str {
        match self {
            BenchMode::Fast => "fast (static IR data)",
            BenchMode::Legacy => "legacy (constructor fn)",
        }
    }
}

/// Mutation modes, both from the issue's retrigger list.
#[derive(Clone, Copy)]
enum MutationMode {
    /// Rewrite a literal inside the `#[wgsl]` mod (macro input changes).
    Inside,
    /// Insert/remove a line above the mod (spans of everything below
    /// shift, even though the macro input tokens are equal).
    Above,
}

impl MutationMode {
    fn name(self) -> &'static str {
        match self {
            MutationMode::Inside => "inside",
            MutationMode::Above => "above",
        }
    }

    fn description(self) -> &'static str {
        match self {
            MutationMode::Inside => "rewrite a literal inside the #[wgsl] mod",
            MutationMode::Above => "insert/remove a comment line at the top of the file",
        }
    }
}

/// Deterministically mutate the bench shader for one iteration.
fn mutate(original: &str, mode: MutationMode, iteration: usize) -> String {
    match mode {
        MutationMode::Inside => {
            let start = original
                .find(TICK_CONST)
                .expect("bench shader is missing the BENCH_TICK anchor const");
            let value_start = start + TICK_CONST.len();
            let digit_len = original[value_start..]
                .chars()
                .next()
                .map(char::len_utf8)
                .expect("BENCH_TICK literal is empty");
            // A single digit keeps the byte length stable, so only the
            // token stream changes and no spans shift.
            assert!(iteration < 10, "iteration must stay a single digit");
            let mut mutated = original.to_string();
            mutated.replace_range(value_start..value_start + digit_len, &iteration.to_string());
            mutated
        }
        MutationMode::Above => {
            if iteration % 2 == 1 {
                format!("{SPACER_LINE}\n{original}")
            } else {
                original.to_string()
            }
        }
    }
}

/// Time one isolated `cargo build -p compile-bench` at `opt-level=1`,
/// in the given emission mode.
fn timed_build(mode: BenchMode) -> Option<f64> {
    let start = Instant::now();
    let mut cmd = std::process::Command::new("cargo");
    cmd.args(["build", "-p", "compile-bench"]);
    if matches!(mode, BenchMode::Legacy) {
        cmd.arg("--no-default-features");
    }
    let status = cmd
        .env("CARGO_PROFILE_DEV_OPT_LEVEL", "1")
        .status()
        .expect("could not spawn cargo");
    let elapsed = start.elapsed().as_secs_f64();
    if status.success() {
        Some(elapsed)
    } else {
        None
    }
}

/// Restore the original shader source.
fn restore(shader: &std::path::Path, original: &str) {
    std::fs::write(shader, original).expect("could not restore the shader source");
}

/// Median of the measured rebuild times.
fn median(times: &[f64]) -> f64 {
    let mut sorted = times.to_vec();
    sorted.sort_by(f64::total_cmp);
    sorted[sorted.len() / 2]
}

/// Path to the bench shader source, resolved from the workspace root
/// exported by `.cargo/config.toml`.
fn shader_path() -> PathBuf {
    let root = std::env::var("CARGO_WORKSPACE_DIR")
        .expect("CARGO_WORKSPACE_DIR must be set via .cargo/config.toml");
    PathBuf::from(root).join("crates/compile-bench/src/main.rs")
}

/// Fail the bench after cargo already printed its error output.
fn exit_failed() -> ! {
    eprintln!("bench build failed; see cargo output above");
    std::process::exit(1);
}
