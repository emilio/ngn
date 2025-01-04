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

    let cur_config = p2pdevice.p2pdevice_config().await?;
    info!("Initial device config: {cur_config:?}");

    p2pdevice
        .set_p2pdevice_config({
            let mut config = HashMap::new();
            config.insert("DeviceName", "RustTest".into());
            config.insert("GOIntent", 15u32.into());
            config
        })
        .await?;

    // Start the find operation.
    info!("Starting find operation");
    p2pdevice.find(HashMap::default()).await?;

    let mut device_found = p2pdevice.receive_device_found().await?;
    let mut device_lost = p2pdevice.receive_device_lost().await?;

    let mut invitation_received = p2pdevice.receive_invitation_received().await?;

    let mut group_started = p2pdevice.receive_group_started().await?;
    let mut group_finished = p2pdevice.receive_group_finished().await?;

    let mut persistent_group_added = p2pdevice.receive_persistent_group_added().await?;
    let mut persistent_group_removed = p2pdevice.receive_persistent_group_removed().await?;

    let mut pd_failure = p2pdevice.receive_provision_discovery_failure().await?;
    let mut pd_req_display_pin = p2pdevice.receive_provision_discovery_request_display_pin().await?;
    let mut pd_rsp_display_pin = p2pdevice.receive_provision_discovery_response_display_pin().await?;
    let mut pd_req_enter_pin = p2pdevice.receive_provision_discovery_request_enter_pin().await?;
    let mut pd_rsp_enter_pin = p2pdevice.receive_provision_discovery_response_enter_pin().await?;
    let mut pd_pbc_req = p2pdevice.receive_provision_discovery_pbcrequest().await?;
    let mut pd_pbc_rsp = p2pdevice.receive_provision_discovery_pbcresponse().await?;

    // TODO?
    // ProvisionDiscoveryPBCRequest(o: peer_object)
    // ProvisionDiscoveryPBCResponse(o: peer_object)
    // ProvisionDiscoveryFailure
    //
    // Properties
    // TODO? This seems silly (as noted in the docs, there can be concurrent groups so watching
    // the group property doesn't make much sense).
    // let mut group_changed = p2pdevice.receive_group_changed().await;
    let mut peers_changed = p2pdevice.receive_peers_changed().await;
    let mut persistent_groups_changed = p2pdevice.receive_persistent_groups_changed().await;

    futures_util::try_join!(
        async {
            while let Some(msg) = device_found.next().await {
                let args = msg.args()?;
                let device_path = args.path();
                info!("Found device at {device_path}, try to connect?");
                let mut args = HashMap::default();
                let peer = Value::from(device_path);
                let method = Value::from("display");
                args.insert("peer", &peer);
                args.insert("wps_method", &method);
                let pin = p2pdevice.connect(args).await?;
                info!("Connected with pin: {pin}");
            }
            Ok::<_, zbus::Error>(())
        },
        async {
            while let Some(msg) = device_lost.next().await {
                let args = msg.args()?;
                let device_path = args.path();
                info!("Lost device at {device_path}");
            }
            Ok(())
        },
        async {
            while let Some(msg) = invitation_received.next().await {
                let args = msg.args()?;
                let props = args.properties();
                info!("Got invitation: {props:?}");
            }
            Ok(())
        },
        async {
            while let Some(msg) = group_started.next().await {
                let args = msg.args()?;
                let props = args.properties();
                info!("Group started: {props:?}");
            }
            Ok(())
        },
        async {
            while let Some(msg) = group_finished.next().await {
                let args = msg.args()?;
                let props = args.properties();
                info!("Group finished: {props:?}");
            }
            Ok(())
        },
        async {
            while let Some(msg) = peers_changed.next().await {
                let value = msg.get().await?;
                info!("Peers changed: {value:?}");
            }
            Ok(())
        },
        async {
            while let Some(msg) = persistent_group_added.next().await {
                let args = msg.args()?;
                let path = args.path();
                let props = args.properties();
                info!("Persistent Group added ({path}): {props:?}");
            }
            Ok(())
        },
        async {
            while let Some(msg) = persistent_group_removed.next().await {
                let args = msg.args()?;
                let path = args.path();
                info!("Persistent Group removed: {path:?}");
            }
            Ok(())
        },
        async {
            while let Some(msg) = persistent_groups_changed.next().await {
                let value = msg.get().await?;
                info!("Persistent Groups changed: {value:?}");
            }
            Ok(())
        },
        async {
            while let Some(msg) = pd_failure.next().await {
                let args = msg.args()?;
                let peer_object = args.peer_object();
                let status = args.status();
                info!("Provision Discovery failure: {peer_object} ({status})");
            }
            Ok(())
        },
        async {
            while let Some(msg) = pd_req_display_pin.next().await {
                let args = msg.args()?;
                let peer_object = args.peer_object();
                let pin = args.pin();
                info!("PD Request display pin: {peer_object} ({pin})");
            }
            Ok(())
        },
        async {
            while let Some(msg) = pd_rsp_display_pin.next().await {
                let args = msg.args()?;
                let peer_object = args.peer_object();
                let pin = args.pin();
                info!("PD Response display pin: {peer_object} ({pin})");
            }
            Ok(())
        },
        async {
            while let Some(msg) = pd_req_enter_pin.next().await {
                let args = msg.args()?;
                let peer_object = args.peer_object();
                info!("PD Request enter pin: {peer_object}");
            }
            Ok(())
        },
        async {
            while let Some(msg) = pd_rsp_enter_pin.next().await {
                let args = msg.args()?;
                let peer_object = args.peer_object();
                info!("PD Response enter pin: {peer_object}");
            }
            Ok(())
        },
        async {
            while let Some(msg) = pd_pbc_req.next().await {
                let args = msg.args()?;
                let peer_object = args.peer_object();
                info!("PD PBC Request: {peer_object}");
            }
            Ok(())
        },
        async {
            while let Some(msg) = pd_pbc_rsp.next().await {
                let args = msg.args()?;
                let peer_object = args.peer_object();
                info!("PD PBC Response: {peer_object}");
            }
            Ok(())
        },
    )?;

    Ok(())
}
