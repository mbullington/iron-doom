use ultraviolet::UVec2;
use wgpu::Texture;

#[derive(Debug)]
pub struct GpuFrameTextureDescriptor {
    pub label: Option<&'static str>,
    pub format: wgpu::TextureFormat,
    pub mip_level_count: u32,
    pub sample_count: u32,
    pub view_formats: &'static [wgpu::TextureFormat],
}

impl Default for GpuFrameTextureDescriptor {
    fn default() -> Self {
        Self {
            label: None,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            view_formats: &[],
        }
    }
}

/// GpuFrameTexture is a lightweight wrapper around [Texture] that automatically
/// will resize the texture if the size of the texture is not the same as the
/// size of the swapchain.
#[derive(Debug)]
pub struct GpuFrameTexture {
    size: UVec2,
    texture: Texture,
    descriptor: GpuFrameTextureDescriptor,
}

impl GpuFrameTexture {
    fn _texture_of_size(
        device: &wgpu::Device,
        size: &UVec2,
        descriptor: &GpuFrameTextureDescriptor,
    ) -> Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            dimension: wgpu::TextureDimension::D2,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            label: descriptor.label,
            mip_level_count: descriptor.mip_level_count,
            sample_count: descriptor.sample_count,
            format: descriptor.format,
            view_formats: descriptor.view_formats,
        })
    }

    pub fn new(device: &wgpu::Device, size: &UVec2, descriptor: GpuFrameTextureDescriptor) -> Self {
        let texture = Self::_texture_of_size(device, size, &descriptor);
        Self {
            size: *size,
            texture,
            descriptor,
        }
    }

    pub fn create_texture(&mut self, device: &wgpu::Device, size: &UVec2) -> &Texture {
        if self.size != *size {
            self.size = *size;
            self.texture.destroy();
            self.texture = Self::_texture_of_size(device, size, &self.descriptor);
        }

        &self.texture
    }
}
