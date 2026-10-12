//! Tests for enum `#[default]` handling and impls on enums.
//!
//! `#[default]` variant attributes are preserved in the IR, and an enum
//! carrying `#[default]` gets a synthesized `default()` method — so
//! `Kind::default()` works in both worlds: the GPU renders the mangled
//! `Kind_default()` call, and on the CPU the expression resolves
//! through the `Default` trait impl. The macro-generated
//! `__validate_wgsl` tests confirm the rendered WGSL (including the
//! synthesized method and its call sites) is naga-valid.

#![allow(dead_code)]

use wgsl_rs::wgsl;

#[wgsl]
mod enum_default {
    #[derive(Debug, Default, PartialEq)]
    #[repr(u32)]
    pub enum Kind {
        #[default]
        Alpha,
        Beta,
    }

    /// Uses the synthesized default: `Kind::default()` renders the
    /// mangled call on the GPU and resolves through the `Default`
    /// trait on the CPU.
    pub fn fallback() -> Kind {
        Kind::default()
    }

    /// Explicit impls on enum self types: `Self` resolves to the enum
    /// name and the method renders mangled against the enum's `u32`
    /// alias.
    pub fn use_explicit_impl() -> Kind {
        Kind::or_default()
    }

    impl Kind {
        pub fn or_default() -> Self {
            Self::Alpha
        }
    }
}

#[test]
fn default_impl_is_synthesized() {
    let src = enum_default::WGSL_SOURCE.wgsl_source().unwrap();
    // `default` contains no underscore, so the mangled name takes the
    // single-underscore form.
    assert!(
        src.contains("fn Kind_default() -> Kind"),
        "expected a synthesized Kind_default, got: {src}"
    );
    assert!(
        src.contains("return Kind_Alpha;"),
        "expected the default variant's const as the body, got: {src}"
    );
}

#[test]
fn default_call_site_renders_mangled() {
    let src = enum_default::WGSL_SOURCE.wgsl_source().unwrap();
    assert!(
        src.contains("return Kind_default();"),
        "expected Kind::default() to render as the mangled call, got: {src}"
    );
}

#[test]
fn impl_on_enum_renders_mangled() {
    let src = enum_default::WGSL_SOURCE.wgsl_source().unwrap();
    // `Self` in the impl resolves to `Kind`, and the method's mangled
    // name is `Kind__1or_default`.
    assert!(
        src.contains("fn Kind__1or_default() -> Kind"),
        "expected the impl-on-enum method to render mangled, got: {src}"
    );
    assert!(
        src.contains("return Kind_Alpha;"),
        "expected `Self::Alpha` to resolve to the variant const, got: {src}"
    );
}

#[test]
fn cpu_side_resolves_through_the_default_trait() {
    use enum_default::Kind;
    // On the CPU the module is plain Rust: `Kind::default()` goes
    // through `Default` (the derive-generated impl) and the explicit
    // impl's `Self` resolution makes this the same value.
    assert_eq!(Kind::Alpha, enum_default::fallback());
    assert_eq!(Kind::Alpha, enum_default::use_explicit_impl());
}

#[test]
fn runtime_ir_preserves_variant_attrs() {
    // The rendered WGSL is attrs-invisible by design, so the render
    // tests cannot catch a regression in the attribute plumbing. This
    // asserts on the runtime IR — the module extensions like
    // `SlabItemExt` actually receive — exercising the full pipeline:
    // parse attrs -> ir_convert -> ir_emit tokens -> constructor.
    let module = (enum_default::WGSL_SOURCE.ir_constructor)();
    let e = module
        .items
        .iter()
        .find_map(|item| match item {
            wgsl_rs::ir::Item::Enum(e) if e.name == "Kind" => Some(e),
            _ => None,
        })
        .expect("Kind enum in the runtime IR");
    assert_eq!(
        1,
        e.variants[0].attrs.len(),
        "expected the #[default] attr on the Alpha variant"
    );
    assert_eq!("default", e.variants[0].attrs[0].path);
    assert!(e.variants[0].attrs[0].args.is_empty());
    assert!(e.variants[1].attrs.is_empty());

    // The synthesized default() method rides along in the same IR.
    let has_default = module.items.iter().any(|item| match item {
        wgsl_rs::ir::Item::Impl(i) => {
            i.self_ty == "Kind"
                && i.items
                    .iter()
                    .any(|m| matches!(m, wgsl_rs::ir::ImplItem::Fn(f) if f.name == "default"))
        }
        _ => false,
    });
    assert!(has_default, "expected the synthesized Kind default() impl");
}
