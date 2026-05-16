pub mod about_surface;
pub mod application;
pub mod assets;
pub mod components;
pub mod easing;
pub mod hsv;
pub mod icon_tool;
pub mod jobs;
pub mod lerp;
pub mod macros;
pub mod notification;
pub mod permissions;
pub mod platform_support;
pub mod setup;
pub mod styling;
pub mod surface;
mod tracing;
pub mod transition;
pub mod window;

#[cfg(feature = "self-update")]
pub mod self_update;

#[cfg(feature = "tokio")]
pub mod tokio;

pub mod settings;
#[cfg(feature = "parlance")]
pub mod setup_parlance;
