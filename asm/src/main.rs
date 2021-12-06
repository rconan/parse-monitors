use asm::{pressure, refraction_index};
use parse_monitors::cfd;
//use rayon::prelude::*;
use polars::prelude::*;
use std::time::Instant;

const R: f64 = 1.2;

fn refraction_index(temperature: f64) -> f64 {
    let pref = 75000.0; //  Reference pressure [Pa]
    let wlm = 0.5; // wavelength [micron]
    7.76e-7 * pref * (1. + 0.00752 / (wlm * wlm)) / temperature
}

fn main() -> anyhow::Result<()> {
    let duration = 400_usize;
    let cfd_case = cfd::CfdCase::<2021>::colloquial(30, 0, "os", 7)?;
    println!("CFD case: {}", cfd_case);
    let now = Instant::now();
    let pressure_df = pressure::stats(duration, cfd_case, R)?
        .into_iter()
        .collect::<Result<DataFrame>>()?;
    println!("pressure files processed in: {}s", now.elapsed().as_secs());
    print!("{}", pressure_df);

    Ok(())
}
