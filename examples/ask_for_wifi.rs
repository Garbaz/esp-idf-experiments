use std::{sync::mpsc, time::Duration};

use embedded_svc::http::Headers as _;
use esp_idf_svc::{
    eventloop::{self, EspEventLoop, EspSystemEventLoop},
    hal::peripherals::Peripherals,
    http::{server::EspHttpServer, Method},
    io::{Read as _, Write as _},
    nvs::{EspDefaultNvsPartition, EspNvsPartition, NvsDefault},
    wifi::{self, AccessPointConfiguration, AuthMethod, BlockingWifi, EspWifi},
};
use log::info;

const AP_SSID: &str = "esp32c3";
const AP_PASSWORD: &str = "12345678";
static AP_INDEX_HTML: &str = include_str!("ask_for_wifi.html");

// Max payload length
const MAX_LEN: usize = 256;

// Need lots of stack to parse JSON
const STACK_SIZE: usize = 10240;

// Wi-Fi channel, between 1 and 11
const CHANNEL: u8 = 11;

#[derive(serde::Deserialize, Clone)]
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

    let server_configuration = esp_idf_svc::http::server::Configuration {
        stack_size: STACK_SIZE,
        ..Default::default()
    };

    let mut server = EspHttpServer::new(&server_configuration)?;

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

        if let Ok(form) = serde_json::from_slice::<FormData>(&buf) {
            write!(resp, "Connecting to {}...", form.ssid)?;
            tx.send(form.clone())?;
        } else {
            resp.write_all("JSON error".as_bytes())?;
        }

        Ok(())
    })?;

    // wifi_connect(&mut wifi, form.ssid, form.password);

    let form = rx.recv()?;

    // // Keep server running beyond when main() returns (forever)
    // // Do not call this if you ever want to stop or access it later.
    // // Otherwise you can either add an infinite loop so the main task
    // // never returns, or you can move it to another thread.
    // // https://doc.rust-lang.org/stable/core/mem/fn.forget.html
    // core::mem::forget(server);

    // Main task no longer needed, free up some memory
    Ok(())
}

fn wifi_access_point(
    wifi: &mut BlockingWifi<EspWifi<'static>>,
    ssid: &str,
    password: &str,
) -> anyhow::Result<()> {
    let config = wifi::Configuration::AccessPoint(AccessPointConfiguration {
        ssid: ssid.try_into().unwrap(),
        ssid_hidden: false,
        auth_method: AuthMethod::WPA2Personal,
        password: password.try_into().unwrap(),
        channel: CHANNEL,
        ..Default::default()
    });
    wifi.set_configuration(&config)?;
    wifi.start()?;
    info!("Wifi started");
    wifi.wait_netif_up()?;
    info!("Wifi netif up");

    Ok(())
}

fn wifi_connect(
    wifi: &mut BlockingWifi<EspWifi<'static>>,
    ssid: &str,
    password: &str,
) -> anyhow::Result<()> {
    let wifi_configuration = wifi::Configuration::Client(wifi::ClientConfiguration {
        ssid: ssid.try_into().unwrap(),
        bssid: None,
        auth_method: AuthMethod::WPA2Personal,
        password: password.try_into().unwrap(),
        channel: None,
    });

    wifi.set_configuration(&wifi_configuration)?;

    wifi.start()?;
    info!("Wifi started");

    wifi.connect()?;
    info!("Wifi connected");

    wifi.wait_netif_up()?;
    info!("Wifi netif up");

    Ok(())
}
