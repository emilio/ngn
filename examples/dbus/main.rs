use futures_util::StreamExt;
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

    let iface_path = {
        let name = Value::new("wlo2");
        let mut args = HashMap::new();
        args.insert("Ifname", &name);
        proxy.create_interface(args).await?
    };

    dbg!(&iface_path);

    // let iface = wpa_supplicant::interface::InterfaceProxy::new(&conn, &iface_path).await?;
    let p2pdevice = wpa_supplicant::p2pdevice::P2PDeviceProxy::new(&conn, iface_path).await?;
    dbg!(&p2pdevice);

    let cur_config = p2pdevice.p2pdevice_config().await?;
    dbg!(&cur_config);

    p2pdevice
        .set_p2pdevice_config({
            let mut config = HashMap::new();
            config.insert("DeviceName", "RustTest".into());
            config.insert("GOIntent", 15u32.into());
            config
        })
        .await?;

    // Start the find operation.
    p2pdevice.find(HashMap::default()).await?;

    let mut device_found_stream = p2pdevice.receive_device_found().await?;
    while let Some(msg) = device_found_stream.next().await {
        dbg!(&msg);
    }

    Ok(())
}
