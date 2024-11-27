use std::collections::HashMap;

use anyhow::Error;
use encase::ShaderType;

pub const DEFAULT_CVARS: &[(&str, CVar)] = &[
    // #############################
    // GAMEPLAY VARIABLES:
    // #############################
    (
        "g_speed",
        CVar {
            description: "The speed of the player, in pixels per tick.",
            value: CVarValue::F32(1.0),
        },
    ),
    (
        "g_speedshift",
        CVar {
            description:
                "The speed of the player when they are holding the shift key, in pixels per tick.",
            value: CVarValue::F32(3.0),
        },
    ),
    // #############################
    // RENDERING VARIABLES:
    // These typically are also passed into CVarUniforms.
    // #############################

    // If 1, the renderer will render the scene with fullbright lighting.
    (
        "r_fullbright",
        CVar {
            description: "",
            value: CVarValue::Bool(false),
        },
    ),
    // Every 8 units, the light level falls off by 1.
    (
        "r_lightfalloff",
        CVar {
            description: "",
            value: CVarValue::F32(16.0),
        },
    ),
    // Z near plane for the camera.
    (
        "r_camera_znear",
        CVar {
            description: "",
            value: CVarValue::F32(1.0),
        },
    ),
    // FOV of the camera.
    (
        "r_camera_fov",
        CVar {
            description: "",
            value: CVarValue::F32(85.0),
        },
    ),
];

pub type CVarsMap = HashMap<&'static str, CVar>;

#[derive(ShaderType)]
/// This struct is used to pass the CVars into the shader uniform buffer.
/// Typically used for rendering variables.
pub struct CVarUniforms {
    /// WGSL doesn't support boolean types, so we use a u32 instead.
    pub r_fullbright: u32,
    pub r_lightfalloff: f32,
}

impl CVarUniforms {
    pub fn from_cvars(cvars: &CVarsMap) -> Self {
        Self {
            r_fullbright: cvars.get("r_fullbright").unwrap().value.as_bool().unwrap() as u32,
            r_lightfalloff: cvars.get("r_lightfalloff").unwrap().value.as_f32().unwrap(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CVar {
    pub description: &'static str,
    pub value: CVarValue,
}

#[derive(Debug, Clone, Copy)]
pub enum CVarValue {
    Bool(bool),
    U32(u32),
    F32(f32),
}

impl CVarValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            CVarValue::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_u32(&self) -> Option<u32> {
        match self {
            CVarValue::U32(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        match self {
            CVarValue::F32(v) => Some(*v),
            _ => None,
        }
    }

    pub fn set_from_str(&mut self, value: &str) -> Result<(), Error> {
        match self {
            CVarValue::Bool(ref mut v) => *v = value.parse()?,
            CVarValue::U32(ref mut v) => *v = value.parse()?,
            CVarValue::F32(ref mut v) => *v = value.parse()?,
        };

        Ok(())
    }
}
