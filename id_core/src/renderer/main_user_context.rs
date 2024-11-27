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

        let ubo = GpuUniformBuffer::new(
            BufferUsages::UNIFORM,
            device,
            Len(UBO::min_size().get()),
            Some("MainUserContext::ubo"),
        )?;

        // Time how long it takes to parse the wad files.
        let mut stopwatch = Stopwatch::new();

        let (palette_colormap_data, palette_image_data, sector_data, wall_data) = {
            let world = world.borrow();

            let palette_colormap_data = PaletteColormapData::new(device, &world.wad)?;
            let palette_image_data = PaletteImageData::new(device, &world)?;

            let sector_data = SectorData::new(device, &world, &palette_image_data)?;
            let wall_data = WallData::new(device, &world, &sector_data, &palette_image_data)?;

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

    pub palette_colormap_data: PaletteColormapData,
    pub palette_image_data: PaletteImageData,

    pub sector_data: SectorData,
    pub wall_data: WallData,

    pub setup_time: Duration,
}

impl UserContext for MainUserContext {
    fn think(&mut self, context: &UserContextContext, delta: Duration) -> Result<()> {
        // Update egui if necessary.
        self.egui_user_context.think(context, delta)?;

        // Update the camera info, view-projection matrix, and cvars.
        let world = self.world.clone();
        let z_near = world
            .borrow()
            .cvars
            .get("r_camera_znear")
            .unwrap()
            .value
            .as_f32()
            .unwrap();
        let fov = world
            .borrow()
            .cvars
            .get("r_camera_fov")
            .unwrap()
            .value
            .as_f32()
            .unwrap();

        let camera_info = world.borrow_mut().with_player_pos(|player| {
            let camera = Camera {
                movable: player,
                z_near,
                fov,
            };

            let view_proj_mat =
                camera.get_projection_matrix(context.size) * camera.get_view_matrix();

            CameraInfo {
                view_proj_mat,
                screen_size: context.size.into(),
                camera_pos: player.pos,
            }
        })?;

        let ubo = &mut self.ubo;
        ubo.write(
            context.queue,
            UBO {
                camera_info,
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
