use parse_monitors::MonitorsLoader;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "parse-monitors", about = "Parsing Star-CCM+ monitors")]
struct Opt {
    /// Path to the monitor file repository
    #[structopt(long)]
    path: Option<String>,
    /// Monitors regular expression filter
    #[structopt(short, long)]
    monitor: Option<String>,
    /// Monitors exclude regular expression filter
    #[structopt(short = "x", long)]
    exclude: Option<String>,
    /// Monitors start time
    #[structopt(short, long)]
    start: Option<f64>,
    /// Monitors end time
    #[structopt(short, long)]
    end: Option<f64>,
    /// Save monitors to CSV file
    #[structopt(long)]
    csv: Option<String>,
    /// Plot monitors
    #[structopt(short, long)]
    plot: bool,
    /// Evaluates the moments at the part location instead of the OSS
    #[structopt(long)]
    local: bool,
    /// Write M1 mirror covers loads to `windloads.pkl`
    #[structopt(long = "m1-covers")]
    m1_covers: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    //println!("{:?}", opt);

    let mut loader = MonitorsLoader::<2021>::default();
    if let Some(arg) = opt.path {
        loader = loader.data_path(arg);
    }
    if let Some(arg) = opt.monitor {
        loader = loader.header_filter(arg);
    }
    if let Some(arg) = opt.exclude {
        loader = loader.exclude_filter(arg);
    }
    if let Some(arg) = opt.start {
        loader = loader.start_time(arg);
    }
    if let Some(arg) = opt.end {
        loader = loader.end_time(arg);
    }

    let mut monitors = loader.load()?;
    if opt.local {
        monitors.into_local();
    }
    monitors.summary();
    if opt.plot {
        monitors.plot_htc();
        monitors.plot_forces();
        monitors.plot_moments();
    }

    if let Some(filename) = opt.csv {
        monitors.to_csv(filename)?;
    }

    if opt.m1_covers {
        monitors.m1covers_windloads()?;
    }

    Ok(())
}
