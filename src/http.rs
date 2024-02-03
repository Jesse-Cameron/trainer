use embedded_svc::{
    http::{client::Client, Method},
    utils::io,
};
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use log::info;

pub fn get<'a>(url: String, headers: &'a [(&'a str, &'a str)]) -> anyhow::Result<String> {
    let mut res = Ok("".to_string());
    for _i in 1..3 {
        res = get_internal(url.clone(), headers);

        if res.is_ok() {
            return res;
        }
    }
    res
}

fn get_internal<'a>(
    url: impl AsRef<str>,
    headers: &'a [(&'a str, &'a str)],
) -> anyhow::Result<String> {
    let connection = EspHttpConnection::new(&Configuration {
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        ..Default::default()
    })?;

    let mut client = Client::wrap(connection);
    let request = client.request(Method::Get, url.as_ref(), headers)?;
    let response = request.submit()?;
    let status = response.status();
    info!("status: {}", status);

    let mut reader = response;
    // TODO: we want to work out the rough size of our payload, then set this size
    let mut buf = [0u8; 1024];
    let bytes_read = io::try_read_full(&mut reader, &mut buf).map_err(|err| err.0)?;

    // Drain the remaining response bytes
    while reader.read(&mut buf)? > 0 {}

    let res_body = std::str::from_utf8(&buf[0..bytes_read])?;

    if (200..=299).contains(&status) {
        Ok(res_body.to_string())
    } else {
        Err(anyhow::Error::msg(format!(
            "could not reach endpoint. status: {}. message: {}",
            status, res_body
        )))
    }
}
