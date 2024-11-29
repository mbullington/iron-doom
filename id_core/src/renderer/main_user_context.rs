use std::{cell::RefCell, rc::Rc, time::Duration};

use anyhow::Result;
use encase::ShaderType;
use ultraviolet::{Mat4, UVec2, Vec2, Vec3};
use wgpu::BufferUsages;

use crate::{cvars::CVarUniforms, helpers::Camera, world::World, Stopwatch};

use super::{
    data::{PaletteColormapData, PaletteImageData, SectorData, WallData},
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
        let queue = context.queue;

        let ubo = GpuUniformBuffer::new(
            BufferUsages::UNIFORM,
            device,
            Len(Ubo::min_size().get()),
            Some("MainUserContext::ubo"),
        )?;

        // Time how long it takes to parse the wad files.
        let mut stopwatch = Stopwatch::new();

        let (palette_colormap_data, palette_image_data, sector_data, wall_data) = {
            let world = world.borrow();

            let palette_colormap_data = PaletteColormapData::new(device, &world)?;
            let palette_image_data = PaletteImageData::new(device, queue, &world)?;

            let sector_data = SectorData::new(device, queue, &world, &palette_image_data)?;
            let wall_data =
                WallData::new(device, queue, &world, &sector_data, &palette_image_data)?;

            (
                palette_colormap_data,
                palette_image_data,
                sector_data,
                wall_data,
            )
        };

        Ok(Box::new(MainUserContext {
            egui_user_context,
            world: world.clone(),

            ubo,

            palette_colormap_data,
            palette_image_data,

            sector_data,
            wall_data,

            setup_time: stopwatch.lap(),
        }))
    }
}

#[derive(ShaderType)]
pub struct CameraInfo {
    view_proj_mat: Mat4,
    screen_size: Vec2,
    camera_pos: Vec3,
    rotation_rad: f32,
}

#[derive(ShaderType)]
pub struct Ubo {
    camera_info: CameraInfo,
    cvars: CVarUniforms,
}

pub struct MainUserContext {
    egui_user_context: Box<EguiUserContext>,

    pub world: Rc<RefCell<World>>,

    pub ubo: GpuUniformBuffer<Ubo>,

    pub palette_colormap_data: PaletteColormapData,
    pub palette_image_data: PaletteImageData,

    pub sector_data: SectorData,
    pub wall_data: WallData,

    pub setup_time: Duration,
}

impl UserContext for MainUserContext {
    fn think(&mut self, context: &UserContextContext, delta: Duration) -> Result<()> {
        let world = self.world.clone();

        // Start by letting the world think.
        world.borrow_mut().think()?;

        // Update egui if necessary.
        self.egui_user_context.think(context, delta)?;

        // Update sector & wall data.
        self.sector_data
            .think(context.queue, &world.borrow(), &self.palette_image_data)?;
        self.wall_data.think(
            context.queue,
            &world.borrow(),
            &self.sector_data,
            &self.palette_image_data,
        )?;

        // Update the camera info, view-projection matrix, and cvars.
        let cvars = CVarUniforms::from_cvars(&self.world.borrow().cvars);

        let camera_info = world.borrow_mut().with_player_pos(|player| {
            let camera = Camera {
                movable: player,
                z_near: cvars.r_znear,
                fov: cvars.r_fov,
            };

            let view_proj_mat =
                camera.get_projection_matrix(context.size) * camera.get_view_matrix();

            CameraInfo {
                view_proj_mat,
                screen_size: context.size.into(),
                camera_pos: player.pos,
                rotation_rad: player.yaw.to_radians(),
            }
        })?;

        let ubo = &mut self.ubo;
        ubo.write(context.queue, Ubo { camera_info, cvars })?;

        // End by letting the world think_end.
        world.borrow_mut().think_end()?;

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
