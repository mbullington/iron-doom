use std::time::Duration;

use anyhow::Result;
use ultraviolet::UVec2;

use super::super::window::{UserContext, UserContextContext, UserContextSetup};
use super::egui_system_platform::EguiPlatform;

pub trait HasEguiUserContext {
    fn egui_platform(&mut self) -> &mut EguiPlatform;
    fn ui(&mut self) -> egui::Context;
}

pub fn egui_user_context() -> impl UserContextSetup<EguiUserContext> {
    move |_context: &UserContextContext, size: UVec2| {
        let egui_platform = EguiPlatform::new((size.x, size.y))?;
        Ok(Box::new(EguiUserContext { egui_platform }))
    }
}

pub struct EguiUserContext {
    egui_platform: EguiPlatform,
}

impl UserContext for EguiUserContext {
    fn think(&mut self, _context: &UserContextContext, _delta: Duration) -> Result<()> {
        Ok(())
    }
}

impl HasEguiUserContext for EguiUserContext {
    fn egui_platform(&mut self) -> &mut EguiPlatform {
        &mut self.egui_platform
    }

    fn ui(&mut self) -> egui::Context {
        self.egui_platform.context()
    }
}
