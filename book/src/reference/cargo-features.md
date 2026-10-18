# Cargo Features

## `wgsl-rs`

| Feature             | Default? | Description                                                                |
|---------------------|----------|----------------------------------------------------------------------------|
| `validation`        | Yes      | Compile-time WGSL validation of generated source                           |
| `dispatch-runtime`  | Yes      | Enable runtime dispatch support                                            |
| `fast-compile`      | Yes      | Emit each module's IR as static data plus a tiny conversion shim, instead of a runtime constructor function; keeps incremental rebuilds of big `#[wgsl]` modules fast |

## `wgsl-rs-linkage-wgpu`

| Feature | Default? | Description                                                          |
|---------|----------|----------------------------------------------------------------------|
| `v29`   | No       | Linkage compiled against wgpu 29 (module `wgsl_rs_linkage_wgpu::v29`) |
| `v30`   | Yes      | Linkage compiled against wgpu 30 (module `wgsl_rs_linkage_wgpu::v30`, also re-exported at the crate root) |
| `validation` | Yes  | Forwards to `wgsl-rs`'s `validation` feature (naga WGSL validation) |

## `wgsl-rs-layout`

| Feature         | Default? | Description                                                       |
|-----------------|----------|-------------------------------------------------------------------|
| `doc-diagrams`  | No       | Enable `generate_svg` for byte-layout SVG diagrams                |

## Emission Modes: `fast-compile`

`fast-compile` (on by default) makes the `#[wgsl]` macro emit a
module's IR as static data together with a small function that
converts it when needed. Without the feature the macro emits a
constructor function that builds the IR step by step; for large
shader modules that constructor is enormous and slows down every
incremental rebuild.

Opting out is possible, but only for the whole build graph:

```toml
[dependencies]
wgsl-rs = { version = "0.1", default-features = false, features = ["validation", "dispatch-runtime"] }
```

Cargo unifies features across the dependency graph: if *any* crate in
your build keeps `wgsl-rs`'s default features, `fast-compile` stays on
for every crate in that build. Opting out therefore requires every
crate in the graph to opt out. Libraries should not force either
mode; let the final application choose.

## Usage

```toml
[dependencies]
wgsl-rs = { version = "0.1", default-features = false, features = ["validation", "dispatch-runtime"] }
wgsl-rs-linkage-wgpu = "0.1" # default: latest wgpu major
wgsl-rs-layout = { version = "0.1", features = ["doc-diagrams"] }
```