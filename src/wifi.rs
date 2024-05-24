use esp_idf_svc::wifi::{self, AccessPointConfiguration, AuthMethod, BlockingWifi, EspWifi};
use log::info;

// Wi-Fi channel, between 1 and 11
const CHANNEL: u8 = 11;

/// Configure wifi as access point and start it.
pub fn wifi_access_point(
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

/// Configure wifi as client and connect.
pub fn wifi_connect(
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
