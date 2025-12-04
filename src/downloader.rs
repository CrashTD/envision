use crate::{
    constants::APP_ID, paths::get_cache_dir, util::file_utils::get_writer, util::hash::sha256,
};
use reqwest::{
    Method,
    header::{HeaderMap, USER_AGENT},
};
use std::{io::prelude::*, path::Path};
use std::{path::PathBuf, time::Duration};

const TIMEOUT: Duration = Duration::from_secs(60);
const CHUNK_SIZE: usize = 1024;

fn headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, format!("{APP_ID}/1.0").parse().unwrap());
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

/// get the cache path for a downloaded file
pub fn cache_file_path(url: &str, extension: Option<&str>) -> PathBuf {
    let hash = sha256(url);
    get_cache_dir().join(format!(
        "{hash}{}",
        if let Some(ext) = extension {
            format!(".{ext}")
        } else {
            "".into()
        }
    ))
}

/// Creates a unique hash for the url, checks if already present. If present
/// will skip download and give you the dest file path, if absent it will
/// download it and return the same destination path
///
/// # Arguments
///
/// * `url` - the url to the file that will be downloaded
/// * `extension` - optional file extension that will be appended to the name, without the dot
pub async fn cache_file(url: &str, extension: Option<&str>) -> Result<PathBuf, reqwest::Error> {
    let dest = cache_file_path(url, extension);
    if !dest.is_file() {
        download_file_async(url, &dest).await?;
    }
    Ok(dest)
}
