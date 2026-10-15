//! Subcommand that performs CI actions.

use crate::help::*;

/// The nightly pinned for formatting.
///
/// `rustfmt.toml` uses nightly-only options, and a floating nightly
/// drifts (a September 2026 nightly changed comment wrapping, failing
/// every open PR on files its PR never touched). The workspace toolchain
/// is stable (see rust-toolchain.toml), so formatting selects this
/// nightly explicitly. CI's fmt job reads this constant from the source.
/// Bump deliberately, then reformat the workspace with
/// `cargo +nightly-YYYY-MM-DD fmt --all` and commit the diff.
pub const PINNED_FMT_NIGHTLY: &str = "nightly-2026-08-22";

#[derive(clap::Subcommand)]
pub enum CiAction {
    /// Run unit tests with `nextest`.
    UnitTests,
    /// Run roundtrip tests.
    RoundtripTests,
    /// Run documentation tests.
    DocTests,
    /// Run the wgpu-linkage crate feature matrix.
    LinkageMatrix,
    /// Run all tests.
    AllTests,
    /// Install clippy CI deps.
    InstallClippyDeps,
    /// Run clippy lints.
    Clippy,
    /// Run cargo fmt.
    Fmt,
    /// Run cargo doc generation and err on any warnings.
    Docs,
    /// Run a pull-request check, including:
    /// * clippy lints
    /// * formatting
    /// * tests
    /// * documentation
    PrCheck,
}

impl CiAction {
    pub fn ensure_cargo_nextest() {
        if !does_binary_exist("cargo-nextest") {
            log::info!("Installing cargo-nextest...");
            cmd_capture("cargo", "install --locked cargo-nextest")
                .ensure_success("Could not install cargo-nextest");
            log::info!("...done!");
        }
    }

    pub fn ensure_clippy_sarif() {
        if !does_binary_exist("clippy-sarif") {
            log::info!("Installing clippy-sarif");
            cmd_capture("cargo", "install clippy-sarif")
                .ensure_success("Could not install clippy-sarif");
        }
        if !does_binary_exist("sarif-fmt") {
            log::info!("Installing sarif-fmt");
            cmd_capture("cargo", "install sarif-fmt").ensure_success("Could not install sarif-fmt");
        }
    }

    pub fn run(&self) {
        match self {
            CiAction::UnitTests => {
                Self::ensure_cargo_nextest();
                cmd("cargo", "nextest run --all-features");
            }
            CiAction::RoundtripTests => {
                cmd("cargo", "run -p roundtrip-tests -- --list");
                cmd("cargo", "run -p roundtrip-tests");
            }
            CiAction::DocTests => cmd("cargo", "test --doc --all-features"),
            CiAction::LinkageMatrix => {
                // The linkage crate compiles one shared adapter
                // (`src/linkage.rs`) into feature-gated per-major modules,
                // so there is no template copy to keep in sync. wgpu
                // majors do not interop, so every buildable feature
                // combination must stay green, and each combination must
                // resolve exactly the wgpu major(s) it asks for.
                for (args, expected) in [
                    ("", vec![30]),
                    ("--no-default-features --features v29,validation", vec![29]),
                    (
                        "--no-default-features --features v29,v30,validation",
                        vec![29, 30],
                    ),
                    ("--no-default-features", vec![]),
                ] {
                    let found = wgpu_majors_in_tree(args);
                    assert_eq!(
                        found, expected,
                        "unexpected wgpu majors for feature combination `{args}`"
                    );
                }
                cmd("cargo", "test -p wgsl-rs-linkage-wgpu");
                cmd(
                    "cargo",
                    "test -p wgsl-rs-linkage-wgpu --no-default-features --features v29,validation",
                );
                cmd(
                    "cargo",
                    "test -p wgsl-rs-linkage-wgpu --no-default-features --features \
                     v29,v30,validation",
                );
                // Bare build: the wgpu-free analysis core only.
                cmd(
                    "cargo",
                    "test -p wgsl-rs-linkage-wgpu --no-default-features",
                );
            }
            CiAction::AllTests => {
                CiAction::UnitTests.run();
                CiAction::RoundtripTests.run();
                CiAction::DocTests.run();
                CiAction::LinkageMatrix.run();
            }
            CiAction::InstallClippyDeps => Self::ensure_clippy_sarif(),
            CiAction::Clippy => cmd(
                "cargo",
                "clippy --all-features --all-targets -- -D warnings",
            ),

            CiAction::Fmt => {
                // Auto-install of the pinned nightly uses the configured
                // rustup profile, which can lack `rustfmt` (CI runner
                // images default to the minimal profile). Add the
                // component explicitly, or install the whole toolchain if
                // this is its first use.
                if does_binary_exist("rustup") {
                    let added = cmd_capture(
                        "rustup",
                        format!("component add --toolchain {PINNED_FMT_NIGHTLY} rustfmt"),
                    );
                    if !added.status.success() {
                        cmd(
                            "rustup",
                            format!(
                                "toolchain install {PINNED_FMT_NIGHTLY} --profile minimal \
                                 --component rustfmt"
                            ),
                        );
                    }
                }
                cmd(
                    "cargo",
                    format!("+{PINNED_FMT_NIGHTLY} fmt --all -- --check"),
                );
            }
            CiAction::Docs => cmd_with_env(
                "cargo",
                "doc --all-features --no-deps",
                Some(("RUSTDOCFLAGS", "-D warnings")),
            ),
            CiAction::PrCheck => {
                CiAction::Fmt.run();
                CiAction::Clippy.run();
                CiAction::Docs.run();
                CiAction::AllTests.run();
            }
        }
    }
}

/// The wgpu major versions present in `cargo tree` output for
/// `wgsl-rs-linkage-wgpu`, built with the given feature arguments.
///
/// Used by the linkage-matrix check to assert that each feature
/// combination resolves exactly the wgpu major(s) it asks for.
fn wgpu_majors_in_tree(feature_args: &str) -> Vec<u32> {
    let out = cmd_capture(
        "cargo",
        format!("tree -p wgsl-rs-linkage-wgpu {feature_args}"),
    );
    out.ensure_success("could not run cargo tree for the linkage crate");
    let mut majors: Vec<u32> = out
        .stdout
        .lines()
        .filter_map(|line| {
            let (_, version) = line.split_once(" wgpu v")?;
            let major = version.split('.').next()?;
            major.parse().ok()
        })
        .collect();
    majors.sort_unstable();
    majors.dedup();
    majors
}
