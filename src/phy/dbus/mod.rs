//! DBus implementation of the physical layer, interfacing with wpa_supplicant.
//!
//! Known limitation: Only current method of provisioning the GO IP address is via IPV6 link-local
//! addresses. This means that if you use something like private address assignment via something
//! like `slaac private`, it will fail to connect to the GO. Other neighbor discovery approaches
//! could be used in the future.

pub mod wpa_supplicant;

use std::collections::HashMap;

use log::trace;
use wpa_supplicant::{p2pdevice::P2PDeviceProxy, wpa_supplicant::WpaSupplicantProxy};
use zbus::zvariant::Value;
use super::PeerId;

#[derive(Debug)]
struct Peer {
    proxy: wpa_supplicant::peer::PeerProxy<'static>,
    name: String,
    address: Vec<u8>,
}

/// Global state for a P2P session.
#[derive(Debug)]
pub struct Session {
    system_bus: zbus::Connection,
    wpa_supplicant: WpaSupplicantProxy<'static>,
    p2pdevice: P2PDeviceProxy<'static>,
    peers: handy::HandleMap<Peer>,
}

pub struct SessionInit<'a> {
    /// The interface name to use, or `None` to use the first available interface.
    pub interface_name: Option<&'a str>,
    /// The device name we're advertised as.
    pub device_name: &'a str,
    /// Our group owner intent, from 0 to 15.
    pub go_intent: u32,
}

#[async_trait::async_trait]
impl super::P2PSession for Session {
    type InitArgs<'a> = SessionInit<'a>;

    /// Create a new P2P session.
    async fn new(init: SessionInit<'_>) -> Result<Self, Box<dyn std::error::Error>> {
        trace!("Trying to connect to system bus");
        let system_bus = zbus::Connection::system().await?;

        trace!("Trying to create wpa_supplicant proxy");
        let wpa_supplicant = wpa_supplicant::wpa_supplicant::WpaSupplicantProxy::new(&system_bus).await?;

        trace!("Scanning for p2p capabilities");

        let caps = wpa_supplicant.capabilities().await?;
        trace!("Got {caps:?}");

        if !caps.iter().any(|s| s == "p2p") {
            return Err(crate::utils::trivial_error!("wpa_supplicant has no p2p support"));
        }

        // TODO(emilio): Maybe remove this once stuff works more reliably, or do this based on the
        // current log level.
        trace!("Setting debug log level");
        wpa_supplicant.set_debug_level("debug").await?;

        let p2pdevice = match init.interface_name {
            Some(iface) => {
                trace!("Requested explicit interface {iface:?}");
                let iface_path = match wpa_supplicant.get_interface(iface).await {
                    Ok(path) => path,
                    Err(e) => {
                        trace!("Couldn't get interface for {iface}, creating: {e}");
                        let name = Value::new(iface);
                        let mut args = HashMap::new();
                        args.insert("Ifname", &name);
                        wpa_supplicant.create_interface(args).await?
                    }
                };

                trace!("Got path {iface_path:?}");
                wpa_supplicant::p2pdevice::P2PDeviceProxy::new(&system_bus, iface_path.clone()).await?
            }
            None => {
                trace!("Looking for interfaces with p2p support");

                let ifaces = wpa_supplicant.interfaces().await?;
                trace!("Got interfaces: {ifaces:?}");

                let mut result = None;
                for iface_path in ifaces {
                    trace!("trying {iface_path}");
                    match wpa_supplicant::p2pdevice::P2PDeviceProxy::new(&system_bus, iface_path.clone()).await {
                        Ok(p) => {
                            trace!("Successfully created a proxy for {iface_path:?}");
                            result = Some(p);
                            break;
                        }
                        Err(e) => trace!("Creating P2P proxy for {iface_path} failed: {e}"),
                    }
                }
                match result {
                    Some(r) => r,
                    None => return Err(crate::utils::trivial_error!(
                        "Couldn't create P2P proxy for any interface"
                    ))
                }
            }
        };

        let cur_config = p2pdevice.p2pdevice_config().await?;
        trace!("Initial device config: {cur_config:?}");

        // TODO: Make configurable.
        p2pdevice.set_p2pdevice_config({
            let mut config = HashMap::new();
            config.insert("DeviceName", init.device_name.into());
            config.insert("GOIntent", init.go_intent.into());
            config
        }).await?;

        trace!("Successfully initialized P2P session");
        Ok(Self {
            system_bus,
            wpa_supplicant,
            p2pdevice,
            peers: handy::HandleMap::default(),
        })
    }

    async fn discover_peers(&self) -> Result<(), Box<dyn std::error::Error>> {
        trace!("Session::discover_peers");
        self.p2pdevice.find(HashMap::default()).await?;
        Ok(())
    }

    fn peer_name(&self, id: PeerId) -> Option<&str> {
        Some(&self.peers.get(id.0)?.name)
    }
}

impl Session {
    pub fn system_bus(&self) -> &zbus::Connection {
        &self.system_bus
    }

    pub fn wpa_s(&self) -> &WpaSupplicantProxy<'static> {
        &self.wpa_supplicant
    }

    pub fn p2pdevice(&self) -> &P2PDeviceProxy<'static> {
        &self.p2pdevice
    }
}
