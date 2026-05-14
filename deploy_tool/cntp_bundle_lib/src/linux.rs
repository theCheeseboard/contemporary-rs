mod appimage;
pub mod bundle;
pub mod deploy;
mod rootdir;

const APPRUN_TEMPLATE: &str = include_str!("linux/apprun.sh");
