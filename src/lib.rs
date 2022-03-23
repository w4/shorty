use anyhow::Context;
use async_trait::async_trait;
use rusoto_core::{
    credential::{AwsCredentials, CredentialsError, ProvideAwsCredentials},
    HttpClient, Region,
};
use rusoto_s3::S3Client;
use serde::Deserialize;

pub const CONFIG_PATH: &'static str = ".config/shorty.toml";

#[derive(Deserialize)]
pub struct Config {
    pub s3: S3Config,
}

impl Config {
    pub async fn load() -> anyhow::Result<Self> {
        let home = std::env::var("HOME").context("$HOME missing")?;
        let path = std::path::PathBuf::from(home).join(CONFIG_PATH);
        let config = tokio::fs::read(path)
            .await
            .with_context(|| format!("couldn't load config from {}", CONFIG_PATH))?;
        Ok(toml::from_slice(&config)?)
    }
}

#[derive(Deserialize)]
pub struct S3Config {
    pub endpoint: String,
    pub bucket: String,
    pub key: String,
    pub secret: String,
}

pub struct Credentials {
    key: String,
    secret: String,
}

#[async_trait]
impl ProvideAwsCredentials for Credentials {
    async fn credentials(&self) -> Result<AwsCredentials, CredentialsError> {
        Ok(AwsCredentials::new(&self.key, &self.secret, None, None))
    }
}

pub fn s3_client(key: String, secret: String, endpoint: String) -> anyhow::Result<S3Client> {
    Ok(S3Client::new_with(
        HttpClient::new()?,
        Credentials { key, secret },
        Region::Custom {
            name: "us-east-1".to_string(),
            endpoint,
        },
    ))
}
