use std::{borrow::Cow, time::Duration};

use wgpu::ShaderStages;
use wgpu_pp::include_wgsl;

use anyhow::Result;
use encase::ShaderType;
use ultraviolet::{Mat4, UVec2, Vec2, Vec3};

use crate::{
    cvars::CVarUniforms,
    renderer::helpers::{
        gpu::{GpuFrameTexture, GpuFrameTextureDescriptor},
        system::SystemEvent,
        window::{Window, WindowContext, WindowSetup},
    },
};

use super::{
    data::SectorData, helpers::movement_controller::MovementController,
    main_user_context::MainUserContext,
};

#[derive(ShaderType)]
struct CameraInfo {
    view_proj_mat: Mat4,
    screen_size: Vec2,
    camera_pos: Vec3,
}

#[derive(ShaderType)]
struct UBO {
    camera_info: CameraInfo,
    cvars: CVarUniforms,
}

pub struct MainWindow {
    movement_controller: MovementController,

    bind_group: wgpu::BindGroup,

    render_pipeline_floor: wgpu::RenderPipeline,
    render_pipeline_ceiling: wgpu::RenderPipeline,
    render_pipeline_wall: wgpu::RenderPipeline,

    depth_texture: GpuFrameTexture,
}

fn _create_sector_render_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    pipeline_layout: &wgpu::PipelineLayout,
    sector_data: &SectorData,
    front_face: wgpu::FrontFace,
) -> wgpu::RenderPipeline {
    let vertex_state = wgpu::VertexState {
        buffers: &[wgpu::VertexBufferLayout {
            array_stride: sector_data.vertex_buf.stride as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint32,
                    offset: 8,
                    shader_location: 1,
                },
            ],
        }],
        module: shader,
        entry_point: "vs_main",
        compilation_options: Default::default(),
    };

    let fragment_state = Some(wgpu::FragmentState {
        targets: &[Some(wgpu::ColorTargetState {
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        })],
        module: shader,
        entry_point: "fs_main",
        compilation_options: Default::default(),
    });

    let depth_stencil = Some(wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth32Float,
        depth_write_enabled: true,
        // Since we're using reverse depth.
        depth_compare: wgpu::CompareFunction::Greater,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        layout: Some(pipeline_layout),
        vertex: vertex_state,
        fragment: fragment_state,
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil,
        label: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    })
}

fn _create_wall_render_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    pipeline_layout: &wgpu::PipelineLayout,
) -> wgpu::RenderPipeline {
    let vertex_state = wgpu::VertexState {
        buffers: &[wgpu::VertexBufferLayout {
            array_stride: 8,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            }],
        }],
        module: shader,
        entry_point: "vs_main",
        compilation_options: Default::default(),
    };

    let fragment_state = Some(wgpu::FragmentState {
        targets: &[Some(wgpu::ColorTargetState {
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })],
        module: shader,
        entry_point: "fs_main",
        compilation_options: Default::default(),
    });

    let depth_stencil = Some(wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth32Float,
        depth_write_enabled: true,
        // Since we're using reverse depth.
        depth_compare: wgpu::CompareFunction::Greater,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        layout: Some(pipeline_layout),
        vertex: vertex_state,
        fragment: fragment_state,
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil,
        label: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    })
}

use MainUserContext as UC;

pub fn main_window() -> impl WindowSetup<UC> {
    move |context: &WindowContext<UC>, size: UVec2| {
        let device = context.device;

        let ubo = &context.user_context.ubo;

        let sector_data = &context.user_context.sector_data;
        let wall_data = &context.user_context.wall_data;
        let palette_colormap_data = &context.user_context.palette_colormap_data;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_wgsl!("./shaders/sector.wgsl"))),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                ubo.bind_group_layout_entry(0, ShaderStages::all()),
                sector_data
                    .storage_buf
                    .bind_group_layout_entry(1, ShaderStages::all()),
                sector_data
                    .flat_storage_buf
                    .bind_group_layout_entry(2, ShaderStages::FRAGMENT),
                wall_data
                    .vertices_storage_buffer
                    .bind_group_layout_entry(3, ShaderStages::all()),
                wall_data
                    .wall_storage_buffer
                    .bind_group_layout_entry(4, ShaderStages::all()),
                wall_data
                    .patch_headers_storage_data
                    .bind_group_layout_entry(5, ShaderStages::all()),
                wall_data
                    .patch_storage_buffer
                    .bind_group_layout_entry(6, ShaderStages::all()),
                palette_colormap_data
                    .palette_storage_buf
                    .bind_group_layout_entry(7, ShaderStages::FRAGMENT),
                palette_colormap_data
                    .colormap_storage_buf
                    .bind_group_layout_entry(8, ShaderStages::FRAGMENT),
            ],
            label: Some("bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                ubo.bind_group_descriptor_entry(0),
                sector_data.storage_buf.bind_group_descriptor_entry(1),
                sector_data.flat_storage_buf.bind_group_descriptor_entry(2),
                wall_data
                    .vertices_storage_buffer
                    .bind_group_descriptor_entry(3),
                wall_data.wall_storage_buffer.bind_group_descriptor_entry(4),
                wall_data
                    .patch_headers_storage_data
                    .bind_group_descriptor_entry(5),
                wall_data
                    .patch_storage_buffer
                    .bind_group_descriptor_entry(6),
                palette_colormap_data
                    .palette_storage_buf
                    .bind_group_descriptor_entry(7),
                palette_colormap_data
                    .colormap_storage_buf
                    .bind_group_descriptor_entry(8),
            ],
            label: Some("bind_group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
            label: None,
            push_constant_ranges: &[],
        });

        let render_pipeline_floor = _create_sector_render_pipeline(
            device,
            &shader,
            &pipeline_layout,
            &sector_data,
            wgpu::FrontFace::Cw,
        );
        let render_pipeline_ceiling = _create_sector_render_pipeline(
            device,
            &shader,
            &pipeline_layout,
            &sector_data,
            wgpu::FrontFace::Ccw,
        );

        let shader_wall = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_wgsl!("./shaders/wall.wgsl"))),
        });

        let render_pipeline_wall =
            _create_wall_render_pipeline(device, &shader_wall, &pipeline_layout);

        let depth_texture = GpuFrameTexture::new(
            device,
            &size,
            GpuFrameTextureDescriptor {
                label: Some("MainWindow::depth_texture"),
                format: wgpu::TextureFormat::Depth32Float,
                ..Default::default()
            },
        );

        Ok(Box::new(MainWindow {
            movement_controller: MovementController::new(),

            bind_group,

            render_pipeline_floor,
            render_pipeline_ceiling,
            render_pipeline_wall,

            depth_texture,
        }))
    }
}

impl Window<UC> for MainWindow {
    fn handle_event(
        &mut self,
        context: &mut WindowContext<UC>,
        event: &SystemEvent,
    ) -> Result<bool> {
        let world = context.user_context.world.clone();

        // Handle mouse motion separately from movement controller.
        // TODO: Combine this.
        if let SystemEvent::MouseMotion { xrel, yrel, .. } = event {
            world
                .borrow_mut()
                .camera
                .rotate_pitch_yaw((*yrel as f32) * 2.2, (*xrel as f32) * 2.2);
        }

        self.movement_controller.handle_event(event);
        Ok(false)
    }

    fn think(&mut self, context: &mut WindowContext<UC>, delta: Duration) -> Result<()> {
        let world = context.user_context.world.clone();

        self.movement_controller
            .think(&mut world.borrow_mut().camera, delta);
        Ok(())
    }

    fn draw(
        &mut self,
        context: &mut WindowContext<UC>,
        texture: &wgpu::Texture,
        _delta: Duration,
    ) -> Result<()> {
        let device = context.device;
        let queue = context.queue;

        let sector_data = &context.user_context.sector_data;
        let wall_data = &context.user_context.wall_data;

        let output = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let output_depth = self
            .depth_texture
            .create_texture(device, &context.size)
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("command_encoder"),
        });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &output_depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                label: Some("MainWindow::render_pass"),
                ..Default::default()
            });

            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_vertex_buffer(0, sector_data.vertex_buf.buf.slice(..));
            rpass.set_index_buffer(
                sector_data.index_buf.buf.slice(..),
                wgpu::IndexFormat::Uint32,
            );

            {
                rpass.set_pipeline(&self.render_pipeline_floor);
                rpass.draw_indexed(
                    0..(sector_data.index_buf.buf.size() / (sector_data.index_buf.stride as u64))
                        as u32,
                    0,
                    0..1,
                );
            }

            {
                rpass.set_pipeline(&self.render_pipeline_ceiling);
                rpass.draw_indexed(
                    0..(sector_data.index_buf.buf.size() / (sector_data.index_buf.stride as u64))
                        as u32,
                    0,
                    1..2,
                );
            }

            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_vertex_buffer(0, wall_data.wall_quad_vertex_buf.buf.slice(..));

            {
                rpass.set_pipeline(&self.render_pipeline_wall);
                rpass.draw(
                    0..(wall_data.wall_quad_vertex_buf.buf.size()
                        / (wall_data.wall_quad_vertex_buf.stride as u64))
                        as u32,
                    0..(wall_data.wall_storage_buffer.buf.size()
                        / (wall_data.wall_storage_buffer.stride as u64))
                        as u32,
                );
            }
        }

        queue.submit([encoder.finish()]);
        Ok(())
    }
}