use colorous;
use parse_monitors::{cfd, pressure, CFD_YEAR};
use plotters::element::Cubiod;
use plotters::prelude::*;
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    let case = cfd::CfdCase::colloquial(30, 0, "os", 7)?;
    println!("{}", case);
    let paths = cfd::CfdDataFile::<{ CFD_YEAR }>::TelescopePressure.glob(case)?;
    let data_file = paths.last().unwrap();
    println!("{:?}", data_file);
    let now = Instant::now();
    let telp = pressure::Telescope::from_path(data_file)?;
    println!("Data loaded in {}ms", now.elapsed().as_millis());
    println!("{}", telp);

    let root = BitMapBackend::new("telescope-pressure.png", (1024, 1024)).into_drawing_area();
    root.fill(&WHITE)?;

    let xy_axis = (-18.0..18.0).step(0.5);
    let z_axis = (-10.5..25.5).step(0.5);

    let mut chart = ChartBuilder::on(&root)
        .caption(String::from(&telp.filename), ("sans", 20))
        .build_cartesian_3d(xy_axis.clone(), z_axis.clone(), xy_axis.clone())?;
    chart.with_projection(|mut pb| {
        pb.yaw = 0.; //-std::f64::consts::FRAC_PI_2;
        pb.pitch = 0.; //std::f64::consts::FRAC_PI_2;
        pb.scale = 0.9;
        pb.into_matrix()
    });
    chart.configure_axes().draw()?;

    let cmap = colorous::PLASMA;
    let (pmin, pmax) = telp.minmax_pressure().unwrap();
    chart.draw_series(
        telp.xyz_iter()
            .zip(telp.area_mag().into_iter().map(|x| 0.5 * (0.5 * x).cbrt()))
            .zip(telp.pressure_iter())
            .step_by(10)
            .map(|((v, d), p)| {
                let u = (p - pmin) / (pmax - pmin);
                let c = cmap.eval_continuous(u).as_tuple();
                let rgb = RGBColor(c.0, c.1, c.2);
                //                let d = 0.1;
                Cubiod::new(
                    [
                        (v[0] - d, v[2] - d, v[1] - d),
                        (v[0] + d, v[2] + d, v[1] + d),
                    ],
                    rgb,
                    rgb,
                )

                //Circle::new((v[0], v[2], v[1]), 1, rgb)
            }),
    )?;

    Ok(())
}
