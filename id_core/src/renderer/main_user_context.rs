use std::{cell::RefCell, rc::Rc, time::Duration};

use anyhow::Result;
use encase::ShaderType;
use ultraviolet::{Mat4, UVec2, Vec2, Vec3};
use wgpu::BufferUsages;

use crate::{cvars::CVarUniforms, world::World, Stopwatch};

use super::{
    data::{PaletteColormapData, SectorData, WallData},
    helpers::{
        egui_system_platform::EguiPlatform,
        gpu::{GpuUniformBuffer, LenOrData::Len},
        window::{
            egui_user_context, EguiUserContext, HasEguiUserContext, UserContext,
            UserContextContext, UserContextSetup,
        },
    },
};

pub fn main_user_context(world: Rc<RefCell<World>>) -> impl UserContextSetup<MainUserContext> {
    move |context: &UserContextContext, size: UVec2| {
        let egui_user_context = egui_user_context()(context, size)?;

        let device = context.device;

        let ubo = GpuUniformBuffer::new(
            BufferUsages::UNIFORM,
            device,
            Len(UBO::min_size().get()),
            Some("MainUserContext::ubo"),
        )?;

        // Time how long it takes to parse the wad files.
        let mut stopwatch = Stopwatch::new();

        let (sector_data, wall_data, palette_colormap_data) = {
            let world = world.borrow();

            let wad = &world.wad;
            let map = &world.map;

            let sector_data = SectorData::new(device, wad, map)?;
            let wall_data = WallData::new(device, wad, map)?;
            let palette_colormap_data = PaletteColormapData::new(device, wad)?;

            (sector_data, wall_data, palette_colormap_data)
        };

        Ok(Box::new(MainUserContext {
            egui_user_context,
            world: world.clone(),

            ubo,

            sector_data,
            wall_data,
            palette_colormap_data,

            setup_time: stopwatch.lap(),
        }))
    }
}

#[derive(ShaderType)]
pub struct CameraInfo {
    view_proj_mat: Mat4,
    screen_size: Vec2,
    camera_pos: Vec3,
}

#[derive(ShaderType)]
pub struct UBO {
    camera_info: CameraInfo,
    cvars: CVarUniforms,
}

pub struct MainUserContext {
    egui_user_context: Box<EguiUserContext>,

    pub world: Rc<RefCell<World>>,

    pub ubo: GpuUniformBuffer<UBO>,

    pub sector_data: SectorData,
    pub wall_data: WallData,
    pub palette_colormap_data: PaletteColormapData,

    pub setup_time: Duration,
}

impl UserContext for MainUserContext {
    fn think(&mut self, context: &UserContextContext, delta: Duration) -> Result<()> {
        // Update egui if necessary.
        self.egui_user_context.think(context, delta)?;

        // Update the camera info, view-projection matrix, and cvars.
        let camera = &self.world.borrow().camera;
        let view_proj_mat = camera.get_projection_matrix(context.size) * camera.get_view_matrix();
        let ubo = &mut self.ubo;
        ubo.write(
            context.queue,
            UBO {
                camera_info: CameraInfo {
                    view_proj_mat,
                    screen_size: context.size.into(),
                    camera_pos: camera.pos,
                },
                cvars: CVarUniforms::from_cvars(&self.world.borrow().cvars),
            },
        )?;

        Ok(())
    }
}

impl HasEguiUserContext for MainUserContext {
    fn egui_platform(&mut self) -> &mut EguiPlatform {
        self.egui_user_context.egui_platform()
    }

    fn ui(&mut self) -> egui::Context {
        self.egui_user_context.ui()
    }
}
