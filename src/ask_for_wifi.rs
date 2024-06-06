use std::sync::mpsc;

use embedded_svc::http::Headers as _;
use esp_idf_svc::{
    http::Method,
    io::{Read as _, Write as _},
    wifi::{BlockingWifi, EspWifi},
};
use log::error;
use serde::{Deserialize, Serialize};

use crate::{http::http_server, wifi::wifi_access_point};

static AP_INDEX_HTML: &str = include_str!("../res/ask_for_wifi.html");

#[derive(serde::Deserialize)]
struct FormData<'a> {
    ssid: &'a str,
    password: &'a str,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WifiCreds {
    pub ssid: String,
    pub password: String,
}

/// Set the wifi up as an access point and host a simple website for the user to
/// enter their wifi network's SSID and password.
/// 
/// The user should connect to the access point and navigate to
/// http://192.168.71.1.
pub fn ask_for_wifi(
    wifi: &mut BlockingWifi<EspWifi<'static>>,
    ap_ssid: &str,
    ap_password: &str,
) -> anyhow::Result<WifiCreds> {
    wifi_access_point(wifi, ap_ssid, ap_password)?;

    let mut server = http_server()?;

    server.fn_handler("/", Method::Get, |req| {
        req.into_ok_response()?
            .write_all(AP_INDEX_HTML.as_bytes())
            .map(|_| ())
    })?;

    let (tx, rx) = mpsc::channel();

    server.fn_handler::<anyhow::Error, _>("/post", Method::Post, move |mut req| {
        const MAX_REQUEST_LEN: usize = 2048;

        let len = req.content_len().unwrap_or(0) as usize;

        if len > MAX_REQUEST_LEN {
            req.into_status_response(413)?
                .write_all("Request too big".as_bytes())?;
            return Ok(());
        }

        let mut buf = vec![0; len];
        req.read_exact(&mut buf)?;
        let mut resp = req.into_ok_response()?;

        match serde_json::from_slice::<FormData>(&buf) {
            Ok(form) => {
                write!(resp, "Connecting to {}...", form.ssid)?;
                tx.send((form.ssid.to_string(), form.password.to_string()))?;
            }
            Err(e) => {
                error!("{}", e);
                resp.write_all("JSON error".as_bytes())?;
            }
        }

        Ok(())
    })?;

    let (ssid, password) = rx.recv()?;

    Ok(WifiCreds { ssid, password })
}
