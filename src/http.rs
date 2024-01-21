// use anyhow::Error;
// use anyhow::Result;
use embedded_svc::{
    http::{client::Client, Method},
    utils::io,
};
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use log::info;

trait HttpClient {
    fn get() {}
}

pub fn get<'a>(url: impl AsRef<str>, headers: &'a [(&'a str, &'a str)]) -> anyhow::Result<String> {
    // create a https client
    let connection = EspHttpConnection::new(&Configuration {
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        ..Default::default()
    })?;

    let mut client = Client::wrap(connection);
    info!("https client created");

    let request = client.request(Method::Get, url.as_ref(), headers)?;
    info!("request created");

    let response = request.submit()?;
    info!("request submitted");
    // TODO(jcam): should we do some retries here?
    let status = response.status();
    info!("status: {}", status);

    let mut reader = response;
    // TODO: we want to work out the rough size of our payload, then set this size
    let mut buf = [0u8; 1024];
    let bytes_read = io::try_read_full(&mut reader, &mut buf).map_err(|err| err.0)?;

    // Drain the remaining response bytes
    while reader.read(&mut buf)? > 0 {}

    let res_body = std::str::from_utf8(&buf[0..bytes_read]).map(|s| s.to_string())?;

    info!("{}", res_body);

    Ok(res_body)
}
