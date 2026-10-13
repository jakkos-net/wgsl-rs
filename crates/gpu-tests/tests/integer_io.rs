//! GPU test for integer inter-stage IO carrying explicit
//! `@interpolate(flat)`.
//!
//! WGSL §13.3.1.4 requires user-defined vertex outputs and fragment
//! inputs of integer type to be declared `@interpolate(flat)`; wgpu 30 /
//! naga 30 no longer imply it. The `wgsl-rs` renderer emits the attribute
//! automatically when the source omits it, and this test proves the full
//! GPU path: render pipeline creation (which validates the inter-stage
//! interface matches) and the flat value arriving at every covered
//! fragment.
//!
//! Requires a GPU (or software rasterizer) — the test is skipped if no
//! adapter is available.

use futures::{
    Future, FutureExt,
    channel::oneshot,
    executor::block_on,
    future::{self, Either},
};
use std::{sync::mpsc, thread, time::Duration};
use wgsl_rs::wgsl;

const WIDTH: u32 = 32;
const HEIGHT: u32 = 32;
const TEXEL_SIZE: u32 = 16; // 4 f32s * 4 bytes each = 16 bytes per pixel (Rgba32Float)

/// Upper bound on any single GPU operation in this test file; see the
/// rationale in `derivative_comparison.rs`.
const GPU_TIMEOUT: Duration = Duration::from_secs(60);

/// Sentinel returned when [`race_timeout`] fires before the work future.
#[derive(Debug)]
struct TimedOut;

/// Races `fut` against a deadline of `dur`.
async fn race_timeout<F, T>(fut: F, dur: Duration) -> Result<T, TimedOut>
where
    F: Future<Output = T> + Send,
{
    let (tx, rx) = oneshot::channel::<()>();
    thread::spawn(move || {
        thread::sleep(dur);
        let _ = tx.send(());
    });
    let work = fut.boxed();
    let timer = rx.fuse();
    match future::select(work, timer).await {
        Either::Left((result, _timer)) => Ok(result),
        Either::Right((_timer_fired, _work)) => Err(TimedOut),
    }
}

/// Creates a headless wgpu device, or returns `None` if no suitable
/// adapter is found or enumeration hangs. Checks that the adapter
/// supports `Rgba32Float` as a render attachment.
fn create_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let result = block_on(race_timeout(
        get_adapter_device_and_queue_inner(),
        GPU_TIMEOUT,
    ));
    match result {
        Ok(Some((_adapter, device, queue))) => Some((device, queue)),
        Ok(None) => None,
        Err(TimedOut) => {
            eprintln!(
                "wgpu adapter/device creation timed out after {GPU_TIMEOUT:?} — skipping GPU test"
            );
            None
        }
    }
}

/// Inner async adapter/device acquisition, raced against a timer by
/// [`create_device`].
async fn get_adapter_device_and_queue_inner() -> Option<(wgpu::Adapter, wgpu::Device, wgpu::Queue)>
{
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .map_err(|e| eprintln!("{e}"))
        .ok()?;

    // Verify Rgba32Float is renderable on this adapter.
    let format_features = adapter.get_texture_format_features(wgpu::TextureFormat::Rgba32Float);
    if !format_features
        .allowed_usages
        .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
    {
        eprintln!("Adapter does not support Rgba32Float as render attachment — skipping GPU test");
        return None;
    }

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .map_err(|e| eprintln!("{e}"))
        .ok()?;

    Some((adapter, device, queue))
}

/// Creates a render target texture with `Rgba32Float` format.
fn create_render_target(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("render_target"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

/// Reads back a texture into a `Vec<[f32; 4]>` (one entry per pixel,
/// row-major).
fn read_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
) -> Vec<[f32; 4]> {
    let bytes_per_row = align_to(width * TEXEL_SIZE, 256);
    let buffer_size = (bytes_per_row * height) as u64;

    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("staging"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("readback"),
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: None,
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    let idx = queue.submit(Some(encoder.finish()));

    match readback_inner(staging, device, idx, width, height, bytes_per_row) {
        Ok(pixels) => pixels,
        Err(reason) => panic!("GPU readback failed: {reason}"),
    }
}

/// Performs the full GPU→CPU readback: map the staging buffer, copy
/// pixels, and unmap. The [`GPU_TIMEOUT`] deadline is passed directly
/// into [`wgpu::Device::poll`].
fn readback_inner(
    staging: wgpu::Buffer,
    device: &wgpu::Device,
    submission_index: wgpu::SubmissionIndex,
    width: u32,
    height: u32,
    bytes_per_row: u32,
) -> Result<Vec<[f32; 4]>, &'static str> {
    let (sender, receiver) = mpsc::channel();
    staging
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
    let poll_result = device.poll(wgpu::PollType::Wait {
        submission_index: Some(submission_index),
        timeout: Some(GPU_TIMEOUT),
    });
    match poll_result {
        Ok(_) => {}
        Err(wgpu::PollError::Timeout) => return Err("GPU readback timed out"),
        Err(_) => return Err("device.poll returned an error"),
    }
    receiver
        .recv()
        .map_err(|_| "buffer mapping channel closed")?
        .map_err(|_| "buffer mapping failed")?;

    let data = staging.slice(..).get_mapped_range();
    let mut pixels = Vec::with_capacity((width * height) as usize);
    for y in 0..height {
        let row_offset = (y * bytes_per_row) as usize;
        for x in 0..width {
            let offset = row_offset + (x * TEXEL_SIZE) as usize;
            let pixel: [f32; 4] =
                *bytemuck::from_bytes(&data[offset..offset + TEXEL_SIZE as usize]);
            pixels.push(pixel);
        }
    }
    drop(data);
    staging.unmap();
    Ok(pixels)
}

/// Align `value` up to the next multiple of `alignment`.
fn align_to(value: u32, alignment: u32) -> u32 {
    (value + alignment - 1) & !(alignment - 1)
}

/// Vertex output / fragment input shared struct with an integer varying.
/// The `material: u32` field at `@location(1)` gets no `#[interpolate]`
/// in the Rust source; the renderer must emit `@interpolate(flat)` for
/// the generated WGSL to be valid (naga 30) and for the inter-stage
/// interface to match at pipeline creation.
#[wgsl]
pub mod integer_varying_shader {
    use wgsl_rs::std::*;

    pub struct Varyings {
        #[builtin(position)]
        pub clip_position: Vec4f,
        #[location(0)]
        pub uv: Vec2f,
        #[location(1)]
        pub material: u32,
    }

    #[vertex]
    pub fn vtx_main(#[builtin(vertex_index)] vertex_index: u32) -> Varyings {
        let x = f32((vertex_index & 1u32) * 2u32) * 2.0 - 1.0;
        let y = f32((vertex_index >> 1u32) * 2u32) * 2.0 - 1.0;
        Varyings {
            clip_position: vec4f(x, y, 0.0, 1.0),
            uv: vec2f((x + 1.0) / 2.0, (y + 1.0) / 2.0),
            material: 7u32,
        }
    }

    #[fragment]
    pub fn frag_main(input: Varyings) -> Vec4f {
        vec4f(input.uv.x, input.uv.y, f32(input.material), 1.0)
    }
}

#[test]
fn integer_varying_renders_flat_on_gpu() {
    let Some((device, queue)) = create_device() else {
        eprintln!("no suitable GPU adapter — skipping integer IO GPU test");
        return;
    };

    // Runtime IR-based wgpu linkage analysis (issue #120).
    let mut linkage =
        wgsl_rs_linkage_wgpu::v29::analyze_wgsl_module(&integer_varying_shader::WGSL_SOURCE)
            .unwrap();
    let module = linkage.shader_module(&device);
    let pipeline_layout = linkage.pipeline_layout(&device, Some("integer_varying"));

    // Pipeline creation validates the inter-stage interface: a missing or
    // mismatched `@interpolate(flat)` on the integer varying is a
    // validation error here.
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("integer_varying"),
        layout: Some(&pipeline_layout),
        vertex: linkage
            .vertex_entry("vtx_main")
            .expect("vtx_main entry present")
            .vertex_state(&module),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(
            linkage
                .fragment_entry("frag_main")
                .expect("frag_main entry present")
                .fragment_state(
                    &module,
                    &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba32Float,
                        blend: None,
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                ),
        ),
        multiview_mask: None,
        cache: None,
    });

    let texture = create_render_target(&device, WIDTH, HEIGHT);
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("integer_varying"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("integer_varying"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        pass.set_pipeline(&pipeline);
        pass.draw(0..3, 0..1);
    }
    queue.submit(Some(encoder.finish()));

    let pixels = read_texture(&device, &queue, &texture, WIDTH, HEIGHT);

    // Every covered fragment must see the flat value 7 in the blue
    // channel; uncovered pixels stay transparent (alpha 0).
    let mut covered = 0;
    for pixel in &pixels {
        if pixel[3] == 1.0 {
            covered += 1;
            assert_eq!(
                pixel[2], 7.0,
                "flat integer varying should reach every fragment unchanged"
            );
        }
    }
    assert!(
        covered > pixels.len() / 4,
        "expected the triangle to cover a meaningful portion of the target, got {covered}/{} \
         covered pixels",
        pixels.len()
    );
}
