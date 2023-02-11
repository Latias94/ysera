#![allow(clippy::missing_safety_doc)]

extern crate alloc;
extern crate core;

pub use ash;
use core::fmt::Debug;
pub use winit;

pub use error::*;

mod error;
mod gui;
pub mod types;
pub mod vulkan;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub trait GraphicsApi: Clone + Sized {
    type Framebuffer: Debug + Send + Sync;
}
