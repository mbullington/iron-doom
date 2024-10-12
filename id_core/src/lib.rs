#![allow(incomplete_features)]
#![feature(lazy_type_alias)]
#![feature(trait_alias)]

pub mod cvars;
pub mod renderer;
pub mod world;

pub mod components;
pub mod entities;

pub(crate) mod helpers;

pub use helpers::Stopwatch;
