use std::collections::HashMap;

use tokio;
use zbus::zvariant::Value;

mod wpa_supplicant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = zbus::Connection::system().await?;
    let proxy = wpa_supplicant::wpa_supplicant::WpaSupplicantProxy::new(&conn).await?;
    dbg!(&proxy);

    let caps = proxy.capabilities().await?;
    dbg!(&caps);

    let ifaces = proxy.interfaces().await?;
    dbg!(&ifaces);

    // Try to remove all existing interfaces.
    let _ = futures_util::future::try_join_all(
        ifaces.iter().map(|iface| proxy.remove_interface(iface)),
    )
    .await;

    let iface = {
        let name = Value::new("wlo2");
        let mut args = HashMap::new();
        args.insert("Ifname", &name);
        proxy.create_interface(args).await?
    };

    dbg!(&iface);

    let p2pdevice = wpa_supplicant::p2pdevice::P2PDeviceProxy::new(&conn, iface).await?;
    dbg!(&p2pdevice);

    Ok(())
}
