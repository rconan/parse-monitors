use std::fs;

use cfd_report::{
    ForcesCli, PREVIOUS_YEAR, ReportOptions, ReportPathError, batch_force, dome_seeing, opd_maps,
    pressure_maps, report,
};
use clap::{Parser, Subcommand};
use parse_monitors::{
    CFD_YEAR,
    cfd::{self, Baseline, BaselineTrait},
};

#[derive(Parser)]
#[command(
    name = "CFD_REPORT",
    about = "Generates plots & Latex files for the CFD baseline reports",
    after_help = r#"
    cfd_report processes the CFD database in order to generate images and plots for the CFD reports and writes the Latex files of the reports in the `/path/to/CFD/case/report` folder.
Paths to the databases need to be provided for both the current and the previous years with:
 - export CFD_<current year>_REPO=/path/to/current/year/database
 - export CFD_<previous year>_REPO=/path/to/previous/year/database
For example, the full report, including all the plots, for the year 2025 will be created with:
```
export CFD_2025_REPO=/home/ubuntu/mnt/CASES
export CFD_2021_REPO=/home/ubuntu/cfd/CASES
cargo r -r -- full
```
But, all the illustrations and not the report for the year 2025 will be created with:
```
export CFD_2025_REPO=/home/ubuntu/mnt/CASES
export CFD_2021_REPO=/home/ubuntu/cfd/CASES
cargo r -r -- all
```
"#
)]
struct Cli {
    /// skip that many CFD cases
    #[arg(short, long)]
    skip: Option<usize>,
    /// process only that many CFD cases
    #[arg(short, long)]
    take: Option<usize>,
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// generates the CFD report plots & Latex files
    #[command(subcommand)]
    Report(ReportOptions),
    /// generates all CFD plots
    All,
    /// generates CFD forces & moments plots
    Forces {
        /// Truncate monitors to the `last` seconds
        #[arg(short, long)]
        last: Option<usize>,
        /// Make all the plots
        #[arg(long)]
        all: bool,
        /// Make C-Rings force magnitude plot
        #[arg(long)]
        crings: bool,
        /// Make M1 cell force magnitude plot
        #[arg(long)]
        m1_cell: bool,
        /// Make upper truss force magnitude plot
        #[arg(long)]
        upper_truss: bool,
        /// Make lower truss force magnitude plot
        #[arg(long)]
        lower_truss: bool,
        /// Make top-end force magnitude plot
        #[arg(long)]
        top_end: bool,
        /// Make M1 segments force magnitude plot
        #[arg(long)]
        m1_segments: bool,
        /// Make M2 segments force magnitude plot
        #[arg(long)]
        m2_segments: bool,
        /// Make M1 and M2 baffles force magnitude plot
        #[arg(long)]
        m12_baffles: bool,
        /// Make M1 inner mirror covers force magnitude plot
        #[arg(long)]
        m1_inner_covers: bool,
        /// Make M1 outer mirror covers force magnitude plot
        #[arg(long)]
        m1_outer_covers: bool,
        /// Make GIR force magnitude plot
        #[arg(long)]
        gir: bool,
        /// Make PFA arms force magnitude plot
        #[arg(long)]
        pfa_arms: bool,
        /// Make Laser Guide Stars assemblies force magnitude plot
        #[arg(long)]
        lgsa: bool,
        /// Make platforms and cables force magnitude plot
        #[arg(long)]
        platforms_cables: bool,
        /// Remove linear trends from monitors
        #[arg(long)]
        detrend: bool,
    },
    /// generates CFD dome seeing OPD maps
    OpdMaps,
    /// generates CFD pressure maps
    PressureMaps,
    /// generates CFD dome seeing time series plots
    DomeSeeing,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    let cfd_cases = match (cli.skip, cli.take) {
        (None, None) => cfd::Baseline::<CFD_YEAR>::default()
            .into_iter()
            .collect::<Vec<cfd::CfdCase<CFD_YEAR>>>(),
        (None, Some(t)) => cfd::Baseline::<CFD_YEAR>::default()
            .into_iter()
            .take(t)
            .collect::<Vec<cfd::CfdCase<CFD_YEAR>>>(),
        (Some(s), None) => cfd::Baseline::<CFD_YEAR>::default()
            .into_iter()
            .skip(s)
            .collect::<Vec<cfd::CfdCase<CFD_YEAR>>>(),
        (Some(s), Some(t)) => cfd::Baseline::<CFD_YEAR>::default()
            .into_iter()
            .skip(s)
            .take(t)
            .collect::<Vec<cfd::CfdCase<CFD_YEAR>>>(),
    };
    let cfd_root = Baseline::<CFD_YEAR>::path()?;
    cfd_cases
        .iter()
        .map(|cfd_case| {
            let path_to_report = cfd_root.join(cfd_case.to_string()).join("report");
            Ok(if !path_to_report.exists() {
                fs::create_dir(&path_to_report)
                    .map_err(|e| ReportPathError::new(path_to_report, e))?;
            })
        })
        .collect::<Result<Vec<()>, ReportPathError>>()?;
    match cli.commands {
        Commands::Report(opt) => report::taks(&cfd_cases, opt)?,
        Commands::All => {
            batch_force::task(&cfd_cases, Default::default())?;
            opd_maps::task(&cfd_cases)?;
            pressure_maps::task::<geotrans::M1, _>(&cfd_cases)?;
            pressure_maps::task::<geotrans::M2, _>(&cfd_cases)?;
            dome_seeing::taks::<PREVIOUS_YEAR, _>(&cfd_cases)?;
        }
        Commands::Forces {
            last,
            all,
            crings,
            m1_cell,
            upper_truss,
            lower_truss,
            top_end,
            m1_segments,
            m2_segments,
            m12_baffles,
            m1_inner_covers,
            m1_outer_covers,
            gir,
            pfa_arms,
            lgsa,
            platforms_cables,
            detrend,
        } => {
            let forces_cli = ForcesCli {
                last,
                all,
                crings,
                m1_cell,
                upper_truss,
                lower_truss,
                top_end,
                m1_segments,
                m2_segments,
                m12_baffles,
                m1_inner_covers,
                m1_outer_covers,
                gir,
                pfa_arms,
                lgsa,
                platforms_cables,
                detrend,
            };
            batch_force::task(&cfd_cases, forces_cli)?;
        }
        Commands::OpdMaps => opd_maps::task(&cfd_cases)?,
        Commands::PressureMaps => {
            pressure_maps::task::<geotrans::M1, _>(&cfd_cases)?;
            pressure_maps::task::<geotrans::M2, _>(&cfd_cases)?;
        }
        Commands::DomeSeeing => dome_seeing::taks::<PREVIOUS_YEAR, _>(&cfd_cases)?,
    };
    Ok(())
}
