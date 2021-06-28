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
    /// Monitors start time
    #[structopt(short, long)]
    start: Option<f64>,
    /// Monitors end time
    #[structopt(short, long)]
    end: Option<f64>,
    /// Plot monitors
    #[structopt(short, long)]
    plot: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    //println!("{:?}", opt);

    let mut loader = MonitorsLoader::default();
    if let Some(arg) = opt.path {
        loader = loader.data_path(arg);
    }
    if let Some(arg) = opt.monitor {
        loader = loader.header_filter(arg);
    }
    if let Some(arg) = opt.start {
        loader = loader.start_time(arg);
    }
    if let Some(arg) = opt.end {
        loader = loader.end_time(arg);
    }

    let monitors = loader.load()?;
    monitors.summary();
    if opt.plot {
        monitors.plot_htc();
        monitors.plot_forces();
        monitors.plot_moments();
    }

    Ok(())
}
