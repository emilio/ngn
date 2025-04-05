#[macro_use]
extern crate log;

use ngn::phy::{LoggerListener, P2PSession, P2PSessionListener, GroupId, PeerId};
use rand::Rng;
use std::sync::Arc;

#[derive(Debug, Default)]
struct Listener {
    logger: LoggerListener,
}

fn hi_msg() -> String {
    let mut rng = rand::rng();
    let random_chars = rng.random_range(0usize..=(1024 * 10));
    let mut msg = format!("Hi there, this is a message with {random_chars} random chars: ");
    msg.extend(
        rng.sample_iter(rand::distr::Alphanumeric)
            .take(random_chars)
            .map(|b| b as char),
    );
    msg
}

impl<S: P2PSession> P2PSessionListener<S> for Listener {
    fn peer_joined_group(&self, sess: &S, group_id: GroupId, peer_id: PeerId) {
        self.logger.peer_joined_group(sess, group_id, peer_id);
        let sess = sess.to_strong();
        tokio::spawn(async move {
            if let Err(e) = sess.message_peer(peer_id, hi_msg().as_bytes()).await {
                error!("Couldn't message {peer_id:?}: {e}");
            }
        });
    }

    fn peer_discovered(&self, sess: &S, peer_id: PeerId) {
        self.logger.peer_discovered(sess, peer_id);
        let Some(name) = sess.peer_name(peer_id) else {
            return;
        };
        if !name.starts_with("Rust") {
            return;
        }
        let sess = sess.to_strong();
        tokio::spawn(async move {
            if let Err(e) = sess.connect_to_peer(peer_id).await {
                error!("Couldn't connect to {peer_id:?}: {e}");
            }
        });
    }
}

#[tokio::main]
async fn main() -> ngn::phy::GenericResult<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    let interface_name = std::env::args().nth(1);
    let session = ngn::phy::dbus::Session::new(
        ngn::phy::dbus::SessionInit {
            interface_name: interface_name.as_deref(),
            device_name: "RustTest",
            go_intent: 14,
        },
        Arc::new(Listener::default()),
    )
    .await?;

    info!("Starting find operation");
    session.discover_peers().await?;

    session.wait().await?;

    Ok(())
}
