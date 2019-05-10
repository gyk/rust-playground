
pub mod local;
pub mod remote;

mod config;
mod nested_source;
mod value_ext;
mod error;

pub const DEFAULT_ID: &str = "_default_";
pub const OVERRIDE_ID: &str = "_override_";
