use anyhow::anyhow;
use embedded_svc::{
    http::{client::Client, Method},
    utils::io,
};
use esp_idf_svc::http::client::EspHttpConnection;
use log::info;

pub fn get<'a>(
    client: &mut Client<EspHttpConnection>,
    url: String,
    headers: &'a [(&'a str, &'a str)],
) -> anyhow::Result<impl AsRef<str>> {
    let res = Err(anyhow!("error should not be reached"));
    for _i in 1..3 {
        let res = get_internal(client, url.clone(), headers);

        if res.is_ok() {
            return res;
        }
    }
    res
}

fn get_internal<'a>(
    client: &mut Client<EspHttpConnection>,
    url: impl AsRef<str>,
    headers: &'a [(&'a str, &'a str)],
) -> anyhow::Result<impl AsRef<str>> {
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
        Err(anyhow!(
            "could not reach endpoint. status: {}. message: {}",
            status,
            res_body
        ))
    }
}
