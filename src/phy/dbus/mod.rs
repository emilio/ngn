//! DBus implementation of the physical layer, interfacing with wpa_supplicant.
//!
//! Known limitation: Only current method of provisioning the GO IP address is via IPV6 link-local
//! addresses. This means that if you use something like private address assignment via something
//! like `slaac private`, it will fail to connect to the GO. Other neighbor discovery approaches
//! could be used in the future.

pub mod wpa_supplicant;

use log::trace;
use wpa_supplicant::wpa_supplicant::WpaSupplicantProxy;

/// Global state for a P2P session.
pub struct Session {
    system_bus: zbus::Connection,
    wpa_supplicant: WpaSupplicantProxy<'static>,
}

impl Session {
    /// Create a new P2P session.
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
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

        trace!("Successfully initialized P2P session");
        Ok(Self {
            system_bus,
            wpa_supplicant,
        })
    }

    pub fn system_bus(&self) -> &zbus::Connection {
        &self.system_bus
    }

    pub fn wpa_s(&self) -> &WpaSupplicantProxy<'static> {
        &self.wpa_supplicant
    }
}
