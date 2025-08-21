use std::time::{Duration, Instant};

use anyhow::Context;
use image::DynamicImage;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    wgt::CommandEncoderDescriptor,
};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GlobalParams {
    time: f32,
    _padding: [f32; 3], // Padding to match the alignment requirements (16 bytes)
}

pub fn render_holofoil(rs: super::RenderState, width: u32, height: u32) {
    use eframe::wgpu::*;
    let super::RenderState { device, queue, .. } = rs;

    let texture_size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let output_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Output Texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    let output_texture_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let global_params = GlobalParams {
        time: 0.0,
        _padding: [0.0; 3],
    };

    let global_params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Global Params Buffer"),
        contents: bytemuck::cast_slice(&[global_params]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let output_buffer_size = (texture_size.width * texture_size.height * 4) as wgpu::BufferAddress;
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Output Buffer"),
        size: output_buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let shader_module =
        device.create_shader_module(include_wgsl!("../../../assets/holofoil-vornoi.wgsl"));

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Compute Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Compute Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader_module,
        entry_point: Some("imageMain"),
        compilation_options: Default::default(),
        cache: None,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: global_params_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&output_texture_view),
            },
        ],
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Compute Command Encoder"),
    });

    let start_time = Instant::now();
    let mut last_frame_time = start_time;

    loop {
        let current_time = start_time.elapsed().as_secs_f32();

        queue.write_buffer(
            &global_params_buffer,
            0,
            bytemuck::cast_slice(&[GlobalParams {
                time: current_time,
                _padding: [0.0; 3],
            }]),
        );

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                timestamp_writes: None,
                label: Some("Compute Pass"),
            });

            compute_pass.set_pipeline(&compute_pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);

            let workgroup_count_x = texture_size.width.div_ceil(16);
            let workgroup_count_y = texture_size.height.div_ceil(16);
            compute_pass.dispatch_workgroups(workgroup_count_x, workgroup_count_y, 1);
        }

        encoder.copy_texture_to_buffer(
            TexelCopyTextureInfo {
                texture: &output_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * texture_size.width),
                    rows_per_image: Some(texture_size.height),
                },
            },
            texture_size,
        );

        queue.submit(std::iter::once(encoder.finish()));

        let buffer_slice = output_buffer.slice(..);
        buffer_slice.map_async(MapMode::Read, |_| {});
        device.poll(PollType::Wait);

        let data = buffer_slice.get_mapped_range();
        let d_vec = data.to_vec();
        let image = image::RgbaImage::from_raw(width, height, d_vec)
            .context("Failed to create image")
            .unwrap();

        let im = DynamicImage::from(image).crop(0, 0, width, height);
        im.save("holofoil.png").unwrap();
        drop(data);
        output_buffer.unmap();

        encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Compute Command Encoder"),
        });

        let frame_time = Instant::now() - last_frame_time;
        if frame_time < Duration::from_secs_f32(1.0) {
            std::thread::sleep(Duration::from_secs_f32(1.0) - frame_time);
        }
        last_frame_time = Instant::now();
    }
}
