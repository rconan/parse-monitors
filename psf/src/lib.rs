
pub const DETECTOR_SIZE: usize = 760;

mod config;
mod psfs;
pub use config::Config;
pub use psfs::{PSF, PSFs};
