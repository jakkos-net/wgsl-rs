//! wgpu linkage generation for wgsl-rs shader modules.
//!
//! This facade re-exports the linkage implementation matching your wgpu
//! major. It defaults to the latest supported major, exposed at the
//! crate root:
//!
//! ```ignore
//! use wgsl_rs_linkage_wgpu as wgpu_linkage;
//!
//! let linkage = wgpu_linkage::analyze_wgsl_module(&hello_triangle::WGSL_SOURCE).unwrap();
//! let shader_module = linkage.shader_module(&device);
//! ```
//!
//! Pinned to an older major, use the matching module instead:
//!
//! ```ignore
//! // Cargo.toml: wgsl-rs-linkage-wgpu = { version = "0.1", default-features = false, features = ["v29"] }
//! let linkage = wgsl_rs_linkage_wgpu::v29::analyze_wgsl_module(&source).unwrap();
//! ```
//!
//! Both majors may be enabled at once (the modules coexist with
//! distinct, incompatible wgpu types); the crate root always re-exports
//! the latest.
//!
//! Why a facade: linkage output is wgpu types, which do not interop
//! across majors, so each supported major gets its own crate
//! (`wgsl-rs-linkage-wgpu-v29`, `-v30`, ...) compiled against its
//! pinned `wgpu`. `wgsl-rs` itself stays wgpu-free.

/// Linkage compiled against wgpu 29.
#[cfg(feature = "v29")]
pub mod v29 {
    pub use wgsl_rs_linkage_wgpu_v29::*;
}

/// Linkage compiled against wgpu 30.
#[cfg(feature = "v30")]
pub mod v30 {
    pub use wgsl_rs_linkage_wgpu_v30::*;
}

// The crate root always implies the latest enabled major.
#[cfg(feature = "v30")]
pub use v30::*;

#[cfg(all(feature = "v29", not(feature = "v30")))]
pub use v29::*;
