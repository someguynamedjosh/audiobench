use reqwest::blocking::{Client, ClientBuilder};
use serde::Deserialize;
use std::sync::mpsc::Sender;

#[derive(Deserialize)]
pub struct UpdateInfo {
    pub version: u16,
    pub changes: Vec<String>,
    pub download_url: String,
}

fn retrieve_info(client: &mut Client, url: &str) -> Option<UpdateInfo> {
    let request = client
        .get(url)
        // Some websites do not like the default user agent.
        .header("User-Agent", "Chrome/86.0 (KHTML, unlike Lizard)")
        .header("Referer", "https://Audiobench");
    let response = match request.send() {
        Ok(v) => v,
        Err(err) => {
            eprintln!(
                "ERROR: Failed to check for updates at {}, cause by:\nERROR: {}",
                url, err
            );
            return None;
        }
    };
    let response_ok = response.status().is_success();
    let response_text = match response.text() {
        Ok(v) => v,
        Err(err) => {
            eprintln!(
                "ERROR: Failed to decode response at {}, cause by:\nERROR: {}",
                url, err,
            );
            return None;
        }
    };
    if !response_ok {
        eprintln!(
            "ERROR: Failed to check for updates at {}, cause by:\nERROR: {}",
            url, response_text,
        );
        return None;
    }
    match serde_json::from_str(&response_text) {
        Ok(info) => Some(info),
        Err(err) => {
            eprintln!(
                "ERROR: Failed to parse response from {}, cause by:\nERROR: {}",
                url, err,
            );
            None
        }
    }
}

pub fn spawn_update_checker(
    urls_to_check: Vec<String>,
    response_channel: Sender<(String, Option<UpdateInfo>)>,
) {
    std::thread::spawn(move || {
        // TODO: rust-native-tls has a bug which rejects domain names with underscores. PITA when
        // I'm hosting an update checker on code_cube.gitlab.io! Relevant bug report:
        // https://github.com/sfackler/rust-native-tls/issues/177
        let mut client = ClientBuilder::new().use_rustls_tls().build().unwrap();
        for url in urls_to_check.into_iter() {
            let info = retrieve_info(&mut client, &url);
            if let Err(err) = response_channel.send((url, info)) {
                eprintln!(
                    "WARNING: Failed to send update check result, caused by:\n{}",
                    err
                );
            }
        }
    });
}
