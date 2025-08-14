use std::sync::LazyLock;

use clap::Subcommand;

pub mod batch_force;
pub mod dome_seeing;
mod error;
pub mod opd_maps;
pub mod pressure_maps;
pub mod report;

pub use error::{ReportError, ReportPathError};
use indicatif::MultiProgress;

pub const PREVIOUS_YEAR: u32 = 2021;

static PROGRESS: LazyLock<MultiProgress> = LazyLock::new(|| MultiProgress::new());

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
impl ForcesCli {
    pub fn all() -> Self {
        let mut this = Self::default();
        this.all = true;
        this
    }
}

#[derive(Debug, Clone, Subcommand)]
pub enum ReportOptions {
    /// generates the full report: windloads, dome seeing and HTC
    Full,
    /// generates only the dome seeing part of the report
    DomeSeeing,
    /// generates only the windloads part of the report
    WindLoads,
    /// generates only the HTC part of the report
    HTC,
}
