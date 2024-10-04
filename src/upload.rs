use std::os::unix::fs::MetadataExt;

use bytes::BytesMut;
use futures_util::{future::Either, stream};
use rusoto_core::ByteStream;
use rusoto_s3::{PutObjectRequest, S3};
use tokio::{fs::File, io::AsyncReadExt};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args();
    let path = args
        .nth(1)
        .filter(|v| v != "-")
        .map(std::path::PathBuf::from);

    let load_file_and_meta = async {
        if let Some(path) = &path {
            let extension = path.extension().and_then(|v| v.to_str());
            let mime = mime_guess::from_path(path).first_raw();
            let file = File::open(path).await?;
            let size = file.metadata().await?.size() as usize;
            let stream = Either::Left(ReaderStream::new(file));
            Ok((stream, extension, mime, size))
        } else {
            let mut buf = BytesMut::new();
            while tokio::io::stdin().read_buf(&mut buf).await? != 0 {}
            let buf = buf.freeze();
            let len = buf.len();
            let (extension, mime) = infer::get(&buf)
                .map(|v| (v.extension(), v.mime_type()))
                .unzip();
            let stream = Either::Right(stream::once(async move { Ok(buf) }));
            Ok((stream, extension, mime, len))
        }
    };

    let build_client = async {
        let config = shorty::Config::load().await?;
        let client = shorty::s3_client(
            config.s3.key,
            config.s3.secret,
            config.s3.endpoint,
            config.s3.tls,
        )
        .await?;

        Ok::<_, anyhow::Error>((config.s3.bucket, client))
    };

    let ((bucket, s3), (stream, extension, mime, size)) =
        tokio::try_join!(build_client, load_file_and_meta)?;

    let id = Uuid::new_v4();

    let key = if let Some(ext) = extension {
        format!("u/{id}.{ext}")
    } else {
        format!("u/{id}")
    };

    println!("https://{bucket}/{key}");

    s3.put_object(PutObjectRequest {
        key,
        content_type: mime.map(|v| v.to_string()),
        bucket,
        body: Some(ByteStream::new_with_size(stream, size)),
        ..PutObjectRequest::default()
    })
    .await?;

    Ok(())
}
