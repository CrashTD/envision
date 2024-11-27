// how to regenerate this one:
//
// ```bash
// cargo install zbus_xmlgen
// curl -sSLO https://github.com/WiVRn/WiVRn/blob/master/dbus/io.github.wivrn.Server.xml
// zbus-xmlgen file io.github.wivrn.Server.xml
// ```
//
// it should output a file called server.rs, move it accordingly
#[rustfmt::skip]
#[allow(non_snake_case)]
mod internal;

/// timeout for dbus methods in seconds
const TIMEOUT: i32 = 10;

async fn proxy<'a>() -> zbus::Result<internal::ServerProxy<'a>> {
    let connection = zbus::Connection::session().await?;
    let proxy = internal::ServerProxy::new(&connection).await?;
    Ok(proxy)
}

pub async fn is_pairing_mode() -> zbus::Result<bool> {
    proxy().await?.pairing_enabled().await
}

pub async fn enable_pairing() -> zbus::Result<String> {
    proxy().await?.enable_pairing(TIMEOUT).await
}

pub async fn disable_pairing() -> zbus::Result<()> {
    proxy().await?.disable_pairing().await
}

pub async fn pairing_pin() -> zbus::Result<String> {
    proxy().await?.pin().await
}
