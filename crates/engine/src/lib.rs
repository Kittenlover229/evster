#![feature(trait_alias)]
#![feature(generator_trait)]
#![feature(generators)]
#![feature(ptr_as_uninit)]
#![feature(iter_from_generator)]
#![feature(try_trait_v2)]
#![windows_subsystem = "windows"]

mod renderer;
mod input;
mod world;
mod geometry;

pub use renderer::*;
pub use world::*;
pub use input::*;
pub use geometry::*;
