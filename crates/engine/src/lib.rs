#![feature(trait_alias)]
#![feature(generator_trait)]
#![feature(generators)]
#![feature(ptr_as_uninit)]
#![feature(iter_from_generator)]
#![feature(try_trait_v2)]
#![windows_subsystem = "windows"]

mod geometry;
mod input;
mod renderer;
mod world;

pub use geometry::*;
pub use input::*;
pub use renderer::*;
pub use world::*;
