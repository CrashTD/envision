use crate::file_utils::get_writer;
use reqwest::{
    header::{HeaderMap, USER_AGENT},
    Method,
};
use std::{io::prelude::*, thread::JoinHandle};
use std::{thread, time::Duration};

const TIMEOUT: Duration = Duration::from_secs(60);
const CHUNK_SIZE: usize = 1024;

fn headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "org.gabmus.rex2/1.0".parse().unwrap());
    headers
}

fn client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(TIMEOUT)
        .default_headers(headers())
        .build()
        .expect("Failed to build reqwest::Client")
}

pub fn download_file(url: String, path: String) -> JoinHandle<Result<(), reqwest::Error>> {
    thread::spawn(move || {
        let client = client();
        match client.request(Method::GET, url).send() {
            Ok(res) => {
                let status = res.status();
                if status.is_client_error() || status.is_server_error() {
                    return Err(res.error_for_status().unwrap_err());
                }
                let mut writer = get_writer(&path);
                for chunk in res
                    .bytes()
                    .expect("Could not get HTTP response bytes")
                    .chunks(CHUNK_SIZE)
                {
                    writer.write(chunk).expect("Failed to write chunk");
                }
                writer.flush().expect("Failed to flush download writer");
                Ok(())
            }
            Err(e) => Err(e),
        }
    })
}
