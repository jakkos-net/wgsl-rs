use wgsl_rs::wgsl;

#[wgsl]
mod enum_default_impl_const_collision {
    #[derive(Default)]
    #[repr(u32)]
    pub enum Kind {
        #[default]
        Alpha,
        Beta,
    }

    // The associated const mangles to `Kind_default`, colliding with
    // the method synthesized for the `#[default]` enum above.
    impl Kind {
        pub const default: u32 = 0u32;
    }
}

fn main() {}