use std::{error::Error, fmt::Display, io, path::PathBuf};

pub mod batch_force;
pub mod dome_seeing;
pub mod opd_maps;
pub mod pressure_maps;

#[derive(Default, Debug, Clone)]
pub struct ForcesCli {
    pub last: Option<usize>,
    pub all: bool,
    pub crings: bool,
    pub m1_cell: bool,
    pub upper_truss: bool,
    pub lower_truss: bool,
    pub top_end: bool,
    pub m1_segments: bool,
    pub m2_segments: bool,
    pub m12_baffles: bool,
    pub m1_inner_covers: bool,
    pub m1_outer_covers: bool,
    pub gir: bool,
    pub pfa_arms: bool,
    pub lgsa: bool,
    pub platforms_cables: bool,
    pub detrend: bool,
}

#[derive(Debug)]
pub struct ReportPathError {
    path: PathBuf,
    source: io::Error,
}
impl ReportPathError {
    pub fn new(path: PathBuf, source: io::Error) -> Self {
        Self { path, source }
    }
}
impl Display for ReportPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to create report folder: {:?}", self.path)
    }
}
impl Error for ReportPathError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}
