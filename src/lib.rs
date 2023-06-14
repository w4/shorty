use anyhow::Context;
use async_trait::async_trait;
use hyper::client::HttpConnector;
use hyper_tls::{
    native_tls::{Identity, TlsConnector},
    HttpsConnector,
};
use rusoto_core::{
    credential::{AwsCredentials, CredentialsError, ProvideAwsCredentials},
    HttpClient, Region,
};
use rusoto_s3::S3Client;
use serde::Deserialize;
use std::path::PathBuf;

pub const CONFIG_PATH: &str = ".config/shorty.toml";

#[derive(Deserialize)]
pub struct Config {
    pub s3: S3Config,
}

impl Config {
    pub async fn load() -> anyhow::Result<Self> {
        let home = std::env::var("HOME").context("$HOME missing")?;
        let path = std::path::PathBuf::from(home).join(CONFIG_PATH);
        let config = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("couldn't load config from {}", CONFIG_PATH))?;
        Ok(toml::from_str(&config)?)
    }
}

#[derive(Deserialize)]
pub struct S3Config {
    pub endpoint: String,
    pub tls: Option<S3TlsConfig>,
    pub bucket: String,
    pub key: String,
    pub secret: String,
}

/// Configuration used for mutual TLS.
#[derive(Deserialize)]
pub struct S3TlsConfig {
    pub certificate: PathBuf,
    pub key: PathBuf,
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

pub async fn s3_client(
    key: String,
    secret: String,
    endpoint: String,
    tls: Option<S3TlsConfig>,
) -> anyhow::Result<S3Client> {
    let mut tls_connector_builder = TlsConnector::builder();

    if let Some(tls) = tls {
        let (key, cert) = tokio::try_join!(
            tokio::fs::read_to_string(&tls.key),
            tokio::fs::read_to_string(&tls.certificate),
        )?;

        // `Identity::from_pkcs8` requires a very specific header
        let key = key.replace("BEGIN RSA PRIVATE KEY", "BEGIN PRIVATE KEY");

        tls_connector_builder.identity(Identity::from_pkcs8(cert.as_bytes(), key.as_bytes())?);
    }

    let mut http_connector = HttpConnector::new();
    http_connector.enforce_http(false);

    let connector = HttpsConnector::from((http_connector, tls_connector_builder.build()?.into()));

    Ok(S3Client::new_with(
        HttpClient::from_connector(connector),
        Credentials { key, secret },
        Region::Custom {
            name: "us-east-1".to_string(),
            endpoint,
        },
    ))
}
