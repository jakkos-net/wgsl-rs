use wgsl_rs::wgsl;

#[wgsl]
mod enum_default_generic_entry_point_collision {
    use wgsl_rs::std::*;

    #[derive(Default)]
    #[repr(u32)]
    pub enum Kind {
        #[default]
        Alpha,
        Beta,
    }

    // Generic entry points keep their original, unsuffixed WGSL name,
    // colliding with the `Kind_default` method synthesized for the
    // `#[default]` enum above.
    #[compute]
    #[workgroup_size(8)]
    pub fn Kind_default<T: Wgsl>(#[builtin(global_invocation_id)] _gid: Vec3u) {}
}

fn main() {}