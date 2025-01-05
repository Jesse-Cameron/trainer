use anyhow::{bail, Result};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::peripheral,
    nvs::EspDefaultNvsPartition,
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use log::{info, warn};

const RETRY_DELAY_MS: u64 = 2000;

pub fn wifi(
    ssid: &str,
    pass: &str,
    modem: impl peripheral::Peripheral<P = esp_idf_svc::hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
) -> Result<Box<EspWifi<'static>>> {
    let mut auth_method = AuthMethod::WPA2Personal;
    if ssid.is_empty() {
        bail!("Missing WiFi name")
    }
    if pass.is_empty() {
        auth_method = AuthMethod::None;
        info!("Wifi password is empty");
    }

    let nvs_partition = EspDefaultNvsPartition::take()?;
    let mut esp_wifi = EspWifi::new(modem, sysloop.clone(), Some(nvs_partition))?;

    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sysloop)?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: ssid
            .try_into()
            .expect("Could not parse the given SSID into WiFi config"),
        bssid: None,
        password: pass
            .try_into()
            .expect("Could not parse the given password into WiFi config"),
        channel: None,
        auth_method,
        ..Default::default()
    }))?;

    wifi.start()?;
    info!("Wifi started");

    info!("Connecting wifi...");

    let mut connect_counter = 0;

    while connect_counter < 5 {
        match wifi.connect() {
            Ok(()) => break,
            Err(err) => {
                warn!(
                    "Wifi connect failed, reason {}, retrying in {}s",
                    err,
                    RETRY_DELAY_MS / 1000
                );
                sleep_ms(RETRY_DELAY_MS);
                wifi.stop()?;
                wifi.start()?;
            }
        }
        connect_counter += 1;
    }

    info!("Waiting for DHCP lease...");

    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

    info!("Wifi DHCP info: {:?}", ip_info);

    Ok(Box::new(esp_wifi))
}

fn sleep_ms(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms))
}
