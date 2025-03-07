//! Main interface for P2P physical groups.

pub mod dbus;

use std::fmt::Debug;

/// A handle for a given peer.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct PeerId(pub(crate) handy::Handle);

#[async_trait::async_trait]
pub trait P2PSessionListener {
    fn peer_discovered(&self, _: &impl P2PSession, _: PeerId) {}
    fn peer_lost(&self, _: &impl P2PSession, _: PeerId) {}
}

#[async_trait::async_trait]
pub trait P2PSession : Sized + Debug {
    /// The backend-specific arguments needed for initialization.
    type InitArgs<'a>: Sized + 'a;

    /// Create a new session.
    async fn new(args: Self::InitArgs<'_>) -> Result<Self, Box<dyn std::error::Error>>;

    /// Explicitly start peer discovery.
    ///
    /// TODO(emilio): For now this doesn't do any filtering. In practice, eventually, it'd be good
    /// to set up pre-association discovery, via something like Bonjour[1][2] or UPnP[3][4].
    ///
    /// [1]: http://dns-sd.org/ServiceTypes.html
    /// [2]: https://www.iana.org/form/ports-services
    /// [3]: http://upnp.org
    /// [4]: https://docs.macchina.io/edge/00200-UPnPSSDPTutorialAndUserGuide.html
    ///
    /// For now we just use post-association discovery (i.e., this discovers peers but those peers
    /// might not know our protocol or anything).
    async fn discover_peers(&self) -> Result<(), Box<dyn std::error::Error>>;

    /// Returns a name to a given peer. Guaranteed to exist in between peer_discovered and
    /// peer_lost.
    fn peer_name(&self, id: PeerId) -> Option<&str>;

    // async fn connect_to_peer(id: PeerId) -> Result<(), Box<dyn std::error::Error>>;
}
