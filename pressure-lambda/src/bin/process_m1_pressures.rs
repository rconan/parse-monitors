use glob::glob;
use lambda::{
    model::{Environment, FunctionCode, Runtime},
    output::InvokeOutput,
    Blob, Client,
};
use parse_monitors::cfd;
use serde::Deserialize;
use std::{path::Path, time::Instant};

type Arr = [f64; 3];
#[derive(Debug, Deserialize)]
pub struct COPFM {
    pub cop_fm: Vec<(Arr, (Arr, Arr))>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let shared_config = aws_config::from_env().region("us-east-2").load().await;
    let lambda = Client::new(&shared_config);
    let function_name = "rustTest".to_string();
    for cfd_case in cfd::Baseline::<2021>::default()
        .extras()
        .into_iter()
        .take(1)
    {
        /*
                let function_name = "m1_pressure".to_string();
                let zip_file = Blob::new(std::fs::read("lambda.zip").unwrap());
                lambda
                    .create_function()
                    .function_name(&function_name)
                    .code(FunctionCode::builder().zip_file(zip_file).build())
                    .handler("doesnt.matter")
                    .runtime(Runtime::Providedal2)
                    .role("arn:aws:iam::378722409401:role/lambda_basic_execution")
                    .environment(
                        Environment::builder()
                            .variables("RUST_BACKTRACE", "1")
                            .build(),
                    )
                    .timeout(60)
                    .send()
                    .await
                    .unwrap();
        */
        println!("Processing {} ...", &cfd_case);

        let case_path = cfd::Baseline::<2021>::path().join(cfd_case.to_string());
        let files: Vec<_> = glob(
            case_path
                .join("pressures")
                .join("M1p_M1p_*.csv.bz2")
                .to_str()
                .unwrap(),
        )
        .unwrap()
        .map(|p| p.unwrap().to_str().unwrap().to_string())
        .collect();

        println!(" - found {} files!", files.len());

        /*
           let mut wtr = csv::Writer::from_path(case_path.join("M1_segments_force.csv")).unwrap();
           let keys: Vec<_> = std::iter::once("Time [s]".to_string())
               .chain((1..=7).flat_map(|sid| {
                   ["X", "Y", "Z"]
                       .iter()
                       .map(|xyz| format!("M1 S{} FORCE {} [N]", sid, xyz))
                       .collect::<Vec<String>>()
               }))
               .collect();
           wtr.write_record(&keys).unwrap();
        */

        let now = Instant::now();
        let mut handle = vec![];

        for (k, some_files) in files.chunks(100).map(|x| x.to_owned()).enumerate().take(5) {
            let function_name = format!("m1_pressure_{}", k);
            let zip_file = Blob::new(std::fs::read("lambda.zip").unwrap());
            lambda
                .create_function()
                .function_name(&function_name)
                .code(FunctionCode::builder().zip_file(zip_file).build())
                .handler("doesnt.matter")
                .runtime(Runtime::Providedal2)
                .role("arn:aws:iam::378722409401:role/lambda_basic_execution")
                .environment(
                    Environment::builder()
                        .variables("RUST_BACKTRACE", "1")
                        .build(),
                )
                .timeout(60)
                .send()
                .await
                .unwrap();

            for file in some_files {
                let client = lambda.clone();
                let name = function_name.clone();
                handle.push(tokio::spawn(async move {
                    let path = Path::new(&file);
                    let key = path.strip_prefix("/fsx/").unwrap();
                    let payload = format!(r#"{{"key": {:?}}}"#, key);
                    let blob = Blob::new(payload);
                    client
                        .invoke()
                        .function_name(name)
                        .payload(blob)
                        .send()
                        .await
                        .unwrap()
                    /*
                                    match resp.payload {
                                        Some(blob) => {
                                            let stem = Path::new(path.file_stem().unwrap())
                                                .file_stem()
                                                .unwrap()
                                                .to_str()
                                                .unwrap();
                                            let time = &stem[8..].parse::<f64>();
                                            let s = std::str::from_utf8(blob.as_ref()).expect("invalid utf-8");
                                            let v: serde_json::Value = serde_json::from_str(s).unwrap();
                                            v["segments integrated force"].as_array().map(|x| {
                                                time.iter()
                                                    .chain(
                                                        x.iter()
                                                            .cloned()
                                                            .flat_map(|x| {
                                                                x.as_array()
                                                                    .unwrap()
                                                                    .iter()
                                                                    .filter_map(|x| x.as_f64())
                                                                    .collect::<Vec<f64>>()
                                                            })
                                                            .collect::<Vec<f64>>()
                                                            .iter(),
                                                    )
                                                    .cloned()
                                                    .collect::<Vec<f64>>()
                                            })
                                        }
                                        _ => None,
                                    }
                    */
                }));
            }
            println!(
                " - {} lambdas invoked in {}ms",
                handle.len(),
                now.elapsed().as_millis()
            );

            /*            lambda
            .delete_function()
            .function_name(function_name)
            .send()
            .await
            .unwrap();*/
        }
        let mut cop_fm: Vec<Option<COPFM>> = vec![];
        let now = Instant::now();
        for h in handle {
            if let Ok(InvokeOutput {
                payload: Some(data),
                ..
            }) = h.await
            {
                let s = std::str::from_utf8(data.as_ref()).expect("invalid utf-8");
                cop_fm.push(serde_json::from_str(s).ok());
                /*
                        wtr.write_record(
                            f.into_iter()
                                .map(|x| x.to_string())
                                .collect::<Vec<String>>(),
                        )
                        .unwrap();
                */
            }
        }
        //wtr.flush().unwrap();
        println!(" - {} computed in {}s", cfd_case, now.elapsed().as_secs());
        println!("{:?}", cop_fm[0]);
    }
    Ok(())
}
