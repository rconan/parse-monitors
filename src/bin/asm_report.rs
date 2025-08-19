use std::{fs::File, io::Write, path::Path};
use strum::IntoEnumIterator;

use parse_monitors::{cfd, cfd::BaselineTrait};

fn main() -> anyhow::Result<()> {
    for zenith_angle in cfd::ZenithAngle::iter() {
        let asm_pressure: Vec<_> = cfd::Baseline::<2021>::at_zenith(zenith_angle)
            .into_iter()
            .map(|cfd_case| {
                format!(
                    r#"
\subsection{{{}}}
\label{{{}-pressure}}

\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
"#,
                    &cfd_case.to_pretty_string(),
                    &cfd_case.to_string(),
                    cfd::Baseline::<2021>::path()
                        .unwrap()
                        .join(&cfd_case.to_string())
                        .join("m2_pressure-stats_std_within")
                )
            })
            .collect();
        let report_path = Path::new("report");
        let mut file =
            File::create(report_path.join(format!("{}_asm_pressure.tex", zenith_angle)))?;
        write!(file, "{}", asm_pressure.join("\n"))?;
        let asm_temperature: Vec<_> = cfd::Baseline::<2021>::at_zenith(zenith_angle)
            .into_iter()
            .map(|cfd_case| {
                format!(
                    r#"
\subsection{{{}}}
\label{{{}-temperature}}

\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
"#,
                    &cfd_case.to_pretty_string(),
                    &cfd_case.to_string(),
                    cfd::Baseline::<2021>::path()
                        .unwrap()
                        .join(&cfd_case.to_string())
                        .join("m2_temperature-stats_std_within")
                )
            })
            .collect();
        let report_path = Path::new("report");
        let mut file =
            File::create(report_path.join(format!("{}_asm_temperature.tex", zenith_angle)))?;
        write!(file, "{}", asm_temperature.join("\n"))?;
    }
    Ok(())
}
