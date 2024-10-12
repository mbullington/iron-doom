use std::{borrow::BorrowMut, time::Duration};

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use ultraviolet::UVec2;
use wgpu::{Backends, Gles3MinorVersion, Instance, InstanceDescriptor, InstanceFlags};

use super::super::system::SystemEvent;
use super::{
    UserContext, UserContextContext, UserContextSetup, Window, WindowContext, WindowSetup,
};

pub struct WindowRunner<'surface, UC: UserContext> {
    window: Box<dyn Window<UC>>,
    user_context: Box<UC>,

    size: UVec2,

    surface: wgpu::Surface<'surface>,
    surface_format: wgpu::TextureFormat,

    device: wgpu::Device,
    queue: wgpu::Queue,

    // This is somewhat odd... but to keep the think/draw in separate functions,
    // we wait for the next swapchain at the end of draw.
    next_texture: Option<wgpu::SurfaceTexture>,
}

#[derive(Debug)]
pub enum WindowRunnerError {
    HandleError(raw_window_handle::HandleError),

    CreateSurfaceError(wgpu::CreateSurfaceError),
    RequestDeviceError(wgpu::RequestDeviceError),
    SurfaceError(wgpu::SurfaceError),
    WindowError(anyhow::Error),

    InvalidWindowDimensions,

    NoSwapchain,
    NoSuitableAdapter,
}

fn _create_surface_information(
    surface_format: wgpu::TextureFormat,
    width: u32,
    height: u32,
) -> wgpu::SurfaceConfiguration {
    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width,
        height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: Vec::default(),
        desired_maximum_frame_latency: 2,
    }
}

impl<'surface, UC: UserContext> WindowRunner<'surface, UC> {
    pub async fn from_system_window<'a, SW, U, T>(
        system_window: &'surface SW,
        drawable_size: UVec2,
        user_context_setup: U,
        window_setup: T,
    ) -> Result<Self, WindowRunnerError>
    where
        SW: HasWindowHandle + HasDisplayHandle,
        U: UserContextSetup<UC>,
        T: WindowSetup<UC>,
    {
        let (width, height) = (drawable_size.x, drawable_size.y);

        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::from_bits(Backends::VULKAN.bits() | Backends::METAL.bits())
                .unwrap(),
            flags: InstanceFlags::empty(),
            dx12_shader_compiler: Default::default(),
            gles_minor_version: Gles3MinorVersion::Automatic,
        });

        let surface = unsafe {
            match instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_window_handle: system_window
                    .window_handle()
                    .map_err(WindowRunnerError::HandleError)?
                    .into(),
                raw_display_handle: system_window
                    .display_handle()
                    .map_err(WindowRunnerError::HandleError)?
                    .into(),
            }) {
                Ok(s) => s,
                Err(e) => return Err(WindowRunnerError::CreateSurfaceError(e)),
            }
        };

        let adapter = match instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::None,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
        {
            Some(a) => a,
            None => return Err(WindowRunnerError::NoSuitableAdapter),
        };

        let (device, queue) = match adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device"),
                    required_features: wgpu::Features::TEXTURE_COMPRESSION_BC,
                    required_limits: wgpu::Limits::default(),
                    ..Default::default()
                },
                None,
            )
            .await
        {
            Ok(a) => a,
            Err(e) => return Err(WindowRunnerError::RequestDeviceError(e)),
        };

        // Create swap chain.

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap();

        surface.configure(
            &device,
            &_create_surface_information(surface_format, width, height),
        );

        let user_context_context = UserContextContext {
            device: &device,
            queue: &queue,
            surface_format: &surface_format,
            size: drawable_size,
        };

        // Setup the user context.
        let mut user_context = match user_context_setup(&user_context_context, drawable_size) {
            Ok(user_context) => user_context,
            Err(e) => {
                return Err(WindowRunnerError::WindowError(e));
            }
        };

        let window_context = WindowContext {
            device: &device,
            queue: &queue,
            surface_format: &surface_format,
            size: drawable_size,
            user_context: user_context.borrow_mut(),
        };

        // Setup the window.
        let window = match window_setup(
            &window_context,
            UVec2 {
                x: width,
                y: height,
            },
        ) {
            Ok(window) => window,
            Err(e) => {
                return Err(WindowRunnerError::WindowError(e));
            }
        };

        // Block on the first swapchain so it's ready.
        let next_texture = match surface.get_current_texture() {
            Ok(frame) => Some(frame),
            Err(e) => {
                return Err(WindowRunnerError::SurfaceError(e));
            }
        };

        Ok(WindowRunner {
            window,
            user_context,

            size: UVec2 {
                x: width,
                y: height,
            },

            surface,
            surface_format,

            device,
            queue,

            next_texture,
        })
    }

    pub fn handle_event(&mut self, event: SystemEvent) -> Result<(), WindowRunnerError> {
        match event {
            SystemEvent::SizeChanged { width, height } => {
                self.size.x = width;
                self.size.y = height;

                self.next_texture.take();
                self.surface.configure(
                    &self.device,
                    &_create_surface_information(self.surface_format, width, height),
                );

                // Get the next swapchain.
                self.next_texture = match self.surface.get_current_texture() {
                    Ok(frame) => Some(frame),
                    Err(e) => {
                        return Err(WindowRunnerError::SurfaceError(e));
                    }
                };
            }
            _ => {}
        }

        let mut window_context = WindowContext {
            device: &self.device,
            queue: &self.queue,
            surface_format: &self.surface_format,
            size: self.size,
            user_context: self.user_context.borrow_mut(),
        };

        // Let the window handle the event.
        match self.window.handle_event(&mut window_context, &event) {
            Ok(_) => {}
            Err(e) => return Err(WindowRunnerError::WindowError(e)),
        }

        Ok(())
    }

    pub fn think(&mut self, delta: Duration) -> Result<(), WindowRunnerError> {
        {
            let user_context_context = UserContextContext {
                device: &self.device,
                queue: &self.queue,
                surface_format: &self.surface_format,
                size: self.size,
            };

            match self.user_context.think(&user_context_context, delta) {
                Ok(_) => {}
                Err(e) => {
                    return Err(WindowRunnerError::WindowError(e));
                }
            };
        }

        {
            let mut window_context = WindowContext {
                device: &self.device,
                queue: &self.queue,
                surface_format: &self.surface_format,
                size: self.size,
                user_context: self.user_context.borrow_mut(),
            };

            match self.window.think(&mut window_context, delta) {
                Ok(_) => Ok(()),
                Err(e) => Err(WindowRunnerError::WindowError(e)),
            }
        }
    }

    pub fn draw(&mut self, delta: Duration) -> Result<(), WindowRunnerError> {
        if self.next_texture.is_none() {
            return Err(WindowRunnerError::NoSwapchain);
        }

        let mut window_draw_context = WindowContext {
            device: &self.device,
            queue: &self.queue,
            surface_format: &self.surface_format,
            size: self.size,
            user_context: self.user_context.borrow_mut(),
        };

        // Draw the current window using the stored swapchain.
        match self.window.draw(
            &mut window_draw_context,
            &self.next_texture.as_ref().unwrap().texture,
            delta,
        ) {
            Ok(_) => {}
            Err(e) => {
                return Err(WindowRunnerError::WindowError(e));
            }
        }

        let texture = self.next_texture.take();
        texture.unwrap().present();

        // Get the next swapchain.
        self.next_texture = match self.surface.get_current_texture() {
            Ok(frame) => Some(frame),
            Err(e) => {
                return Err(WindowRunnerError::SurfaceError(e));
            }
        };

        Ok(())
    }
}
