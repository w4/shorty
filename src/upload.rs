use anyhow::anyhow;
use rusoto_core::ByteStream;
use rusoto_s3::{PutObjectRequest, S3};
use uuid::Uuid;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args();
    let path = args.nth(1).ok_or_else(|| anyhow!("no file given"))?;
    let path = std::path::PathBuf::from(path);

    let (config, file) = tokio::try_join!(shorty::Config::load(), async {
        Ok(tokio::fs::read(&path).await?)
    },)?;

    let s3 = shorty::s3_client(
        config.s3.key,
        config.s3.secret,
        config.s3.endpoint,
        config.s3.tls,
    )
    .await?;

    let name = if let Some(ext) = path.extension().and_then(|v| v.to_str()) {
        format!("{}.{}", Uuid::new_v4(), ext)
    } else {
        Uuid::new_v4().to_string()
    };

    println!("https://{}/u/{}", config.s3.bucket, name);

    s3.put_object(PutObjectRequest {
        key: format!("u/{}", name),
        content_type: mime_guess::from_path(&path).first().map(|v| v.to_string()),
        bucket: config.s3.bucket,
        body: Some(ByteStream::from(file)),
        ..PutObjectRequest::default()
    })
    .await?;

    Ok(())
}
