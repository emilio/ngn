use adw::prelude::*;
use ngn::{
    protocol::PeerIdentity, GroupId, LoggerListener, P2PSession, P2PSessionListener, PeerId,
};
use rand::Rng;
use std::sync::{Arc, OnceLock};

pub const APP_ID: &str = "es.usal.P2pTest";

pub fn rt() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            // 10 MiB should be plenty even for debug builds.
            .thread_stack_size(10 * 1024 * 1024)
            .worker_threads(3)
            .enable_all()
            .build()
            .unwrap()
    })
}

static SESSION: OnceLock<Arc<ngn::platform::dbus::Session>> = OnceLock::new();
fn start_session(
    device_name: &str,
    interface_name: Option<&str>,
    nickname: String,
    listener: Arc<Listener>,
) {
    SESSION.get_or_init(|| {
        // TODO: Use get_or_try_init and propagate the error to the UI once that's stable.
        let rt = rt();
        let session = rt
            .block_on(async move {
                // TODO: Persist keys and get nickname from user.
                let identity = ngn::protocol::identity::new_own_id(nickname)?;
                ngn::platform::dbus::Session::new(
                    ngn::platform::dbus::SessionInit {
                        interface_name,
                        device_name,
                        identity,
                        go_intent: 1,
                    },
                    listener,
                )
                .await
            })
            .expect("Couldn't init P2P session");

        rt.spawn({
            let session = session.clone();
            async move { session.wait().await }
        });

        session
    });
}

pub enum Event {
    PeerDiscovered { id: PeerId, identity: PeerIdentity },
    PeerConnected { id: PeerId, identity: PeerIdentity },
    PeerLost { id: PeerId },
    PeerDiscoveryStopped,
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
        let Some(identity) = sess.peer_identity(peer_id) else {
            return;
        };
        self.ui_event_sender
            .send(Event::PeerConnected {
                id: peer_id,
                identity,
            })
            .unwrap();
    }

    fn peer_discovered(&self, sess: &S, peer_id: PeerId) {
        self.logger.peer_discovered(sess, peer_id);
        let Some(identity) = sess.peer_identity(peer_id) else {
            return;
        };
        self.ui_event_sender
            .send(Event::PeerDiscovered {
                id: peer_id,
                identity,
            })
            .unwrap();
    }

    fn peer_lost(&self, sess: &S, peer_id: PeerId) {
        self.logger.peer_lost(sess, peer_id);
        self.ui_event_sender
            .send(Event::PeerLost { id: peer_id })
            .unwrap();
    }

    fn peer_discovery_stopped(&self, sess: &S) {
        self.logger.peer_discovery_stopped(sess);
        self.ui_event_sender
            .send(Event::PeerDiscoveryStopped)
            .unwrap();
    }
}

fn start_discovery(peer_list_box: &gtk::ListBox, refresh_button: &gtk::Button) {
    let Some(session) = SESSION.get() else { return };

    // TODO: Maybe shouldn't clear the list?
    peer_list_box.remove_all();
    refresh_button.add_css_class("discovering");
    let session = session.clone();
    rt().spawn(async move { session.discover_peers().await });
}

pub fn build(app: &adw::Application, device_name: &str, interface_name: Option<&str>) {
    let content = adw::ToolbarView::new();
    let refresh_button = gtk::Button::from_icon_name("view-refresh");
    let peer_list_box = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .build();
    {
        let headerbar = adw::HeaderBar::new();
        refresh_button.set_sensitive(false);
        {
            let peer_list_box = peer_list_box.clone();
            refresh_button.connect_clicked(move |button| {
                start_discovery(&peer_list_box, button);
            });
        }

        headerbar.pack_end(&refresh_button);
        content.add_top_bar(&headerbar);
    }

    let (ui_event_sender, mut ui_event_receiver) = tokio::sync::mpsc::unbounded_channel();
    {
        let identity_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        identity_box.add_css_class("toolbar");
        identity_box.set_halign(gtk::Align::Center);

        let identity_entry = gtk::Entry::new();
        identity_entry.set_icon_from_icon_name(gtk::EntryIconPosition::Secondary, Some("go-next"));
        identity_entry.set_icon_activatable(gtk::EntryIconPosition::Secondary, true);
        identity_entry.set_placeholder_text(Some("Nickname"));

        let device_name = device_name.to_owned();
        let interface_name = interface_name.map(|n| n.to_owned());
        // HACK: Slightly hacky, but we only activate once.
        let ui_event_sender = std::cell::RefCell::new(Some(ui_event_sender));
        let peer_list_box = peer_list_box.clone();
        let refresh_button = refresh_button.clone();
        identity_entry.connect_activate(move |entry| {
            let nickname = entry.buffer().text();
            if nickname.is_empty() {
                return;
            }
            entry.set_sensitive(false);
            entry.set_icon_from_icon_name(gtk::EntryIconPosition::Secondary, None);
            let listener = Arc::new(Listener::new(ui_event_sender.borrow_mut().take().unwrap()));
            start_session(
                &device_name,
                interface_name.as_deref(),
                nickname.to_string(),
                listener,
            );
            refresh_button.set_sensitive(true);
            if let Some(sess) = SESSION.get() {
                entry.set_text(&sess.own_identity().to_string());
            }
            start_discovery(&peer_list_box, &refresh_button);
        });

        identity_box.append(&identity_entry);
        content.add_top_bar(&identity_box);
    }

    content.set_content(Some(&peer_list_box));

    gtk::glib::MainContext::default().spawn_local(async move {
        let find_row_with_id = |id: PeerId| {
            let mut i = 0;
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
                    return Some(row.dynamic_cast::<adw::ActionRow>().unwrap());
                }

                i += 1;
            }
            None
        };
        while let Some(event) = ui_event_receiver.recv().await {
            match event {
                Event::PeerDiscovered { id, identity } => {
                    let row = adw::ActionRow::builder().activatable(true).build();
                    row.set_title(&identity.physical.to_string());
                    unsafe {
                        row.set_data::<PeerId>("peer-id", id);
                    }
                    row.connect_activated(move |row| {
                        let Some(session) = SESSION.get() else { return };
                        let connected = row.css_classes().iter().any(|c| c == "connected");
                        rt().spawn(async move {
                            if !connected {
                                if let Err(e) = session.connect_to_peer(id).await {
                                    error!("Failed to connect to peer {id:?}: {e}");
                                    // TODO: Propagate error to UI
                                }
                            } else {
                                let msg = hi_msg();
                                if let Err(e) = session.message_peer(id, msg.as_bytes()).await {
                                    error!("Failed to message peer {id:?}: {e}");
                                    // TODO: Propagate error to UI
                                }
                            }
                        });
                    });
                    peer_list_box.append(&row);
                }
                Event::PeerConnected { id, identity } => {
                    if let Some(row) = find_row_with_id(id) {
                        if let Some(logical_id) = identity.logical {
                            row.set_title(&logical_id.to_string());
                            row.set_subtitle(&identity.physical.to_string());
                        }
                        row.add_css_class("connected");
                        row.remove_css_class("disconnected");
                    }
                }
                Event::PeerLost { id } => {
                    if let Some(row) = find_row_with_id(id) {
                        row.remove_css_class("connected");
                        row.add_css_class("disconnected");
                    }
                }
                Event::PeerDiscoveryStopped => {
                    refresh_button.remove_css_class("discovering");
                }
            }
        }
    });

    // Create a window and set the title
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(device_name)
        .content(&content)
        .build();

    // Present window
    window.present();
}
