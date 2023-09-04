use geotrans::*;
use plotters::prelude::*;

const ASM_R: f64 = 0.5;
const R: f64 = 1.2;

fn outline(dr: f64, radius: f64) -> impl Iterator<Item = (f64, f64)> {
    let n = (2. * std::f64::consts::PI * radius / dr).round() as usize;
    (0..=n).map(move |i| {
        let o = 2. * std::f64::consts::PI * (i as f64) / (n as f64);
        let (y, x) = o.sin_cos();
        (radius * x, radius * y)
    })
}

fn main() -> anyhow::Result<()> {
    let drawing_area = BitMapBackend::new("asm-es_geom.png", (768, 768)).into_drawing_area();
    drawing_area.fill(&WHITE).unwrap();
    let lim = 1.6_f64;
    let mut chart = ChartBuilder::on(&drawing_area)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .build_cartesian_2d(-lim..lim, -lim..lim)
        .unwrap();
    chart.configure_mesh().draw().unwrap();

    for sid in 1..=7 {
        chart
            .draw_series(LineSeries::new(
                outline(1e-2, ASM_R).map(|(x, y)| {
                    let u = [x, y, 0.].to(Segment::<M2>::new(sid)).unwrap();
                    (u[0], u[1])
                }),
                &BLACK,
            ))
            .unwrap();
    }
    chart
        .draw_series(LineSeries::new(outline(1e-2, R), &RED))
        .unwrap();

    Ok(())
}
