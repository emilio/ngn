#[macro_use]
extern crate log;

use futures_util::StreamExt;
use std::collections::HashMap;
use tokio;
use zbus::zvariant::Value;

mod wpa_supplicant;


macro_rules! trivial_error {
    ($($args:tt)*) => {{
        #[derive(Debug)]
        struct TrivialError;
        impl std::error::Error for TrivialError {}
        impl std::fmt::Display for TrivialError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, $($args)*)
            }
        }
        Box::new(TrivialError) as _
    }}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let conn = zbus::Connection::system().await?;
    let proxy = wpa_supplicant::wpa_supplicant::WpaSupplicantProxy::new(&conn).await?;

    let caps = proxy.capabilities().await?;
    info!("Scanning for p2p capabilities: {caps:?}");

    if !caps.iter().any(|s| s == "p2p") {
        return Err(trivial_error!("wpa_supplicant has no p2p support"));
    }

    let (iface_path, p2pdevice) = match std::env::args().nth(1) {
        Some(iface) => {
            info!("Requested explicit interface {iface:?}");
            let iface_path = match proxy.get_interface(&iface).await {
                Ok(path) => path,
                Err(e) => {
                    info!("Couldn't get interface for {iface}, creating: {e}");
                    let name = Value::new(iface);
                    let mut args = HashMap::new();
                    args.insert("Ifname", &name);
                    proxy.create_interface(args).await?
                }
            };

            info!("Got path {iface_path:?}");
            let proxy = wpa_supplicant::p2pdevice::P2PDeviceProxy::new(&conn, iface_path.clone()).await?;
            (iface_path, proxy)
        },
        None => {
            info!("Looking for interfaces with p2p support");

            let ifaces = proxy.interfaces().await?;
            info!("Got interfaces: {ifaces:?}");

            let mut result = None;
            for iface in ifaces {
                info!("trying {iface}");
                match wpa_supplicant::p2pdevice::P2PDeviceProxy::new(&conn, iface.clone()).await {
                    Ok(p) => {
                        result = Some((iface, p));
                        break;
                    },
                    Err(e) => {
                        info!("Creating P2P proxy for {iface} failed: {e}");
                    },
                }
            }
            match result {
                Some(r) => r,
                None => return Err(trivial_error!("Couldn't create P2P proxy for any interface")),
            }
        },
    };

    info!("Using interface {iface_path:?}");
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
