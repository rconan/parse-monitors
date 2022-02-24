use parse_monitors::{cfd, Monitors, Vector};
use rayon::prelude::*;
use std::{
    fs::{create_dir, File},
    io::Write,
    path::{Path, PathBuf},
};

fn main() -> anyhow::Result<()> {
    let stats = |x: &[Vector]| -> Option<(Vec<f64>, Vec<f64>)> {
        let n = x.len() as f64;
        if let Some(mean) = x.iter().fold(Vector::zero(), |s, x| s + x) / n {
            let std = x
                .iter()
                .filter_map(|x| x - mean.clone())
                .filter_map(|x| -> Option<Vec<f64>> { x.into() })
                .fold(vec![0f64; 3], |mut a, x| {
                    a.iter_mut().zip(x.iter()).for_each(|(a, &x)| *a += x * x);
                    a
                });
            let mean: Option<Vec<f64>> = mean.into();
            Some((
                mean.unwrap(),
                std.iter()
                    .map(|x| (*x / n as f64).sqrt())
                    .collect::<Vec<_>>(),
            ))
        } else {
            None
        }
    };

    let parts = vec![
        "C-Rings",
        "GIR",
        "LGS",
        "M1 assembly",
        "M1 covers",
        r#"M2 \& Top-End"#,
        "Trusses",
        r#"Platform \& Trays"#,
    ];

    let (latex,graphics): (Vec<_>,Vec<_>) = cfd::Baseline::<2021>::default().into_iter().collect::<Vec<cfd::CfdCase<2021>>>().into_par_iter().map(|cfd_case| {
        println!("{cfd_case}");
	let mut latex = vec![];
        let mut appendix_graphics= vec![format!(
            r#"
\section{{{}}}
"#,
            cfd_case.to_pretty_string(),
)];
        latex.push(format!(
            r#"\midrule\multicolumn{{13}}{{l}}{{{}}}\\\hline"#,
            cfd_case.to_pretty_string()
        ));
        let data_path = cfd::Baseline::<2021>::path().join(cfd_case.to_string());
	let report_path = data_path.join("report");
	if !report_path.is_dir() {
            create_dir(&report_path).unwrap()
	}

        let mut total_forces: Option<Vec<Vector>> = None;
        let mut total_moments: Option<Vec<Vector>> = None;

        for i in 0..8 {
            let mut groups = vec![
                "Cring",
                "GIR",
                "LGS",
                "M1",
                "M1cov",
                "M2|Topend",
                "Tbot|Tup|arm|cabletruss",
                "cabletrays|platform|Cabs",
            ];
            let group = groups.remove(i);
            if i == 4 {
                groups.remove(3);
            }
            groups.push("M1level");

            /*let mon = if cfd_case == cfd::CfdCase::colloquial(30, 135, "nos", 7).unwrap() {
                Monitors::loader::<PathBuf, 2021>(data_path.clone())
                    .exclude_filter(groups.join("|"))
                    //.header_filter("M1")
                    .header_filter(group)
                    //.exclude_filter("floor|enclosure|screen|shutter")
                    .start_time(200.)
                    .end_time(340.)
                    .load().unwrap()
            } else {*/
                let mut mon = Monitors::loader::<PathBuf, 2021>(data_path.clone())
                    .exclude_filter(groups.join("|"))
                    //.header_filter("M1")
                    .header_filter(group)
                    //.exclude_filter("floor|enclosure|screen|shutter")
                    .load().unwrap();
                mon.keep_last(400);
                /*mon
            };*/
            /*
                        println!("{:?}", mon.forces_and_moments.keys());
                        println!(
                            "{:?}",
                            mon.forces_and_moments
                                .values()
                                .filter_map(|x| x.get(0).unwrap().force.magnitude())
                                .collect::<Vec<_>>()
                        );
            */
            //mon.summary();
            let n = mon.len();
            let (part_total_forces, part_total_moments): (Vec<_>, Vec<_>) =
                mon.forces_and_moments.values().fold(
                    (vec![Vector::zero(); n], vec![Vector::zero(); n]),
                    |(mut fa, mut ma), value| {
                        fa.iter_mut()
                            .zip(value.iter())
                            .for_each(|(mut fa, e)| fa += &e.force);
                        ma.iter_mut()
                            .zip(value.iter())
                            .for_each(|(mut ma, e)| ma += &e.moment);
                        (fa, ma)
                    },
                );
            match (stats(&part_total_forces), stats(&part_total_moments)) {
                (Some((f_mean, f_std)), Some((m_mean, m_std))) => {
                    latex.push(format!(
                        r#"{:>20} & {:.0} & {:.0} & {:.0} & {:.0} & {:.0} & {:.0} & {:.0} & {:.0} & {:.0} & {:.0} & {:.0} & {:.0} \\"#,
                        parts[i],
                        f_mean[0],
                        f_mean[1],
                        f_mean[2],
                        m_mean[0],
                        m_mean[1],
                        m_mean[2],
                        f_std[0],
                        f_std[1],
                        f_std[2],
                        m_std[0],
                        m_std[1],
                        m_std[2],
                    ));
                }
                _ => (),
            };
	    let filename = format!("{}_total_forces_psds.png",group.replace("|","-").to_lowercase());
            let filepath = format!(
            "{:}",
            data_path
                .clone()
                .join("report")
                .join(filename.clone())
                .to_str()
                .unwrap()
                .to_string()
            );
	    let plot = complot::Config::new()
		.filename(filepath).
		xaxis(complot::Axis::new().label("Frequency [Hz]")).
		yaxis(complot::Axis::new().label("FORCE PSD [N^2/Hz]"))
		.legend(vec!["Fx","Fy","Fz"]);
            Monitors::plot_this_forces_psds(part_total_forces.as_slice(),
					    Some(plot));
	    appendix_graphics.push(format!(
            r#"
\subsection{{{}}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
"#,
	    parts[i],
            data_path
                .join("report")
                .join(filename)
                .with_extension(""),
	));
            if let Some(total_forces) = total_forces.as_mut() {
                total_forces
                    .iter_mut()
                    .zip(part_total_forces.as_slice())
                    .for_each(|(mut t, p)| t += p);
            } else {
                total_forces = Some(part_total_forces);
            }
            if let Some(total_moments) = total_moments.as_mut() {
                total_moments
                    .iter_mut()
                    .zip(part_total_moments.as_slice())
                    .for_each(|(mut t, p)| t += p);
            } else {
                total_moments = Some(part_total_moments);
            }
        }
        match (
            stats(total_forces.as_ref().unwrap()),
            stats(total_moments.as_ref().unwrap()),
        ) {
            (Some((f_mean, f_std)), Some((m_mean, m_std))) => {
                latex.push(format!(
                    r#"{:>20} & {:.0} & {:.0} & {:.0} & {:.0} & {:.0} & {:.0} & {:.0} & {:.0} & {:.0} & {:.0} & {:.0} & {:.0} \\"#,
		    "Total",
                        f_mean[0],
                        f_mean[1],
                        f_mean[2],
                        m_mean[0],
                        m_mean[1],
                        m_mean[2],
                        f_std[0],
                        f_std[1],
                        f_std[2],
                        m_std[0],
                        m_std[1],
                        m_std[2],
                ));
            }
            _ => (),
        };
        let filename = format!(
            "{:}",
            data_path
                .clone()
                .join("report")
                .join("total_forces.png")
                .to_str()
                .unwrap()
                .to_string()
        );
	    let plot = complot::Config::new()
		.filename(filename).
		xaxis(complot::Axis::new().label("Time [s]")).
		yaxis(complot::Axis::new().label("FORCE [N]"))
		.legend(vec!["Fx","Fy","Fz"]);
        Monitors::plot_this_forces(total_forces.as_ref().unwrap(),
				   Some(plot));
        let filename = format!(
            "{:}",
            data_path
                .clone()
                .join("report")
                .join("total_forces_psds.png")
                .to_str()
                .unwrap()
                .to_string()
        );
	    let plot = complot::Config::new()
		.filename(filename).
		xaxis(complot::Axis::new().label("Frequency [Hz]")).
		yaxis(complot::Axis::new().label("FORCE PSD [N^2/Hz]"))
		.legend(vec!["Fx","Fy","Fz"]);
        Monitors::plot_this_forces_psds(total_forces.as_ref().unwrap(),
					Some(plot));
        let filename = format!(
            "{:}",
            data_path
                .clone()
                .join("report")
                .join("total_moments.png")
                .to_str()
                .unwrap()
                .to_string()
        );
	    let plot = complot::Config::new()
		.filename(filename).
		xaxis(complot::Axis::new().label("Time [s]")).
		yaxis(complot::Axis::new().label("MOMENT [N.m]"))
		.legend(vec!["Mx","My","Mz"]);
        Monitors::plot_this_forces(total_moments.as_ref().unwrap(),
				   Some(plot));
        let filename = format!(
            "{:}",
            data_path
                .clone()
                .join("report")
                .join("total_moments_psds.png")
                .to_str()
                .unwrap()
                .to_string()
        );
	    let plot = complot::Config::new()
		.filename(filename).
		xaxis(complot::Axis::new().label("Frequency [Hz]")).
		yaxis(complot::Axis::new().label("MOMENT PSD [(N.m)^2/Hz]"))
		.legend(vec!["Mx","My","Mz"]);
        Monitors::plot_this_forces_psds(total_moments.as_ref().unwrap(),
					Some(plot));

        let graphics= format!(
            r#"
\section{{{}}}
\subsection{{Total forces}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\\
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}

\subsection{{Total moments}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\\
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
"#,
            cfd_case.to_pretty_string(),
            data_path
                .join("report")
                .join("total_forces.png")
                .with_extension(""),
            data_path
                .join("report")
                .join("total_forces_psds.png")
                .with_extension(""),
            data_path
                .join("report")
                .join("total_moments.png")
                .with_extension(""),
            data_path
                .join("report")
                .join("total_moments_psds.png")
                .with_extension("")
        );
	(latex.join("\n"),(graphics,appendix_graphics.join("\n")))
    }).unzip();

    let (graphics, appendix_graphics): (Vec<_>, Vec<_>) = graphics.into_iter().unzip();

    let table = format!(
        r#"
\begin{{longtable}}{{r|*{{3}}{{r}}|*{{3}}{{r}}|*{{3}}{{r}}|*{{3}}{{r}}}}\toprule
 ELEMENT & \multicolumn{{3}}{{|c|}}{{Force [N]}} & \multicolumn{{3}}{{|c|}}{{Moment [N.m]}} & \multicolumn{{3}}{{|c|}}{{Force [N]}} & \multicolumn{{3}}{{|c|}}{{Moment [N.m]}} \\
    - & $\langle F_x \rangle$ & $\langle F_y \rangle$ & $\langle F_z \rangle$ & $\langle M_x \rangle$ & $\langle M_y \rangle$ & $\langle M_z \rangle$ & $\sigma_{{F_x}}$ & $\sigma_{{F_y}}$ & $\sigma_{{F_z}}$ & $\sigma_{{M_x}}$ & $\sigma_{{M_y}}$ & $\sigma_{{M_z}}$ \\
\endfirsthead

    - & $\langle F_x \rangle$ & $\langle F_y \rangle$ & $\langle F_z \rangle$ & $\langle M_x \rangle$ & $\langle M_y \rangle$ & $\langle M_z \rangle$ & $\sigma_{{F_x}}$ & $\sigma_{{F_y}}$ & $\sigma_{{F_z}}$ & $\sigma_{{M_x}}$ & $\sigma_{{M_y}}$ & $\sigma_{{M_z}}$ \\
\endhead

\multicolumn{{13}}{{r}}{{{{\emph{{Continued on next page}}}}}} \\ 
  \endfoot

\bottomrule
\endlastfoot

{}
\caption{{Wind loads statistics.}}
\label{{tab:windloads}}
\end{{longtable}}
"#,
        latex.join("\n"),
    );

    let report_path = Path::new("report");
    let mut file = File::create(report_path.join("mount.groups.table.tex"))?;
    write!(file, "{}", table)?;

    let mut file = File::create(report_path.join("mount.time-series.tex"))?;
    write!(file, "{}", graphics.join("\n"))?;

    let mut file = File::create(report_path.join("mount.psds.appendix.tex"))?;
    write!(file, "{}", appendix_graphics.join("\n"))?;

    Ok(())
}
