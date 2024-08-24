use crate::{constants::APP_ID, file_utils::get_writer, hash::sha256, paths::get_cache_dir};
use reqwest::{
    header::{HeaderMap, USER_AGENT},
    Method,
};
use std::{io::prelude::*, path::Path};
use std::{path::PathBuf, time::Duration};

const TIMEOUT: Duration = Duration::from_secs(60);
const CHUNK_SIZE: usize = 1024;

fn headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, format!("{}/1.0", APP_ID).parse().unwrap());
    headers
}

fn sync_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(TIMEOUT)
        .default_headers(headers())
        .build()
        .expect("Failed to build reqwest::blocking::Client")
}

fn async_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(TIMEOUT)
        .default_headers(headers())
        .build()
        .expect("Failed to build request::Client")
}

pub fn download_file_sync(url: &str, path: &Path) -> Result<(), reqwest::Error> {
    let client = sync_client();
    let res = client.request(Method::GET, url).send()?;
    let status = res.status();
    if status.is_client_error() || status.is_server_error() {
        return Err(res.error_for_status().unwrap_err());
    }
    let mut writer = get_writer(path).expect("Unable to write to path");
    for chunk in res
        .bytes()
        .expect("Could not get HTTP response bytes")
        .chunks(CHUNK_SIZE)
    {
        writer.write_all(chunk).expect("Failed to write chunk");
    }
    writer.flush().expect("Failed to flush download writer");
    Ok(())
}

pub async fn download_file_async(url: &str, path: &Path) -> Result<(), reqwest::Error> {
    let client = async_client();
    let res = client.request(Method::GET, url).send().await?;
    let status = res.status();
    if status.is_client_error() || status.is_server_error() {
        return Err(res.error_for_status().unwrap_err());
    }
    let mut writer = get_writer(path).expect("Unable to write to path");
    for chunk in res
        .bytes()
        .await
        .expect("Could not get HTTP response bytes")
        .chunks(CHUNK_SIZE)
    {
        writer.write_all(chunk).expect("Failed to write chunk");
    }
    writer.flush().expect("Failed to flush download writer");
    Ok(())
}

pub async fn cache_file(url: &str) -> Result<PathBuf, reqwest::Error> {
    let hash = sha256(url);
    let dest = get_cache_dir().join(hash);
    if !dest.is_file() {
        download_file_async(url, &dest).await?;
    }
    Ok(dest)
}
