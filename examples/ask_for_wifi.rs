
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::peripherals::Peripherals,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, EspWifi},
};
use log::info;

use esp_idf_experiments::{
    ask_for_wifi::ask_for_wifi,
    http::{https_client, ntfy_send},
    wifi::wifi_connect,
};

const AP_SSID: &str = "esp32c3";
const AP_PASSWORD: &str = "12345678";
static AP_INDEX_HTML: &str = include_str!("../res/ask_for_wifi.html");

// Max payload length
const MAX_LEN: usize = 256;

#[derive(serde::Deserialize)]
struct FormData<'a> {
    ssid: &'a str,
    password: &'a str,
}

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

    let wifi_creds = ask_for_wifi(&mut wifi, AP_SSID, AP_PASSWORD)?;

    wifi_connect(&mut wifi, &wifi_creds.ssid, &wifi_creds.password)?;

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
