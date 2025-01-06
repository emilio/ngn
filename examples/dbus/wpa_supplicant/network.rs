//! # D-Bus interface proxy for: `fi.w1.wpa_supplicant1.Interface.Network`
//!
//! Hand-coded because zbus-xmlgen couldn't pick this one up, and it's trivial anyways.
use zbus::proxy;
#[proxy(
    interface = "fi.w1.wpa_supplicant1.Interface.Mesh",
    default_service = "fi.w1.wpa_supplicant1",
)]
pub trait Network {
    /// Enabled property
    #[zbus(property)]
    fn enabled(&self) -> zbus::Result<bool>;

    /// Properties property
    #[zbus(property)]
    fn properties(&self) -> zbus::Result<std::collections::HashMap<String, zbus::zvariant::OwnedValue>>;
}
