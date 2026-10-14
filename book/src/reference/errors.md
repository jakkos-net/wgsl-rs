# Errors

## `SourceError`

Raised by `wgsl_rs::Source` methods.

| Variant        | When                                                                  |
|----------------|-----------------------------------------------------------------------|
| `TemplateWgsl` | `wgsl_source()` is called on a `Source` that is still a template (has unresolved `TypeParam` nodes). Call `instantiate(...)` first to produce concrete WGSL. |

### Recovery

```rust
match source.wgsl_source() {
    Ok(wgsl) => { /* write or compile */ }
    Err(SourceError::TemplateWgsl) => {
        let concrete = source.instantiate(&[concrete_type])?;
        let wgsl = concrete.wgsl_source()?;
    }
}
```

## `wgsl_rs_linkage_wgpu::Error`

Raised by the `wgsl-rs-linkage-wgpu` crates when analyzing a source or
creating a bind group by name.

| Variant                        | When                                                          |
|--------------------------------|---------------------------------------------------------------|
| `TemplateResolution { .. }`    | Analysis was requested for a template source that has not been instantiated with concrete types. |
| `NoSuchBindGroup { .. }`       | `create_bind_group_named` was called for a `@group(N)` the module does not declare. |
| `MissingBinding { .. }`        | A named-resource bind group creation is missing a resource for one of the group's declared binding names. |
| `UnknownBindingName { .. }`    | A named-resource creation provided a resource for a name the group does not declare. |
| `DuplicateBindingName { .. }`  | A named-resource creation provided the same binding name twice. |

All variants are struct variants; match them with `{ .. }` patterns.

### Recovery

Bind group *lookups* return `Option`, not `Result` (`linkage.bind_group(0)`),
so recovery there is ordinary `Option` handling. Only analysis and
named-resource bind group creation return `Error`:

```rust
use wgsl_rs_linkage_wgpu::{analyze_ir_module, analyze_wgsl_module, Error};

// Analysis is the only source of `TemplateResolution`: instantiate the
// template with concrete types first, then analyze the concrete IR.
let linkage = match analyze_wgsl_module(&my_shader::WGSL_SOURCE) {
    Ok(linkage) => linkage,
    Err(Error::TemplateResolution { .. }) => {
        // The source is a template: instantiate it with concrete types,
        // then analyze the concrete IR instead.
        analyze_ir_module(my_shader::instantiate::<f32, u32>())
    }
    Err(other) => return Err(other),
};

// Named-resource bind group creation surfaces the binding-name variants:
match linkage.create_bind_group_named(0, &device, &[("FRAME", binding)]) {
    Ok(bind_group) => { /* bind it */ }
    Err(Error::MissingBinding { .. }) => {
        // a declared binding had no resource provided
    }
    Err(_) => { /* NoSuchBindGroup, UnknownBindingName, DuplicateBindingName */ }
}
```

## General Pattern

Template errors are recoverable by instantiating with concrete types (see [Template Modules & Instantiation](../generics/templates.md) and [Template Linkage](../linkage/template-linkage.md)). Bind group / binding errors indicate a mismatch between the host-side layout code and the shader declaration — inspect `ir::Module` items (`Uniform`, `Storage`, `Sampler`, `Texture`) to reconcile.