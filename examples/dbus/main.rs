#[macro_use]
extern crate log;

use futures_util::StreamExt;
use macaddr::MacAddr;
use std::{
    collections::HashMap,
    net::{Ipv6Addr, SocketAddrV6},
    time::Duration,
};
use tokio::{
    self,
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use zbus::zvariant::Value;

const PORT: u16 = 9000;

use ngn::phy::P2PSession;
use ngn::phy::dbus::{self, wpa_supplicant};
use ngn::utils::retry_timeout;

async fn say_hi_to(addr: &SocketAddrV6) -> std::io::Result<()> {
    info!("We're a client, connecting to {addr:?}");
    let mut stream =
        match tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(addr)).await? {
            Ok(stream) => stream,
            Err(e) => return Err(io::Error::new(io::ErrorKind::TimedOut, e)),
        };

    for i in 0..3 {
        info!("Sending message {i} over the wire!");
        stream.write_all(b"hi there!").await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    let interface_name = std::env::args().nth(1);
    let session = dbus::Session::new(dbus::SessionInit {
        interface_name: interface_name.as_deref(),
        device_name: "RustTest",
        go_intent: 14,
    }).await?;

    let conn = session.system_bus();
    let p2pdevice = session.p2pdevice();

    info!("Starting find operation");
    session.discover_peers().await?;

    let mut find_stopped = p2pdevice.receive_find_stopped().await?;

    let mut device_found = p2pdevice.receive_device_found().await?;
    let mut device_lost = p2pdevice.receive_device_lost().await?;

    let mut invitation_received = p2pdevice.receive_invitation_received().await?;
    let mut invitation_result = p2pdevice.receive_invitation_result().await?;

    let mut wps_failed = p2pdevice.receive_wps_failed().await?;

    let mut group_started = p2pdevice.receive_group_started().await?;
    let mut group_finished = p2pdevice.receive_group_finished().await?;
    let mut group_formation_failure = p2pdevice.receive_group_formation_failure().await?;

    let mut go_negotiation_request = p2pdevice.receive_gonegotiation_request().await?;
    let mut go_negotiation_success = p2pdevice.receive_gonegotiation_success().await?;
    let mut go_negotiation_failure = p2pdevice.receive_gonegotiation_failure().await?;

    let mut persistent_group_added = p2pdevice.receive_persistent_group_added().await?;
    let mut persistent_group_removed = p2pdevice.receive_persistent_group_removed().await?;

    let mut pd_failure = p2pdevice.receive_provision_discovery_failure().await?;
    let mut pd_req_display_pin = p2pdevice
        .receive_provision_discovery_request_display_pin()
        .await?;
    let mut pd_rsp_display_pin = p2pdevice
        .receive_provision_discovery_response_display_pin()
        .await?;
    let mut pd_req_enter_pin = p2pdevice
        .receive_provision_discovery_request_enter_pin()
        .await?;
    let mut pd_rsp_enter_pin = p2pdevice
        .receive_provision_discovery_response_enter_pin()
        .await?;
    let mut pd_pbc_req = p2pdevice.receive_provision_discovery_pbcrequest().await?;
    let mut pd_pbc_rsp = p2pdevice.receive_provision_discovery_pbcresponse().await?;

    // Properties
    // TODO? This seems silly (as noted in the docs, there can be concurrent groups so watching
    // the group property doesn't make much sense).
    // let mut group_changed = p2pdevice.receive_group_changed().await;
    let mut peers_changed = p2pdevice.receive_peers_changed().await;
    let mut persistent_groups_changed = p2pdevice.receive_persistent_groups_changed().await;

    futures_util::try_join!(
        async {
            while let Some(msg) = wps_failed.next().await {
                info!("WPS failed");
                let args = msg.args()?;
                let args = args.args();
                error!("WPS failed: {args:?}");
            }
            Ok::<_, zbus::Error>(())
        },
        async {
            while let Some(_msg) = find_stopped.next().await {
                info!("Find stopped");
                // TODO: Maybe restart?
                // p2pdevice.find(HashMap::default()).await?;
            }
            Ok(())
        },
        async {
            while let Some(msg) = device_found.next().await {
                let args = msg.args()?;
                let peer_path = args.path().to_owned();
                info!("Found device at {peer_path}");

                {
                    let peer = wpa_supplicant::peer::PeerProxy::new(&conn, &peer_path).await?;
                    let dev_name = peer.device_name().await?;
                    let dev_addr = peer.device_address().await?;
                    info!("Peer name: {dev_name:?}, peer dev addr: {dev_addr:?}");

                    if !dev_name.starts_with("Rust") {
                        continue;
                    }
                }

                let mut args = HashMap::default();
                let peer = Value::from(peer_path);
                let method = Value::from("pbc");
                let go_intent = Value::from(14);
                args.insert("peer", &peer);
                args.insert("wps_method", &method);
                args.insert("go_intent", &go_intent);
                match p2pdevice.connect(args).await {
                    Ok(pin) => info!("Connected with pin: {pin}"),
                    Err(e) => error!("Failed to connect to peer: {e:?}"),
                }
            }
            Ok(())
        },
        async {
            while let Some(msg) = device_lost.next().await {
                let args = msg.args()?;
                let peer_path = args.path();
                info!("Lost device at {peer_path}");
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
            while let Some(msg) = invitation_result.next().await {
                let args = msg.args()?;
                let props = args.invite_result();
                info!("Got invitation result: {props:?}");
            }
            Ok(())
        },
        async {
            while let Some(msg) = group_started.next().await {
                let args = msg.args()?;
                let props = args.properties();
                info!("Group started: {props:?}");
                let interface_path = match props.get("interface_object") {
                    Some(&Value::ObjectPath(ref o)) => o.to_owned(),
                    other => {
                        error!("Expected an interface object path, got {other:?}");
                        continue;
                    }
                };
                info!("Current interface path is {interface_path:?}");
                let iface =
                    wpa_supplicant::interface::InterfaceProxy::new(&conn, interface_path).await?;
                info!("Successfully created interface proxy");
                let iface_name = iface.ifname().await?;
                info!("Interface name is {iface_name:?}");
                let iface_state = iface.state().await?;
                info!("Interface state is {iface_state:?}");

                let scope_id = unsafe {
                    libc::if_nametoindex(std::ffi::CString::new(iface_name).unwrap().as_ptr())
                };
                info!("Interface scope id is {scope_id:?}");

                let group_path = match props.get("group_object") {
                    Some(Value::ObjectPath(ref o)) => o.to_owned(),
                    other => {
                        error!("Expected a group object path, got {other:?}");
                        continue;
                    }
                };

                let group = wpa_supplicant::group::GroupProxy::new(&conn, group_path).await?;
                let group_bssid = group.bssid().await?;
                let Some(go_iface_addr) = to_mac_addr(&group_bssid) else {
                    error!("Expected a valid mac address, got {group_bssid:?}");
                    continue;
                };
                info!("Group BSSID (GO interface address) is {go_iface_addr:?}");

                let is_go = props.get("role") == Some(&Value::from("GO"));
                let local_addr = SocketAddrV6::new(
                    Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0),
                    PORT,
                    /* flowinfo = */ 0,
                    scope_id,
                );
                let go_local_link_addr = mac_addr_to_local_link_address(&go_iface_addr);
                let go_socket_addr =
                    SocketAddrV6::new(go_local_link_addr, PORT, /* flowinfo = */ 0, scope_id);
                info!("GO socket addr is {go_socket_addr:?}, local addr is {local_addr:?}");
                if is_go {
                    info!("Creating GO receiver");
                    // Shamelessly taken from
                    // https://github.com/tokio-rs/tokio/blob/master/examples/echo.rs
                    let listener = TcpListener::bind(local_addr).await?;
                    loop {
                        let (mut socket, _) = listener.accept().await?;
                        tokio::spawn(async move {
                            let mut buf = [0u8; 1024];
                            loop {
                                let n = match socket.read(&mut buf).await {
                                    Ok(n) => n,
                                    Err(e) => {
                                        error!("Failed to read data from socket: {e}");
                                        break;
                                    }
                                };

                                if n == 0 {
                                    return;
                                }

                                info!(
                                    "Got message from socket: {:?}",
                                    String::from_utf8_lossy(&buf[..n])
                                );
                            }
                        });
                    }
                } else {
                    retry_timeout(Duration::from_secs(2), 5, || say_hi_to(&go_socket_addr)).await?;
                }
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
            while let Some(msg) = group_formation_failure.next().await {
                let args = msg.args()?;
                let props = args.reason();
                info!("Group formation failure: {props:?}");
            }
            Ok(())
        },
        async {
            while let Some(msg) = go_negotiation_failure.next().await {
                let args = msg.args()?;
                let props = args.properties();
                info!("GO negotiation failed: {props:?}");
            }
            Ok(())
        },
        async {
            while let Some(msg) = go_negotiation_request.next().await {
                let args = msg.args()?;
                let path = args.path();
                let passwd_id = args.dev_passwd_id();
                let go_intent = args.device_go_intent();
                info!("GO negotiation request from {path} ({passwd_id} / {go_intent})");
            }
            Ok(())
        },
        async {
            while let Some(msg) = go_negotiation_success.next().await {
                let args = msg.args()?;
                let props = args.properties();
                info!("GO negotiation succeeded: {props:?}");
                let is_go = props.get("role_go") == Some(&Value::from("GO"));
                if is_go {
                    info!("We're the group owner!");
                }
                let peer_addr = props.get("peer_interface_addr");
                let Some(interface_addr) = peer_addr.and_then(dbus_mac_addr) else {
                    error!("Expected a valid mac address, got {peer_addr:?}");
                    continue;
                };
                info!("Peer iface address: {interface_addr:}");
                let local_link = mac_addr_to_local_link_address(&interface_addr);
                info!("Peer iface local link address: {local_link:?}");
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
