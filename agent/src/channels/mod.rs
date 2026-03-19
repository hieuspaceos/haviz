pub mod traits;
pub mod zalo_web;

#[cfg(target_os = "macos")]
pub mod zalo_desktop;

#[cfg(target_os = "windows")]
pub mod zalo_desktop_windows;
