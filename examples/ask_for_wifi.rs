use embedded_svc::http::Headers as _;
use esp_idf_experiments::{
    http::{http_server, https_client, ntfy_send},
    wifi::{wifi_access_point, wifi_connect},
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::peripherals::Peripherals,
    http::Method,
    io::{Read as _, Write as _},
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, EspWifi},
};
use log::{error, info};
use std::sync::mpsc;

const AP_SSID: &str = "esp32c3";
const AP_PASSWORD: &str = "12345678";
static AP_INDEX_HTML: &str = include_str!("ask_for_wifi.html");

// Max payload length
const MAX_LEN: usize = 256;

#[derive(serde::Deserialize)]
struct FormData<'a> {
    ssid: &'a str,
    password: &'a str,
}

// struct WifiWrapper {
//     pub wifi: BlockingWifi<EspWifi>,
//     config: wifi::Configuration,
// }

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;

    wifi_access_point(&mut wifi, AP_SSID, AP_PASSWORD)?;

    let mut server = http_server()?;

    server.fn_handler("/", Method::Get, |req| {
        req.into_ok_response()?
            .write_all(AP_INDEX_HTML.as_bytes())
            .map(|_| ())
    })?;

    let (tx, rx) = mpsc::channel();

    server.fn_handler::<anyhow::Error, _>("/post", Method::Post, move |mut req| {
        let len = req.content_len().unwrap_or(0) as usize;

        if len > MAX_LEN {
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

    wifi_connect(&mut wifi, ssid.as_str(), password.as_str())?;

    let mut client = https_client()?;

    info!("{}", ntfy_send(&mut client, "garbaz", "WOW!")?);

    // // Keep server running beyond when main() returns (forever)
    // // Do not call this if you ever want to stop or access it later.
    // // Otherwise you can either add an infinite loop so the main task
    // // never returns, or you can move it to another thread.
    // // https://doc.rust-lang.org/stable/core/mem/fn.forget.html
    // core::mem::forget(server);

    // Main task no longer needed, free up some memory
    Ok(())
}
