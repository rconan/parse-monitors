use glob::glob;
use lambda::{
    model::{Environment, FunctionCode, Runtime},
    Blob, Client,
};
use parse_monitors::cfd;
use std::{path::Path, time::Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let shared_config = aws_config::from_env().region("us-east-2").load().await;
    let lambda = Client::new(&shared_config);
    let function_name = "rustTest".to_string();
    for cfd_case in cfd::Baseline::<2021>::default()
        .extras()
        .into_iter()
        .skip(2)
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

        for a_thousand_files in files.chunks(500).map(|x| x.to_owned()) {
            let now = Instant::now();
            let mut handle = vec![];
            for file in a_thousand_files {
                let client = lambda.clone();
                let name = function_name.clone();
                handle.push(tokio::spawn(async move {
                    let path = Path::new(&file);
                    let key = path.strip_prefix("/fsx/").unwrap();
                    let payload = format!(r#"{{"key": {:?}}}"#, key);
                    let blob = Blob::new(payload);
                    let resp = client
                        .invoke()
                        .function_name(name)
                        .payload(blob)
                        .send()
                        .await
                        .unwrap();
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
                            let f: Vec<f64> = time
                                .iter()
                                .chain(
                                    v["segments integrated force"]
                                        .as_array()
                                        .unwrap()
                                        .iter()
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
                                .collect();
                            Some(f)
                        }
                        _ => None,
                    }
                }))
            }
            println!(
                " - {} lambdas invoked in {}ms",
                handle.len(),
                now.elapsed().as_millis()
            );
            let now = Instant::now();
            for h in handle {
                if let Some(f) = h.await.unwrap() {
                    //            forces.push(f);
                    wtr.write_record(
                        f.into_iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<String>>(),
                    )
                    .unwrap();
                }
            }
            wtr.flush().unwrap();
            println!(
                " - data saved to {}/M1_segments_force.csv in {}s",
                cfd_case,
                now.elapsed().as_secs()
            );
        }
        /*
                lambda
                    .delete_function()
                    .function_name(function_name)
                    .send()
                    .await
                    .unwrap();
        */
    }
    Ok(())
}
