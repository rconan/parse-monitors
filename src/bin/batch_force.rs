use parse_monitors::{cfd::Baseline, plot_monitor, Mirror, MonitorsLoader};
use rayon::prelude::*;
use std::path::Path;

const CFD_YEAR: u32 = 2021;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    //let xmon = "floor|shutter|screen|enclosure";
    let cfd_root = match CFD_YEAR {
        2020_u32 => Path::new("/fsx/Baseline2020"),
        2021_u32 => Path::new("/fsx/Baseline2021/Baseline2021/Baseline2021/CASES"),
        _ => panic!("Not a good year!"),
    };
    let data_paths: Vec<_> = Baseline::<CFD_YEAR>::default()
        .into_iter()
        .map(|cfd_case| {
            cfd_root
                .join(cfd_case.to_string())
                .to_str()
                .unwrap()
                .to_string()
        })
        .collect();
    let n_cases = data_paths.len();
    println!("Found {} CFD cases", n_cases);

    let parts = vec![
        ("c-ring_parts", "Cring"),
        ("m1-cell", "M1cell"),
        ("upper-truss", "Tu"),
        ("lower-truss", "Tb"),
        ("top-end", "Top"),
        ("m2-segments", "M2s"),
        ("m12-baffles", "Baf"),
        ("m1-outer-covers", "M1cov[1-6]"),
        ("m1-inner-covers", "M1covin[1-6]"),
        ("gir", "GIR"),
        ("pfa-arms", "arm"),
        ("lgs", "LGS"),
        ("platforms-cables", "cable|plat|level"),
        ("m1-segments", ""),
    ];

    for part in parts {
        let (name, filter) = part;
        println!("Part: {}", name);

        let _: Vec<_> = data_paths
            .par_iter()
            .map(|arg| {
                let filename = format!("{}/{}.png", arg, name);
                if name == "m1-segments" {
                    let mut m1 = Mirror::m1();
                    if let Err(e) = m1.load(arg, false) {
                        println!("{}: {:}", arg, e);
                    }
                    m1.plot_forces(Some(filename.as_str()));
                } else {
                    let monitors = MonitorsLoader::<CFD_YEAR>::default()
                        .data_path(arg.clone())
                        .header_filter(filter.to_string())
                        //.exclude_filter(xmon)
                        .load()
                        .unwrap();
                    monitors.plot_forces(Some(filename.as_str()));
                }
            })
            .collect();
    }

    Ok(())
}
