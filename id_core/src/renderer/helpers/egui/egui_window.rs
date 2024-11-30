use std::time::Duration;

use anyhow::Result;
use egui_wgpu::ScreenDescriptor;

use super::super::system::SystemEvent;
use super::super::window::{UserContext, Window, WindowContext, WindowSetup};

use super::egui_user_context::HasEguiUserContext;

pub fn egui_window<UC: UserContext + HasEguiUserContext + 'static>(
    child: impl WindowSetup<UC>,
) -> impl WindowSetup<UC> {
    move |context, size| {
        let egui_renderer = egui_wgpu::Renderer::new(
            context.device,
            *context.surface_format,
            None,
            1,
            /* dithering: */ false,
        );

        Ok(Box::new(EguiWindow {
            egui_renderer,
            child: child(context, size)?,
        }))
    }
}

pub struct EguiWindow<UC: UserContext + HasEguiUserContext + 'static> {
    pub egui_renderer: egui_wgpu::Renderer,
    pub child: Box<dyn Window<UC>>,
}

impl<UC: UserContext + HasEguiUserContext + 'static> Window<UC> for EguiWindow<UC> {
    fn handle_event(
        &mut self,
        context: &mut WindowContext<UC>,
        event: &SystemEvent,
    ) -> Result<bool> {
        let egui_platform = context.user_context.egui_platform();
        let consumed = egui_platform.handle_event(event);

        if !consumed {
            self.child.handle_event(context, event)?;
        }

        Ok(consumed)
    }

    fn think(&mut self, context: &mut WindowContext<UC>, _delta: Duration) -> Result<()> {
        self.child.think(context, _delta)
    }

    fn draw(
        &mut self,
        context: &mut WindowContext<UC>,
        texture: &wgpu::Texture,
        delta: Duration,
    ) -> Result<()> {
        let egui_platform = context.user_context.egui_platform();
        egui_platform.begin_frame();

        // Draw child first.
        self.child.draw(context, texture, delta)?;

        let egui_platform = context.user_context.egui_platform();
        let egui_renderer = &mut self.egui_renderer;

        let device = context.device;
        let queue = context.queue;

        let full_output = egui_platform.end_frame()?;
        for (id, image_delta) in &full_output.textures_delta.set {
            egui_renderer.update_texture(device, queue, *id, image_delta);
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("EguiWindow::command_encoder"),
        });

        let paint_jobs = egui_platform.tessellate(&full_output, /* retina: */ 1.0);
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [context.size.x, context.size.y],
            pixels_per_point: 1.0,
        };

        egui_renderer.update_buffers(device, queue, &mut encoder, &paint_jobs, &screen_descriptor);

        let output = texture.create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut rpass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &output,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    label: Some("EguiWindow::rpass"),
                    ..Default::default()
                })
                .forget_lifetime();

            egui_renderer.render(&mut rpass, &paint_jobs, &screen_descriptor);
        }

        queue.submit([encoder.finish()]);
        for texture in full_output.textures_delta.free {
            egui_renderer.free_texture(&texture);
        }

        Ok(())
    }
}
