use parse_monitors::{cfd, cfd::BaselineTrait};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "lscfd")]
struct Opt {
    /// Zenith angle
    #[structopt(short, long, default_value = "30")]
    zenith: u32,
    /// Case file
    #[structopt(long)]
    file: Option<String>,
}

fn main() {
    let opt = Opt::from_args();

    let file = opt.file.unwrap_or_default();
    let cfd_cases: Vec<_> =
        cfd::Baseline::<2021>::at_zenith(cfd::ZenithAngle::new(opt.zenith).unwrap())
            .into_iter()
            .map(|case| format!("{}/{}", case.to_string(), file))
            .collect();
    println!("{}", cfd_cases.join(" "))
}
