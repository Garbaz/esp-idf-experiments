
use core::convert::TryInto;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Point,
    mono_font::MonoTextStyleBuilder,
    pixelcolor::BinaryColor,
    text::{Baseline, Text},
    Drawable as _,
};
use embedded_svc::{
    http::client::Client as HttpClient,
    io::Write,
    utils::io,
    wifi::{AuthMethod, ClientConfiguration, Configuration},
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        adc::{attenuation, config::Config as AdcConfig, AdcChannelDriver, AdcDriver},
        i2c::{I2cConfig, I2cDriver},
        peripherals::Peripherals,
        prelude::*,
    },
    http::client::{Configuration as HttpConfiguration, EspHttpConnection},
    log::EspLogger,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, EspWifi},
};
use log::{error, info};
use misc::FontSize;
use ssd1306::{
    mode::DisplayConfig as _, rotation::DisplayRotation, size::DisplaySize128x32,
    I2CDisplayInterface, Ssd1306,
};
use std::{thread, time::Duration};
use temp::{TempCalibration, TempConverter};

#[toml_cfg::toml_config]
pub struct CfgToml {
    #[default("Couldn't find cfg.toml!")]
    wifi_ssid: &'static str,
    #[default("Couldn't find cfg.toml!")]
    wifi_pass: &'static str,
}

const SSID: &str = CFG_TOML.wifi_ssid;
const PASSWORD: &str = CFG_TOML.wifi_pass;

const CONNECT_TO_WIFI: bool = false;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let config = I2cConfig::new().baudrate(100.kHz().into());
    let i2c = {
        let i2c = peripherals.i2c0;
        let sda = peripherals.pins.gpio6;
        let scl = peripherals.pins.gpio7;
        I2cDriver::new(i2c, sda, scl, &config)?
    };

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate180)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    let mut client = if CONNECT_TO_WIFI {
        text(&mut display, FontSize::H6, "Connecting to wifi...");

        let client = {
            let mut wifi = BlockingWifi::wrap(
                EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
                sys_loop,
            )?;

            match connect_wifi(&mut wifi) {
                Ok(()) => {
                    let config = HttpConfiguration {
                        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
                        ..Default::default()
                    };

                    Some(HttpClient::wrap(EspHttpConnection::new(&config)?))
                }
                Err(e) => {
                    println!("{:#?}", e);
                    None
                }
            }
        };

        if client.is_none() {
            text(&mut display, FontSize::H6, "Couldn't connect to wifi!");
        } else {
            text(&mut display, FontSize::H6, "Connected to wifi!");
        }

        thread::sleep(Duration::from_millis(250));

        client
    } else {
        None
    };

    let mut adc = AdcDriver::new(peripherals.adc1, &AdcConfig::new().calibration(true))?;

    let mut adc_pin: AdcChannelDriver<{ attenuation::DB_11 }, _> =
        AdcChannelDriver::new(peripherals.pins.gpio4)?;

    let temp_converter = TempConverter::new(
        9930.,
        TempCalibration {
            adc_value: 1573,
            temperature: 20.8,
        },
        TempCalibration {
            adc_value: 1090,
            temperature: 13.5,
        },
    );

    let mut notified = false;

    loop {
        let adc_value = adc.read(&mut adc_pin)?;
        let temperature = temp_converter.convert(adc_value);
        // println!("{:.1}°C ({})", temperature, adc_value);

        let s = format!("{:.1}°C ({})", temperature, adc_value);

        text(&mut display, FontSize::H10, s.as_str());

        if temperature > 23.0 {
            if !notified {
                if let Some(client) = client.as_mut() {
                    let n = "Temperature is over 23°C!".to_string();
                    let r = ntfy_send(client, "garbaz", n.as_str())?;
                    println!("{}", r);
                    notified = true;
                }
            }
        } else {
            notified = false;
        }

        thread::sleep(Duration::from_millis(250));
    }

    // Ok(())
}

fn text(
    display: &mut Ssd1306<
        ssd1306::prelude::I2CInterface<I2cDriver>,
        DisplaySize128x32,
        ssd1306::mode::BufferedGraphicsMode<DisplaySize128x32>,
    >,
    size: FontSize,
    text: &str,
)
// where
// D: DrawTarget<Color = BinaryColor>,
// D::Error: core::fmt::Debug,
{
    display.clear(BinaryColor::Off).unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(size.to_font())
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline(text, Point::new(1, 16), text_style, Baseline::Middle)
        .draw(display)
        .unwrap();

    display.flush().unwrap();
}

fn connect_wifi(wifi: &mut BlockingWifi<EspWifi<'static>>) -> anyhow::Result<()> {
    let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        bssid: None,
        auth_method: AuthMethod::WPA2Personal,
        password: PASSWORD.try_into().unwrap(),
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
