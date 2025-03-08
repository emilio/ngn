#[macro_use]
extern crate log;

use std::sync::Arc;
use ngn::phy::{P2PSession, P2PSessionListener, LoggerListener, PeerId};

#[derive(Debug, Default)]
struct Listener {
    logger: LoggerListener
}

impl<S: P2PSession> P2PSessionListener<S> for Listener {
    fn peer_discovered(&self, sess: &S, peer_id: PeerId) {
        self.logger.peer_discovered(sess, peer_id);
        let Some(name) = sess.peer_name(peer_id) else { return };
        if name.starts_with("Rust") {
            let sess = sess.to_strong();
            tokio::spawn(async move {
                sess.connect_to_peer(peer_id).await
            });
        }
    }
}

#[tokio::main]
async fn main() -> ngn::phy::GenericResult<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    let interface_name = std::env::args().nth(1);
    let session = ngn::phy::dbus::Session::new(ngn::phy::dbus::SessionInit {
        interface_name: interface_name.as_deref(),
        device_name: "RustTest",
        go_intent: 14,
    }, Arc::new(Listener::default())).await?;

    info!("Starting find operation");
    session.discover_peers().await?;

    session.wait().await?;

    Ok(())
}
