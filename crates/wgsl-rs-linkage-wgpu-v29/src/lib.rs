//! wgpu 29 linkage generation for wgsl-rs shader modules.
//!
//! This crate compiles the canonical linkage implementation (owned by
//! the latest version crate, `wgsl-rs-linkage-wgpu-v30`) against wgpu
//! 29. The source is shared via `#[path]`, so both majors build from
//! one implementation; this crate would fork a local copy only if
//! wgpu 29 and 30 ever diverge in the linkage API surface.
//!
//! See the `wgsl-rs-linkage-wgpu-v30` documentation for the API. The
//! wgpu 29 types are re-exported as [`wgpu`] so everything the linkage
//! produces can be imported from one path.

/// The canonical linkage implementation, compiled against this crate's
/// pinned `wgpu` dependency.
#[path = "../../wgsl-rs-linkage-wgpu-v30/src/lib.rs"]
mod imp;

pub use imp::*;
