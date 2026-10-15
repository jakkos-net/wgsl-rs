# Cargo Features

## `wgsl-rs`

| Feature             | Default? | Description                                                                |
|---------------------|----------|----------------------------------------------------------------------------|
| `validation`        | Yes      | Compile-time WGSL validation of generated source                           |
| `dispatch-runtime`  | Yes      | Enable runtime dispatch support                                            |

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

## Usage

```toml
[dependencies]
wgsl-rs = { version = "0.1", default-features = false, features = ["validation"] }
wgsl-rs-linkage-wgpu = "0.1" # default: latest wgpu major
wgsl-rs-layout = { version = "0.1", features = ["doc-diagrams"] }
```