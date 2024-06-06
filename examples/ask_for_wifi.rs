use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::peripherals::Peripherals,
    nvs::{EspDefaultNvsPartition, EspNvs},
    wifi::{BlockingWifi, EspWifi},
};
use log::{error, info};

use esp_idf_experiments::{
    ask_for_wifi::{ask_for_wifi, WifiCreds},
    http::{https_client, ntfy_send},
    wifi::wifi_connect,
};

const AP_SSID: &str = "esp32c3";
const AP_PASSWORD: &str = "12345678";

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs_partition = EspDefaultNvsPartition::take()?;

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(
            peripherals.modem,
            sys_loop.to_owned(),
            Some(nvs_partition.to_owned()),
        )?,
        sys_loop,
    )?;

    let namespace = "ask_for_wifi";
    let key = "wifi_creds";

    let mut nvs = {
        match EspNvs::new(nvs_partition, namespace, true) {
            Ok(nvs) => {
                info!("Got namespace {:?} from default partition", namespace);
                nvs
            }
            Err(e) => panic!("Could't get namespace {:?}", e),
        }
    };

    let buffer: &mut [u8] = &mut [0; 256];

    let wifi_creds = {
        let stored_wifi_creds = nvs
            .get_raw(key, buffer)
            .unwrap_or_default()
            .and_then(|b| postcard::from_bytes::<WifiCreds>(b).ok());

        match stored_wifi_creds {
            Some(wifi_creds) => {
                info!("Loaded wifi creds from nvs");
                wifi_creds
            }
            None => {
                info!("Couldn't load wifi creds from nvs, starting AP to ask...");
                let wc = ask_for_wifi(&mut wifi, AP_SSID, AP_PASSWORD)?;

                match nvs.set_raw(key, &postcard::to_vec::<_, 256>(&wc).unwrap()) {
                    Ok(_) => info!("Successfully stored wifi creds in NVS"),
                    Err(e) => error!("Failed to store wifi creds in NVS\n{:?}", e),
                };

                wc
            }
        }
    };

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
