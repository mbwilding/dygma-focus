#[cfg(windows)]
pub mod windows;

#[cfg(unix)]
pub mod posix;

#[cfg(target_arch = "wasm32")]
pub mod wasm;
