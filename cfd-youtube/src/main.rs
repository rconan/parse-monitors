use aws_sdk_s3 as s3;
use google_youtube3::{
    api::{PlaylistItem, PlaylistItemSnippet, ResourceId, Video, VideoSnippet, VideoStatus},
    hyper, hyper_rustls, oauth2, YouTube,
};
use parse_monitors::cfd;
use std::io::Cursor;

// MP4 file name
//  . Gradient of the index of refracton
const MP4_FILENAME: &str = "RI_tel_RI_tel";
//  . Vorticity
// const MP4_FILENAME: &str = "vort_tel_vort_tel";
// Playlist ID
//  . dome seeing
// const PLAYLIST_ID: &str = "PLTrfhf7NjCR1x_RnJBJi65Ycv4nYpoaK1";
//  . wind loading
const PLAYLIST_ID: &str = "PLTrfhf7NjCR0DqiBP_XcvI-nzQbq3PO1L";
const AWS_REGION: &str = "sa-east-1";
const CFD_YEAR: u32 = 2025;
const S3_BUCKET: &str = "maua.cfd.2025";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    let config = aws_config::from_env()
        .region(s3::Region::new(AWS_REGION))
        .load()
        .await;
    let client = s3::Client::new(&config);

    for cfd_case in cfd::Baseline::<CFD_YEAR>::default()
        .into_iter()
        .skip(5)
        .take(5)
    {
        let key = format!("CASES/{}/scenes/{}.mp4", &cfd_case, MP4_FILENAME);
        println!("{key}");

        let stream = client
            .get_object()
            .bucket(S3_BUCKET)
            .key(key)
            .send()
            .await?;
        let data = stream.body.collect().await?;
        let buf = Cursor::new(data.into_bytes());

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
                playlist_id: Some(PLAYLIST_ID.to_string()),
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
    }
    Ok(())
}
