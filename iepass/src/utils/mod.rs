#![allow(unused_imports)]

pub mod framebuffer;
pub mod fps_counter;
pub mod perf;

pub use framebuffer::Framebuffer;
pub use fps_counter::FpsCounter;
pub use perf::{PerfFuture, PerfFutureExt};

pub(crate) static PSRAM_ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();
