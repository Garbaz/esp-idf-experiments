use embedded_svc::{http::client::Client, utils::io};
use esp_idf_svc::{
    http::{
        client::{Configuration, EspHttpConnection},
        server::EspHttpServer,
    },
    io::Write as _,
};
use log::error;

const HTTP_SERVER_STACK_SIZE: usize = 10240;

pub fn http_server() -> anyhow::Result<EspHttpServer<'static>> {
    let server_configuration = esp_idf_svc::http::server::Configuration {
        stack_size: HTTP_SERVER_STACK_SIZE,
        ..Default::default()
    };

    Ok(EspHttpServer::new(&server_configuration)?)
}

pub fn https_client() -> anyhow::Result<Client<EspHttpConnection>> {
    let config = Configuration {
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        ..Default::default()
    };

    Ok(Client::wrap(EspHttpConnection::new(&config)?))
}

pub fn ntfy_send(
    client: &mut Client<EspHttpConnection>,
    topic: &str,
    notification: &str,
) -> anyhow::Result<String> {
    let url = format!("https://ntfy.sh/{topic}");
    post(client, url.as_str(), notification.as_bytes())
}

pub fn post(
    client: &mut Client<EspHttpConnection>,
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
