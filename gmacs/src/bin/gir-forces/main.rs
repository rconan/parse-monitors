use itertools::Itertools;
use parse_monitors::{
    cfd::{self, BaselineTrait, CfdCase},
    MonitorsLoader,
};
use rayon::prelude::*;
use std::fs::File;

fn main() -> anyhow::Result<()> {
    let gir_fxy_max: Vec<_> = cfd::Baseline::<2021>::default()
        .into_iter()
        .collect::<Vec<CfdCase<2021>>>()
        .into_par_iter()
        .map(|cfd_case| {
            let data_path = cfd::Baseline::<2021>::path().join(cfd_case.to_string());
            let mut gir = MonitorsLoader::<2021>::default()
                .data_path(data_path)
                .header_filter("GIR")
                .load()
                .unwrap();
            gir.keep_last(300);
            let (fx, fy): (Vec<f64>, Vec<f64>) = gir
                .forces_and_moments
                .into_values()
                .next()
                .unwrap()
                .into_iter()
                .map(|e| (e.force.x.unwrap(), e.force.y.unwrap()))
                .unzip();
            let fx_max = fx
                .into_iter()
                .map(|x| x.abs())
                .fold(std::f64::NEG_INFINITY, f64::max);
            let fy_max = fy
                .into_iter()
                .map(|x| x.abs())
                .fold(std::f64::NEG_INFINITY, f64::max);
            (cfd_case.to_string(), fx_max, fy_max)
        })
        .collect();

    serde_pickle::to_writer(
        &mut File::create("gir_fxy_max.pkl")?,
        &gir_fxy_max,
        Default::default(),
    )?;

    Ok(())
}
