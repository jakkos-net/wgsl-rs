# Cargo Features

`wgsl-rs` exposes two cargo features, and both are on by default.

## `validation` (default)

Enables naga-based validation of generated WGSL. For every non-template `#[wgsl]` module, the macro auto-generates a `#[test] fn __validate_wgsl()` that compiles the emitted `WGSL_SOURCE` through naga and fails on any validation error.

This is the primary safety net: if your Rust module transpiles but the WGSL is malformed, `cargo test` catches it.

Disable it with `default-features = false`:

```toml
[dependencies]
wgsl-rs = { version = "0.1", default-features = false }
```

Use this when you want no naga dependency at all, e.g. in a build that only consumes `WGSL_SOURCE` text and validates downstream.

## `dispatch-runtime`

Enables the CPU-side fragment dispatch runtime at `wgsl_rs::std::runtime`. It lets you run a fragment shader on the CPU over a set of inputs and compare the output against a GPU render. This is the mechanism used for roundtrip testing: the same shader code runs on both sides and the results are diffed.

Enable it explicitly:

```toml
[dependencies]
wgsl-rs = { version = "0.1", features = ["dispatch-runtime"] }
```

Use this when writing tests that exercise fragment shaders without spinning up a full wgpu pipeline, or when debugging shader logic on the CPU.

## Linkage: a separate crate

wgpu linkage generation used to live behind a `linkage-wgpu` feature. It now
lives in the separate `wgsl-rs-linkage-wgpu` crate (compiled against each
supported wgpu major), because the linkage output is wgpu types that do not
interop across majors:

```toml
[dependencies]
wgsl-rs = "0.1"
wgsl-rs-linkage-wgpu = "0.1" # default: latest wgpu major
```

Use this in application crates that render or compute via wgpu. For pure
shader authoring and validation it is not needed. See
[wgpu Linkage Overview](../linkage/overview.md) for pinned-major setup and
the full API.

## Combining features

Features compose freely. A typical application crate leaves `wgsl-rs` at
its defaults (`validation` and `dispatch-runtime` are both on) and adds
the linkage crate:

```toml
[dependencies]
wgsl-rs = "0.1"
wgsl-rs-linkage-wgpu = "0.1"
```

A shader-only library crate strips the defaults it doesn't need:

```toml
[dependencies]
wgsl-rs = { version = "0.1", default-features = false, features = ["validation"] }
```

## Reference

| Feature            | Default | Module path                 | When to use                                     |
| ------------------ | ------- | --------------------------- | ----------------------------------------------- |
| `validation`       | on      | (macro-internal, naga)      | Always, unless you strip naga deliberately.     |
| `dispatch-runtime` | on      | `wgsl_rs::std::runtime`     | CPU-side fragment dispatch and roundtrip tests. |

| Separate crate               | Default        | When to use                                  |
| ---------------------------- | -------------- | -------------------------------------------- |
| `wgsl-rs-linkage-wgpu`       | latest wgpu major | Building wgpu pipelines from shader modules. |