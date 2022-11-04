use aws_sdk_s3 as s3;
use google_youtube3::{
    api::{PlaylistItem, PlaylistItemSnippet, ResourceId, Video, VideoSnippet, VideoStatus},
    hyper, hyper_rustls, oauth2, YouTube,
};
use parse_monitors::cfd;
use std::io::Cursor;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfd_case = cfd::Baseline::<2021>::default()
        .into_iter()
        .skip(9) // failed
        .next()
        .unwrap();
    let key = format!("CASES/{}/scenes/RI_tel_RI_tel.mp4", &cfd_case);
    println!("key: {key}");

    let config = aws_config::from_env()
        .region(s3::Region::new("us-east-2"))
        .load()
        .await;
    let client = s3::Client::new(&config);

    let stream = client
        .get_object()
        .bucket("gmto.cfd.2022")
        .key(key)
        .send()
        .await?;
    let data = stream.body.collect().await?;
    let buf = Cursor::new(data.into_bytes());

    let secret = oauth2::read_application_secret("client_secret.json").await?;

    let auth = oauth2::InstalledFlowAuthenticator::builder(
        secret,
        oauth2::InstalledFlowReturnMethod::HTTPRedirect,
    )
    .build()
    .await?;
    let hub = {
        YouTube::new(
            hyper::Client::builder().build(
                hyper_rustls::HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .https_or_http()
                    .enable_http1()
                    .enable_http2()
                    .build(),
            ),
            auth,
        )
    };
    let req = Video {
        status: Some(VideoStatus {
            privacy_status: Some("Public".to_string()),
            self_declared_made_for_kids: Some(true),
            ..Default::default()
        }),
        snippet: Some(VideoSnippet {
            description: Some(cfd_case.to_pretty_string()),
            title: Some(cfd_case.to_string()),
            tags: Some(
                ["Astronomy", "Computational Fluid Dynamics", "Telescope"]
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>(),
            ),
            ..Default::default()
        }),
        ..Default::default()
    };
    let (response, video) = hub
        .videos()
        .insert(req)
        .stabilize(false)
        .notify_subscribers(false)
        .auto_levels(true)
        .upload(buf, "video/*".parse().unwrap())
        .await?;
    println!("{response:#?}");

    let req = PlaylistItem {
        snippet: Some(PlaylistItemSnippet {
            playlist_id: Some("PLTrfhf7NjCR1x_RnJBJi65Ycv4nYpoaK1".to_string()),
            resource_id: Some(ResourceId {
                kind: video.kind,
                video_id: video.id,
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };
    let (response, _) = hub.playlist_items().insert(req).doit().await?;
    println!("{response:#?}");

    Ok(())
}
