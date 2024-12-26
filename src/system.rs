#[cfg(all(feature = "miniquad", feature = "sdl"))]
compile_error!("You can't have both backends");

#[cfg(feature = "miniquad")]
mod system_miniquad;
#[cfg(feature = "miniquad")]
use system_miniquad as inner;

#[cfg(feature = "sdl")]
mod system_sdl;
#[cfg(feature = "sdl")]
use system_sdl as inner;

pub use inner::*;
