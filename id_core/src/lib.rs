#![allow(incomplete_features)]
#![feature(lazy_type_alias)]
#![feature(trait_alias)]

pub mod cvars;
pub mod renderer;
pub mod world;

mod animation_state_map;
mod sector_accel;

pub mod components;
pub mod entities;

pub(crate) mod helpers;

pub use helpers::Stopwatch;

pub use animation_state_map::AnimationStateMap;
pub use sector_accel::SectorAccel;
