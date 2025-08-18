/*!
# GMT FEM Wind Loading

```shell
export MOUNT_MODEL=MOUNT_FDR_1kHz
export FEM_REPO=$HOME/mnt/20250506_1715_zen_30_M1_202110_FSM_202305_Mount_202305_pier_202411_M1_actDamping/
export CFD_REPO=$HOME/maua/CASES
cargo r -r
```
*/

use std::{collections::HashMap, env, fs, path::Path};

use clap::Parser;
use gmt_dos_actors::{
    actor::Terminator,
    actorscript,
    model::Model,
    prelude::{AddActorOutput, AddOuput, IntoLogs, TryIntoInputs, vec_box},
    system::Sys,
};
use gmt_dos_clients_arrow::Arrow;
use gmt_dos_clients_io::{
    cfd_wind_loads::{CFDM1WindLoads, CFDM2WindLoads, CFDMountWindLoads},
    gmt_m1::M1RigidBodyMotions,
    gmt_m2::M2RigidBodyMotions,
};
use gmt_dos_clients_servos::{GmtFem, GmtServoMechanisms, WindLoads};
use gmt_dos_clients_windloads::{
    CfdLoads,
    system::{M1, M2, Mount, SigmoidCfdLoads},
};
use gmt_fem::FEM;
use parse_monitors::{
    CFD_YEAR,
    cfd::{Baseline, BaselineTrait, CfdCase},
};
use tokio::task::JoinSet;

const ACTUATOR_RATE: usize = 80;

#[derive(Debug, Parser)]
#[command(
    name = "WIND LOADS",
    about = "Computes M1 & M2 RBMs from CFD wind loads applied to the GMT FEM"
)]
struct Cli {
    /// skip that many CFD cases
    #[arg(short, long)]
    skip: Option<usize>,
    /// process only that many CFD cases
    #[arg(short, long)]
    take: Option<usize>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    {
        let path_to_fem_cfd = Path::new(env!("FEM_REPO")).join("cfd");
        if !path_to_fem_cfd.exists() {
            fs::create_dir(path_to_fem_cfd)?;
        }
    }
    let mut set = JoinSet::new();

    match (cli.skip, cli.take) {
        (None, None) => Box::new(Baseline::<CFD_YEAR>::default().into_iter())
            as Box<dyn Iterator<Item = CfdCase<{ CFD_YEAR }>>>,
        (None, Some(t)) => Box::new(Baseline::<CFD_YEAR>::default().into_iter().take(t))
            as Box<dyn Iterator<Item = CfdCase<{ CFD_YEAR }>>>,
        (Some(s), None) => Box::new(Baseline::<CFD_YEAR>::default().into_iter().skip(s))
            as Box<dyn Iterator<Item = CfdCase<{ CFD_YEAR }>>>,
        (Some(s), Some(t)) => Box::new(Baseline::<CFD_YEAR>::default().into_iter().skip(s).take(t))
            as Box<dyn Iterator<Item = CfdCase<{ CFD_YEAR }>>>,
    }
    .for_each(|cfd_case| {
        set.spawn(task(cfd_case));
    });

    while let Some(res) = set.join_next().await {
        let Ok(res) = res else {
            return res?;
        };
        res?;
    }

    Ok(())
}

async fn task<const Y: u32>(cfd_case: CfdCase<Y>) -> anyhow::Result<()> {
    unsafe {
        env::set_var("DATA_REPO", Path::new(env!("FEM_REPO")).join("cfd"));
    }

    let path = Baseline::<Y>::path()?.join(cfd_case.to_string());
    println!("{path:?}");

    let sim_sampling_frequency = 1000;
    let sim_duration = 400_usize; // second
    let n_step = sim_sampling_frequency * sim_duration;

    let mut fem = Option::<FEM>::None;

    let cfd_loads: Sys<SigmoidCfdLoads> =
        CfdLoads::foh(path.as_os_str().to_str().unwrap(), sim_sampling_frequency)
            .duration(sim_duration as f64)
            .windloads(
                fem.get_or_insert(FEM::from_env().unwrap()),
                Default::default(),
            )
            .try_into()?;

    let gmt_servos: Sys<GmtServoMechanisms<ACTUATOR_RATE, 1>> =
        GmtServoMechanisms::<ACTUATOR_RATE, 1>::new(sim_sampling_frequency as f64, fem.unwrap())
            .wind_loads(WindLoads::new())
            .try_into()?;

    if cfg!(feature = "api") {
        log::info!("API");
        // build the model using the traditional API
        let fem_tag = Path::new(env!("FEM_REPO"))
            .components()
            .last()
            .unwrap()
            .as_os_str()
            .to_string_lossy();
        let mut metadata = HashMap::new();
        metadata.insert("GMT FEM".to_string(), fem_tag.clone().into_owned());
        let data_path = Path::new(&cfd_case.to_string()).to_path_buf();
        let full_data_path = Path::new(env::var("DATA_REPO").as_ref().unwrap()).join(&data_path);
        if !full_data_path.exists() {
            fs::create_dir(full_data_path)?;
        }
        let mut rbms_logger: Terminator<Arrow> = Arrow::builder(n_step)
            .filename(
                data_path
                    .join("m1_m2_rbms.parquet")
                    .to_str()
                    .unwrap()
                    .to_string(),
            )
            .metadata(metadata)
            .build()
            .into();

        let mut gmt_servos_clone = gmt_servos.clone();
        let mut cfd_loads_clone = cfd_loads.clone();
        AddActorOutput::<Mount, _, _>::add_output(&mut cfd_loads_clone)
            .build::<CFDMountWindLoads>()
            .into_input::<GmtFem>(&mut gmt_servos_clone)?;
        AddActorOutput::<M1, _, _>::add_output(&mut cfd_loads_clone)
            .build::<CFDM1WindLoads>()
            .into_input::<GmtFem>(&mut gmt_servos_clone)?;
        AddActorOutput::<M2, _, _>::add_output(&mut cfd_loads_clone)
            .build::<CFDM2WindLoads>()
            .into_input::<GmtFem>(&mut gmt_servos_clone)?;

        AddActorOutput::<GmtFem, _, _>::add_output(&mut gmt_servos_clone)
            .unbounded()
            .build::<M1RigidBodyMotions>()
            .log(&mut rbms_logger)?;
        AddActorOutput::<GmtFem, _, _>::add_output(&mut gmt_servos_clone)
            .unbounded()
            .build::<M2RigidBodyMotions>()
            .log(&mut rbms_logger)?;

        Model::new(vec_box!(cfd_loads_clone, gmt_servos_clone, rbms_logger))
            .quiet()
            .check()?
            .run()
            .await?;
    } else {
        // build the model using the actorscript macro
        actorscript! {
            1: {cfd_loads::Mount}[CFDMountWindLoads] -> {gmt_servos::GmtFem}
            1: {cfd_loads::M1}[CFDM1WindLoads] -> {gmt_servos::GmtFem}[M1RigidBodyMotions]$
            1: {cfd_loads::M2}[CFDM2WindLoads] -> {gmt_servos::GmtFem}[M2RigidBodyMotions]$
        }
    }
    Ok(())
}
