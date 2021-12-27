use parse_monitors::{Mirror, MonitorsLoader};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "parse-monitors", about = "Parsing Star-CCM+ monitors")]
struct Opt {
    /// Path to the monitor file repository
    path: String,
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
    /// Truncate monitors to the `last` seconds
    #[structopt(short, long)]
    last: Option<usize>,
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
    /// Display M1 force table summary
    #[structopt(long)]
    m1_table: bool,
    /// Display M1 net force table summary
    #[structopt(long)]
    m1_table_net: bool,
    /// Remove linear trends from monitors
    #[structopt(long)]
    detrend: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();

    let mut loader = MonitorsLoader::<2021>::default();
    loader = loader.data_path(&opt.path);
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
    if let Some(arg) = opt.last {
        monitors.keep_last(arg);
    }
    if opt.detrend {
        monitors.detrend();
    }
    if opt.local {
        monitors.into_local();
    }
    monitors.summary();
    if opt.plot {
        monitors.plot_htc();
        monitors.plot_forces(None);
        monitors.plot_moments(None);
    }

    if let Some(filename) = opt.csv {
        monitors.to_csv(filename)?;
    }
    #[cfg(feature = "windloading")]
    if opt.m1_covers {
        monitors.m1covers_windloads()?;
    }

    if opt.m1_table {
        if let Some(arg) = opt.start {
            Mirror::m1(&opt.path).start_time(arg).load()?.summary();
        } else {
            Mirror::m1(&opt.path).load()?.summary();
        };
    }
    if opt.m1_table_net {
        if let Some(arg) = opt.start {
            Mirror::m1(&opt.path)
                .net_force()
                .start_time(arg)
                .load()?
                .summary();
        } else {
            Mirror::m1(&opt.path).net_force().load()?.summary();
        };
    }

    Ok(())
}
