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
