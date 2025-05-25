use adw::prelude::*;
use ngn::phy::{GroupId, LoggerListener, P2PSession, P2PSessionListener, PeerId};
use rand::Rng;
use std::sync::Arc;

pub const APP_ID: &str = "es.usal.P2pTest";

pub enum Event {
    PeerDiscovered { id: PeerId, name: String },
    PeerConnected { id: PeerId },
    PeerLost { id: PeerId },
}

#[derive(Debug)]
pub struct Listener {
    logger: LoggerListener,
    ui_event_sender: tokio::sync::mpsc::UnboundedSender<Event>,
}

impl Listener {
    pub fn new(ui_event_sender: tokio::sync::mpsc::UnboundedSender<Event>) -> Self {
        Self {
            logger: LoggerListener::default(),
            ui_event_sender,
        }
    }
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
        self.ui_event_sender
            .send(Event::PeerConnected { id: peer_id })
            .unwrap();
    }

    fn peer_discovered(&self, sess: &S, peer_id: PeerId) {
        self.logger.peer_discovered(sess, peer_id);
        let Some(name) = sess.peer_name(peer_id) else {
            return;
        };
        self.ui_event_sender
            .send(Event::PeerDiscovered { id: peer_id, name })
            .unwrap();
    }

    fn peer_lost(&self, sess: &S, peer_id: PeerId) {
        self.logger.peer_lost(sess, peer_id);
        self.ui_event_sender
            .send(Event::PeerLost { id: peer_id })
            .unwrap();
    }
}

pub fn build(
    app: &adw::Application,
    device_name: &str,
    session: &ngn::phy::dbus::Session,
    listener: Arc<Listener>,
    mut ui_event_receiver: tokio::sync::mpsc::UnboundedReceiver<Event>,
) {
    let content = gtk::Box::new(gtk::Orientation::Vertical, /* spacing = */ 0);
    let headerbar = adw::HeaderBar::new();
    let refresh_button = gtk::Button::from_icon_name("view-refresh");
    let peer_list_box = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .build();
    {
        let session = session.to_strong();
        let peer_list_box = peer_list_box.clone();
        refresh_button.connect_clicked(move |_| {
            // TODO: Maybe shouldn't clear the list?
            peer_list_box.remove_all();
            let session = Arc::clone(&session);
            super::rt().spawn(async move { session.discover_peers().await });
        });
    }

    headerbar.pack_end(&refresh_button);
    content.append(&headerbar);
    content.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    content.append(&peer_list_box);

    let session = session.to_strong();
    gtk::glib::MainContext::default().spawn_local(async move {
        while let Some(event) = ui_event_receiver.recv().await {
            match event {
                Event::PeerDiscovered { id, name } => {
                    let row = adw::ActionRow::builder().activatable(true).build();
                    row.set_title(&name);
                    unsafe {
                        row.set_data::<PeerId>("peer-id", id);
                    }
                    let session = Arc::clone(&session);
                    row.connect_activated(move |_| {
                        let session = session.to_strong();
                        super::rt().spawn(async move {
                            if let Err(e) = session.connect_to_peer(id).await {
                                error!("Failed to connect to peer {id:?}: {e}");
                                // TODO: Propagate error to UI
                            }
                        });
                    });
                    peer_list_box.append(&row);
                }
                Event::PeerConnected { id } | Event::PeerLost { id } => {
                    let mut i = 0;
                    let mut class_to_add = "connected";
                    let mut class_to_remove = "disconnected";
                    if matches!(event, Event::PeerLost { .. }) {
                        std::mem::swap(&mut class_to_add, &mut class_to_remove);
                    };
                    loop {
                        let Some(row) = peer_list_box.row_at_index(i) else {
                            break;
                        };

                        let peer_id = unsafe {
                            *row.data::<PeerId>("peer-id")
                                .expect("Peer id not set?")
                                .as_ref()
                        };

                        if peer_id == id {
                            row.add_css_class(class_to_add);
                            row.remove_css_class(class_to_remove);
                            break;
                        }

                        i += 1;
                    }
                }
            }
        }
    });

    /*
    let sess = sess.to_strong();
    tokio::spawn(async move {
        if let Err(e) = sess.connect_to_peer(peer_id).await {
            error!("Couldn't connect to {peer_id:?}: {e}");
        }
    });*/

    // Create a window and set the title
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(device_name)
        .content(&content)
        .build();

    // Present window
    window.present();
}
