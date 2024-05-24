mod misc;
mod temp;


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


fn ntfy_send(
    client: &mut HttpClient<EspHttpConnection>,
    topic: &str,
    notification: &str,
) -> anyhow::Result<String> {
    let url = format!("https://ntfy.sh/{topic}");
    post(client, url.as_str(), notification.as_bytes())
}

fn post(
    client: &mut HttpClient<EspHttpConnection>,
    url: &str,
    data: &[u8],
) -> anyhow::Result<String> {
    let content_length_header = format!("{}", data.len());
    let headers = [
        ("content-type", "text/plain"),
        ("content-length", &*content_length_header),
    ];

    let mut request = client.post(url, &headers)?;
    request.write_all(data)?;
    request.flush()?;
    let mut response = request.submit()?;

    let mut result = String::new();

    loop {
        let mut buf = [0u8; 1024];
        let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0)?;
        if bytes_read == 0 {
            break;
        }
        match std::str::from_utf8(&buf[0..bytes_read]) {
            Ok(body) => result.push_str(body),
            Err(e) => error!("Error decoding response body: {}", e),
        };
    }

    Ok(result)
}
