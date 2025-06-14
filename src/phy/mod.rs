//! Main interface for P2P physical groups.

#[cfg(not(target_os = "android"))]
pub mod dbus;
#[cfg(target_os = "android")]
pub mod android;
mod protocol;

use log::trace;
use std::fmt::Debug;
use std::sync::Arc;

/// A handle for a given peer.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct PeerId(pub(crate) handy::Handle);

/// A handle for a given group.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct GroupId(pub(crate) handy::Handle);

pub trait P2PSessionListener<S: P2PSession>: Debug + Send + Sync {
    fn peer_discovered(&self, sess: &S, peer_id: PeerId) {
        trace!("Listener::peer_discovered({peer_id:?})");
        let peer_name = sess.peer_name(peer_id);
        trace!(" > name: {peer_name:?}");
    }

    fn peer_lost(&self, _: &S, peer_id: PeerId) {
        trace!("Listener::peer_lost({peer_id:?})");
    }

    /// Called when peer discovery stops, either by timeout or explicitly.
    ///
    /// TODO(emilio): What more info can we provide?
    fn peer_discovery_stopped(&self, _: &S) {
        trace!("Listener::peer_discovery_stopped()");
    }

    fn joined_group(&self, _: &S, group_id: GroupId, is_go: bool) {
        trace!("Listener::joined_group({group_id:?}, is_go={is_go}");
    }

    fn left_group(&self, _: &S, group_id: GroupId, is_go: bool) {
        trace!("Listener::left_group({group_id:?}, is_go={is_go}");
    }

    fn peer_joined_group(&self, _: &S, group_id: GroupId, peer_id: PeerId) {
        trace!("Listener::peer_joined_group({group_id:?}, {peer_id:?})");
    }

    fn peer_left_group(&self, _: &S, group_id: GroupId, peer_id: PeerId) {
        trace!("Listener::peer_left_group({group_id:?}, {peer_id:?})");
    }

    fn peer_messaged(&self, _: &S, peer_id: PeerId, group_id: GroupId, message: &[u8]) {
        trace!("Listener::peer_messaged({peer_id:?}, {group_id:?}, {message:?})");
    }
}

/// A listener implementation that logs.
#[derive(Debug, Default)]
pub struct LoggerListener;
impl<S: P2PSession> P2PSessionListener<S> for LoggerListener {
    // Do nothing, default implementation logs.
}

pub type GenericResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[async_trait::async_trait]
pub trait P2PSession: Sized + Debug + Send + Sync + 'static {
    /// The backend-specific arguments needed for initialization.
    type InitArgs<'a>: Sized + 'a;

    /// Create a new session.
    async fn new(
        args: Self::InitArgs<'_>,
        listener: Arc<dyn P2PSessionListener<Self>>,
    ) -> GenericResult<Arc<Self>>;

    /// Stop the session.
    async fn stop(&self) -> GenericResult<()>;

    /// Wait for the session to exit on its own (potentially never).
    async fn wait(&self) -> GenericResult<()>;

    fn to_strong(&self) -> Arc<Self> {
        // SAFETY: Sessions are always arc-allocated, see new().
        unsafe {
            Arc::increment_strong_count(self);
            Arc::from_raw(self)
        }
    }

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
    ///
    /// TODO(emilio): You might want to configure how persistent this really is etc.
    async fn discover_peers(&self) -> GenericResult<()>;

    /// Returns a name to a given peer. Guaranteed to exist in between peer_discovered and
    /// peer_lost.
    fn peer_name(&self, id: PeerId) -> Option<String>;

    async fn connect_to_peer(&self, id: PeerId) -> GenericResult<()>;

    /// Try to send a message to a given peer.
    async fn message_peer(&self, id: PeerId, message: &[u8]) -> GenericResult<()>;
}
