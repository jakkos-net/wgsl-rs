use wgsl_rs::wgsl;

#[wgsl]
mod enum_default_collision {
    #[derive(Default)]
    #[repr(u32)]
    pub enum Kind {
        #[default]
        Alpha,
        Beta,
    }

    // Collides with the `Kind_default` method synthesized for the
    // `#[default]` enum above.
    pub fn Kind_default() -> u32 {
        0u32
    }
}

fn main() {}