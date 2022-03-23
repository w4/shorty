use anyhow::anyhow;
use rusoto_core::ByteStream;
use rusoto_s3::{PutObjectRequest, S3};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args();
    let url = args.nth(1).ok_or_else(|| anyhow!("no url given"))?;

    let config = shorty::Config::load().await?;
    let s3 = shorty::s3_client(config.s3.key, config.s3.secret, config.s3.endpoint)?;

    let redirect = format!(r#"<meta http-equiv="refresh" content="0; URL={}" />"#, url);
    let link = gpw::PasswordGenerator::default().next().unwrap();

    println!("https://{}/s/{}", config.s3.bucket, link);

    s3.put_object(PutObjectRequest {
        key: format!("s/{}", link),
        content_type: Some("text/html".to_string()),
        bucket: config.s3.bucket,
        body: Some(ByteStream::from(redirect.into_bytes())),
        ..PutObjectRequest::default()
    })
    .await?;

    Ok(())
}
