//! DBus implementation of the physical layer, interfacing with wpa_supplicant.
//!
//! Known limitation: Only current method of provisioning the GO IP address is via IPV6 link-local
//! addresses. This means that if you use something like private address assignment via something
//! like `slaac private`, it will fail to connect to the GO. Other neighbor discovery approaches
//! could be used in the future.

mod store;
pub mod wpa_supplicant;

use crate::{
    protocol::{
        self, identity::OwnIdentity, ControlMessage, GroupInfo, P2pPorts, PeerAddress,
        PeerGroupInfo, PeerIdentity, PeerInfo, PeerOwnIdentifier, PhysiscalPeerIdentity,
        GO_CONTROL_PORT,
    },
    utils::{self, trivial_error},
    GenericResult, GroupId, P2PSession, P2PSessionListener, PeerId,
};

use futures_lite::StreamExt;
use log::{error, trace, warn};
use macaddr::MacAddr;
use parking_lot::RwLock;
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv6Addr, SocketAddr, SocketAddrV6},
    sync::{Arc, OnceLock},
    time::Duration,
};
use store::{DbusPath, DbusStore};
use tokio::{self, net::TcpListener, task::JoinHandle};
use wpa_supplicant::{p2pdevice::P2PDeviceProxy, wpa_supplicant::WpaSupplicantProxy};
use zbus::zvariant::{OwnedObjectPath, Value};

const WPS_METHOD: &str = "pbc";

#[derive(Debug)]
struct DbusPeerData {
    proxy: wpa_supplicant::peer::PeerProxy<'static>,
    path: OwnedObjectPath,
}

type Peer = PeerInfo<DbusPeerData>;

impl DbusPath for Peer {
    fn path(&self) -> &OwnedObjectPath {
        &self.data.path
    }
}

#[derive(Debug)]
struct DbusGroupData {
    /// Proxy to the group object.
    proxy: wpa_supplicant::group::GroupProxy<'static>,
    /// Proxy to the interface object that connects the nodes in this group.
    iface: wpa_supplicant::interface::InterfaceProxy<'static>,
    /// DBUS Path of the interface.
    iface_path: OwnedObjectPath,
    /// Path of the group.
    path: OwnedObjectPath,
    /// Dev address of the GO
    go_dev_addr: MacAddr,
}

type Group = GroupInfo<DbusGroupData>;

impl DbusPath for Group {
    fn path(&self) -> &OwnedObjectPath {
        &self.data.path
    }
}

/// Global state for a P2P session.
#[derive(Debug)]
pub struct Session {
    system_bus: zbus::Connection,
    wpa_supplicant: WpaSupplicantProxy<'static>,
    p2pdevice: P2PDeviceProxy<'static>,
    go_intent: u32,
    peers: RwLock<DbusStore<Peer>>,
    groups: RwLock<DbusStore<Group>>,
    listener: Arc<dyn P2PSessionListener<Self>>,
    /// Task handle to our run loop. Canceled and awaited on drop.
    run_loop_task: RwLock<Option<JoinHandle<GenericResult<()>>>>,
    /// Our own logical identity.
    identity: OwnIdentity,
    /// Our own P2P identifier address.
    own_phy_id: PeerOwnIdentifier,
}

impl Drop for Session {
    fn drop(&mut self) {
        if let Some(ref t) = *self.run_loop_task.read() {
            t.abort();
        }
    }
}

pub struct SessionInit<'a> {
    /// The interface name to use, or `None` to use the first available interface.
    pub interface_name: Option<&'a str>,
    /// The device name we're advertised as.
    pub device_name: &'a str,
    /// The identity we use to communicate with other peers.
    pub identity: OwnIdentity,
    /// Our group owner intent, from 0 to 15.
    pub go_intent: u32,
}

#[async_trait::async_trait]
impl P2PSession for Session {
    type InitArgs<'a> = SessionInit<'a>;

    /// Create a new P2P session.
    async fn new(
        init: SessionInit<'_>,
        listener: Arc<dyn P2PSessionListener<Self>>,
    ) -> GenericResult<Arc<Self>> {
        trace!("Trying to connect to system bus");
        let system_bus = zbus::Connection::system().await?;

        trace!("Trying to create wpa_supplicant proxy");
        let wpa_supplicant =
            wpa_supplicant::wpa_supplicant::WpaSupplicantProxy::new(&system_bus).await?;

        trace!("Scanning for p2p capabilities");

        let caps = wpa_supplicant.capabilities().await?;
        trace!("Got {caps:?}");

        if !caps.iter().any(|s| s == "p2p") {
            return Err(trivial_error!("wpa_supplicant has no p2p support"));
        }

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
                wpa_supplicant::p2pdevice::P2PDeviceProxy::new(&system_bus, iface_path.clone())
                    .await?
            }
            None => {
                trace!("Looking for interfaces with p2p support");

                let ifaces = wpa_supplicant.interfaces().await?;
                trace!("Got interfaces: {ifaces:?}");

                let mut result = None;
                for iface_path in ifaces {
                    trace!("trying {iface_path}");
                    match wpa_supplicant::p2pdevice::P2PDeviceProxy::new(
                        &system_bus,
                        iface_path.clone(),
                    )
                    .await
                    {
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
                    None => {
                        return Err(trivial_error!(
                            "Couldn't create P2P proxy for any interface"
                        ))
                    }
                }
            }
        };

        let cur_config = p2pdevice.p2pdevice_config().await?;
        trace!("Initial device config: {cur_config:?}");

        // TODO: Make configurable.
        p2pdevice
            .set_p2pdevice_config({
                let mut config = HashMap::new();
                config.insert("DeviceName", init.device_name.into());
                config.insert("GOIntent", init.go_intent.into());
                config
            })
            .await?;

        // TODO(emilio): use device_address() if available.
        let own_phy_id = PeerOwnIdentifier::Name(init.device_name.into());

        /*
                let dev_addr = p2pdevice.device_address().await?;
                trace!("Own P2P device address: {dev_addr:?}");
                let Some(dev_addr) = utils::to_mac_addr(&dev_addr) else {
                    return Err(trivial_error!(
                        "Expected a valid mac address from
        P2PDevice::device_address()"
                    ));
                };
                */

        trace!("Successfully initialized P2P session");
        let session = Arc::new(Self {
            system_bus,
            wpa_supplicant,
            p2pdevice,
            go_intent: init.go_intent,
            identity: init.identity,
            peers: Default::default(),
            groups: Default::default(),
            own_phy_id,
            listener,
            run_loop_task: RwLock::new(None),
        });

        let handle = tokio::spawn(Session::run_loop(Arc::clone(&session)));
        *session.run_loop_task.write() = Some(handle);

        Ok(session)
    }

    async fn wait(&self) -> GenericResult<()> {
        trace!("Session::wait");
        let handle = self.run_loop_task.write().take();
        if let Some(t) = handle {
            t.await??;
        }
        Ok(())
    }

    async fn stop(&self) -> GenericResult<()> {
        trace!("Session::stop");
        // TODO: More graceful termination.
        self.groups.write().clear();
        self.peers.write().clear();
        if let Some(ref t) = *self.run_loop_task.read() {
            t.abort();
        }
        self.wait().await
    }

    async fn discover_peers(&self) -> GenericResult<()> {
        trace!("Session::discover_peers");
        self.p2pdevice.find(HashMap::default()).await?;
        Ok(())
    }

    fn peer_identity(&self, id: PeerId) -> Option<PeerIdentity> {
        Some(self.peers.read().get(id.0)?.identity.clone())
    }

    fn all_peers(&self) -> Vec<(PeerId, PeerIdentity)> {
        self.peers
            .read()
            .iter_with_handles()
            .map(|(id, info)| (PeerId(id), info.identity.clone()))
            .collect()
    }

    fn own_identity(&self) -> &OwnIdentity {
        &self.identity
    }

    async fn connect_to_peer(&self, id: PeerId) -> GenericResult<()> {
        trace!("Session::connect_to_peer({id:?})");
        let peer_path = {
            let guard = self.peers.read();
            match guard.get(id.0) {
                Some(p) => p.data.path.to_owned(),
                None => return Err(trivial_error!("Can't locate peer")),
            }
        };
        self.connect_to_peer_by_path(peer_path).await?;
        Ok(())
    }

    async fn message_peer(&self, id: PeerId, message: &[u8]) -> GenericResult<()> {
        let (peer_address, scope_id) = {
            let peers = self.peers.read();
            let groups = self.groups.read();
            let Some(peer) = peers.get(id.0) else {
                return Err(trivial_error!("Peer was lost (stale handle?)"));
            };
            // Choose one arbitrary group to connect to it.
            let Some(group_id) = peer.groups.first() else {
                // TODO: Maybe we want to call connect_to_peer automatically?
                return Err(trivial_error!("Peer is not connected to any group"));
            };
            let Some(group) = groups.get(group_id.0) else {
                // TODO: Maybe we want to call connect_to_peer automatically?
                return Err(trivial_error!("Group not found"));
            };
            let Some(info) = group.peers.get(&id) else {
                return Err(trivial_error!(
                    "Peer doesn't have a link local address (yet?)"
                ));
            };
            (info.address.clone(), group.scope_id)
        };
        let socket_addr =
            protocol::peer_to_socket_addr(peer_address.address, scope_id, peer_address.ports.p2p);
        utils::retry_timeout(Duration::from_secs(2), 5, || {
            protocol::send_message(Some(&self.identity), &socket_addr, message)
        })
        .await
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

    fn peer_id_from_address(&self, group_id: GroupId, address: &SocketAddr) -> Option<PeerId> {
        // TODO: This lookup could be faster, really.
        let groups = self.groups.read();
        let Some(group) = groups.get(group_id.0) else {
            error!("Group {group_id:?} lost?");
            return None;
        };
        for (peer_id, info) in &group.peers {
            if info.address.address == address.ip() {
                return Some(*peer_id);
            }
        }
        error!("Address {address:?} couldn't be mapped to a known peer in the group!");
        None
    }

    async fn send_control_message(
        &self,
        ip: IpAddr,
        control_port: u16,
        scope_id: u32,
        message: ControlMessage,
    ) -> GenericResult<()> {
        trace!("Session::send_control_message({ip:?}, {scope_id}, {message:?})");
        let addr = protocol::peer_to_socket_addr(ip, scope_id, control_port);
        let msg = bincode::encode_to_vec(message, bincode::config::standard())?;
        utils::retry_timeout(Duration::from_secs(2), 5, || {
            protocol::send_message(None, &addr, &msg)
        })
        .await
    }

    async fn establish_control_channel(
        session: Arc<Self>,
        control_listener: TcpListener,
        group_id: GroupId,
        scope_id: u32,
        own_ports: P2pPorts,
        is_go: bool,
    ) -> GenericResult<()> {
        trace!(
            "Session::establish_control_channel({group_id:?}, {scope_id}, {own_ports:?}, {is_go})"
        );
        loop {
            // TODO: Use a buffered reader.
            // TODO: Keep a single stream around for faster bi-lateral communication maybe?
            let (mut stream, address) = control_listener.accept().await?;
            let session = Arc::clone(&session);
            tokio::spawn(async move {
                trace!("Incoming connection from {address:?}");
                while let Ok(control_message) =
                    protocol::read_control_message(&mut stream, &address).await
                {
                    trace!("Got control message {control_message:?} on group {group_id:?}");
                    match control_message {
                        ControlMessage::Associate {
                            physical_id,
                            logical_id,
                            key_exchange_public_key,
                            ports,
                        } => {
                            let (peer_id, key_exchange_public_key) = {
                                let mut peers = session.peers.write();
                                let mut result = None;
                                for (id, peer) in peers.iter_mut_with_handles() {
                                    trace!(" {:?} -> {:?}", id, peer.identity.physical);
                                    if peer.identity.physical.matches(&physical_id) {
                                        if peer
                                            .identity
                                            .logical
                                            .as_ref()
                                            .is_some_and(|i| *i != logical_id)
                                        {
                                            error!("Refusing to associate {id:?} with different logical {logical_id:?}");
                                            break;
                                        }
                                        if peer.groups.contains(&group_id) {
                                            error!("Refusing to associate {id:?} with {group_id:?} again");
                                            break;
                                        }
                                        peer.groups.push(group_id);
                                        peer.identity.logical = Some(logical_id);
                                        if let Err(e) =
                                            peer.key_exchange.finish(&key_exchange_public_key)
                                        {
                                            error!("Failed to finish key exchange ({:?})", e);
                                        }
                                        result = Some((
                                            PeerId(id),
                                            peer.key_exchange.export_public_key(),
                                        ));
                                        break;
                                    }
                                }
                                match result {
                                    Some(r) => r,
                                    None => {
                                        error!("Couldn't associate with {physical_id:?}");
                                        continue;
                                    }
                                }
                            };
                            let address = address.ip();
                            {
                                let mut groups = session.groups.write();
                                let Some(group) = groups.get_mut(group_id.0) else {
                                    warn!("Group {group_id:?} was torn down?");
                                    continue;
                                };
                                let address = PeerAddress { address, ports };
                                group.peers.insert(peer_id, PeerGroupInfo { address });
                            };
                            if is_go {
                                // Try to send the association request back to the peer. This
                                // ensures that the peer notifies of the connection (via the
                                // is_new_connection code-path).
                                //
                                // TODO(emilio): It'd be a lot cleaner if the peer would've access
                                // to the GO device address on negotiation success. Then, it could
                                // just associate and notify itself...
                                let result = session
                                    .send_control_message(
                                        address,
                                        ports.control,
                                        scope_id,
                                        ControlMessage::Associate {
                                            physical_id: session.own_phy_id.clone(),
                                            logical_id: session.identity.to_public(),
                                            key_exchange_public_key,
                                            ports: own_ports,
                                        },
                                    )
                                    .await;
                                if let Err(e) = result {
                                    error!("Failed to send associate message from GO to {peer_id:?}: {e}");
                                }
                            }
                            trace!("Notifying of new association of {peer_id:?} to {group_id:?}");
                            session
                                .listener
                                .peer_joined_group(&session, group_id, peer_id);
                        }
                    }
                }
            });
        }
        #[allow(unreachable_code)]
        Ok(())
    }

    async fn listen_to_peer_messages(
        session: Arc<Self>,
        listener: TcpListener,
        group_id: GroupId,
        scope_id: u32,
    ) -> GenericResult<()> {
        trace!("Session::listen_to_peer_messages({group_id:?}, {scope_id})");
        loop {
            // TODO: Use a buffered reader.
            // TODO: Keep a single stream around for faster bi-lateral communication maybe?
            let (mut stream, address) = listener.accept().await?;
            let Some(peer_id) = session.peer_id_from_address(group_id, &address) else {
                warn!("Got message from {address:?} but couldn't map that to a peer in group {group_id:?}");
                continue;
            };
            let Some(peer_identity) = session
                .peers
                .read()
                .get(peer_id.0)
                .and_then(|p| p.identity.logical.clone())
            else {
                warn!("Got message from {address:?} but haven't received his keys yet");
                continue;
            };
            let session = Arc::clone(&session);
            tokio::spawn(async move {
                trace!("Incoming connection from {address:?}");
                while let Ok(buf) = protocol::read_peer_message(
                    &session.identity,
                    &peer_identity,
                    &mut stream,
                    &address,
                )
                .await
                {
                    trace!(
                        "Got message from socket: {:?}",
                        String::from_utf8_lossy(&buf)
                    );
                    session
                        .listener
                        .peer_messaged(&session, peer_id, group_id, &buf);
                }
            });
        }
        #[allow(unreachable_code)]
        Ok(())
    }

    async fn group_task(session: Arc<Self>, group_id: GroupId) -> GenericResult<()> {
        trace!("Session::group_task({group_id:?})");

        let (is_go, go_ip, go_dev_addr, scope_id, proxy) =
            match session.groups.read().get(group_id.0) {
                Some(g) => (
                    g.is_go,
                    g.go_ip_address,
                    g.data.go_dev_addr,
                    g.scope_id,
                    g.data.proxy.clone(),
                ),
                None => {
                    error!("Didn't find {group_id:?} on group_task start!");
                    return Err(trivial_error!("Didn't find group on group_task start!"));
                }
            };

        let (control_listener, p2p_listener) = tokio::try_join!(
            TcpListener::bind(SocketAddrV6::new(
                Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0),
                if is_go { GO_CONTROL_PORT } else { 0 },
                /* flowinfo = */ 0,
                scope_id,
            )),
            TcpListener::bind(SocketAddrV6::new(
                Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0),
                0,
                /* flowinfo = */ 0,
                scope_id,
            )),
        )?;

        let my_ports = P2pPorts {
            control: control_listener.local_addr()?.port(),
            p2p: p2p_listener.local_addr()?.port(),
        };

        trace!(" > go = {is_go}, ports = {my_ports:?}, go_ip = {go_ip:?}");
        if !is_go {
            let key_exchange_public_key = {
                trace!(" > GO dev addr is {}", go_dev_addr);
                let peers = session.peers.read();
                let id = PeerOwnIdentifier::DevAddr(go_dev_addr.into());
                let key = peers.iter().find_map(|p| {
                    p.identity
                        .physical
                        .matches(&id)
                        .then(|| p.key_exchange.export_public_key())
                });
                match key {
                    Some(key) => key,
                    None => return Err(trivial_error!("Couldn't find GO by dev addr")),
                }
            };

            let control_message = ControlMessage::Associate {
                physical_id: session.own_phy_id.clone(),
                logical_id: session.identity.to_public(),
                ports: my_ports,
                key_exchange_public_key,
            };
            tokio::try_join!(
                Self::listen_to_peer_messages(
                    Arc::clone(&session),
                    p2p_listener,
                    group_id,
                    scope_id
                ),
                Self::establish_control_channel(
                    Arc::clone(&session),
                    control_listener,
                    group_id,
                    scope_id,
                    my_ports,
                    is_go,
                ),
                async move {
                    trace!("Trying to send control message to {go_ip:?}");
                    session
                        .send_control_message(go_ip, GO_CONTROL_PORT, scope_id, control_message)
                        .await
                },
            )?;
            return Ok(());
        }

        let mut peer_joined = proxy.receive_peer_joined().await?;
        let mut peer_left = proxy.receive_peer_disconnected().await?;

        tokio::try_join!(
            async {
                while let Some(msg) = peer_joined.next().await {
                    let args = msg.args()?;
                    let peer = args.peer();
                    trace!("Peer joined {group_id:?}: {peer:?}, waiting for association");
                }
                Ok(())
            },
            async {
                while let Some(msg) = peer_left.next().await {
                    let args = msg.args()?;
                    let peer = args.peer();
                    trace!("Peer left {group_id:?}: {peer:?}");
                    let peer_id = {
                        let mut peers = session.peers.write();
                        let Some(peer_id) = peers.id_by_path(peer) else {
                            error!("couldn't find peer {peer:?}");
                            continue;
                        };
                        let peer_id = PeerId(peer_id);

                        let mut groups = session.groups.write();
                        let this_group = groups.get_mut(group_id.0).unwrap();
                        let this_peer = peers.get_mut(peer_id.0).unwrap();
                        debug_assert!(
                            this_group.peers.contains_key(&peer_id),
                            "Peer not associated to group?"
                        );
                        debug_assert!(
                            this_peer.groups.contains(&group_id),
                            "Group not associated to peer?"
                        );
                        this_peer.groups.retain(|g| *g != group_id);
                        this_group.peers.remove(&peer_id);
                        peer_id
                    };
                    // TODO: Broadcast to non-GOs?
                    session
                        .listener
                        .peer_left_group(&session, group_id, peer_id);
                }
                Ok(())
            },
            Self::listen_to_peer_messages(Arc::clone(&session), p2p_listener, group_id, scope_id),
            Self::establish_control_channel(
                Arc::clone(&session),
                control_listener,
                group_id,
                scope_id,
                my_ports,
                is_go,
            ),
        )?;
        Ok(())
    }

    async fn run_loop(session: Arc<Self>) -> GenericResult<()> {
        trace!("Session::run_loop");

        let mut find_stopped = session.p2pdevice.receive_find_stopped().await?;

        let mut device_found = session.p2pdevice.receive_device_found().await?;

        let mut device_lost = session.p2pdevice.receive_device_lost().await?;

        let mut invitation_received = session.p2pdevice.receive_invitation_received().await?;
        let mut invitation_result = session.p2pdevice.receive_invitation_result().await?;

        let mut wps_failed = session.p2pdevice.receive_wps_failed().await?;

        let mut group_started = session.p2pdevice.receive_group_started().await?;
        let mut group_finished = session.p2pdevice.receive_group_finished().await?;
        let mut group_formation_failure =
            session.p2pdevice.receive_group_formation_failure().await?;

        let mut go_negotiation_request = session.p2pdevice.receive_gonegotiation_request().await?;
        let mut go_negotiation_success = session.p2pdevice.receive_gonegotiation_success().await?;
        let mut go_negotiation_failure = session.p2pdevice.receive_gonegotiation_failure().await?;

        let mut persistent_group_added = session.p2pdevice.receive_persistent_group_added().await?;
        let mut persistent_group_removed =
            session.p2pdevice.receive_persistent_group_removed().await?;

        let mut pd_failure = session
            .p2pdevice
            .receive_provision_discovery_failure()
            .await?;
        let mut pd_req_display_pin = session
            .p2pdevice
            .receive_provision_discovery_request_display_pin()
            .await?;
        let mut pd_rsp_display_pin = session
            .p2pdevice
            .receive_provision_discovery_response_display_pin()
            .await?;
        let mut pd_req_enter_pin = session
            .p2pdevice
            .receive_provision_discovery_request_enter_pin()
            .await?;
        let mut pd_rsp_enter_pin = session
            .p2pdevice
            .receive_provision_discovery_response_enter_pin()
            .await?;
        let mut pd_pbc_req = session
            .p2pdevice
            .receive_provision_discovery_pbcrequest()
            .await?;
        let mut pd_pbc_rsp = session
            .p2pdevice
            .receive_provision_discovery_pbcresponse()
            .await?;

        // Properties
        // TODO? This seems silly (as noted in the docs, there can be concurrent groups so watching
        // the group property doesn't make much sense).
        // let mut group_changed = session.p2pdevice.receive_group_changed().await;
        let mut peers_changed = session.p2pdevice.receive_peers_changed().await;
        let mut persistent_groups_changed =
            session.p2pdevice.receive_persistent_groups_changed().await;

        tokio::try_join!(
            async {
                while let Some(msg) = wps_failed.next().await {
                    trace!("WPS failed");
                    let args = msg.args()?;
                    let args = args.args();
                    error!("WPS failed: {args:?}");
                }
                Ok::<_, zbus::Error>(())
            },
            async {
                while let Some(_msg) = find_stopped.next().await {
                    trace!("Find stopped");
                    session.listener.peer_discovery_stopped(&session);
                    // TODO: Maybe restart?
                    // p2pdevice.find(HashMap::default()).await?;
                }
                Ok(())
            },
            async {
                while let Some(msg) = device_found.next().await {
                    let args = msg.args()?;
                    let path = args.path().to_owned();
                    trace!("Found device at {path}");

                    let proxy =
                        wpa_supplicant::peer::PeerProxy::new(&session.system_bus, &path).await?;
                    let name = proxy.device_name().await?;
                    let dev_addr = proxy.device_address().await?;
                    trace!("Peer name: {name:?}, peer dev addr: {dev_addr:?}");

                    let Some(dev_addr) = utils::to_mac_addr(&dev_addr) else {
                        error!("Expected a valid mac address for peer {name}");
                        continue;
                    };

                    let physical_identity = PhysiscalPeerIdentity { name, dev_addr };

                    let handle = {
                        let mut peers = session.peers.write();
                        if let Some(id) = peers.id_by_path(&path) {
                            let existing = peers.get_mut(id).expect("DBUS store out of sync");
                            trace!("Peer was already registered (from previous scan?) with identity {:?}", existing.identity);
                            // TODO(emilio): Consider not notifying? Kinda puts the burden of
                            // preserving peer list to the parent.
                            id
                        } else {
                            peers.insert(Peer {
                                identity: PeerIdentity {
                                    physical: physical_identity,
                                    logical: None,
                                },
                                key_exchange: protocol::key_exchange::KeyExchange::new().unwrap(),
                                groups: Vec::new(),
                                data: DbusPeerData {
                                    proxy,
                                    path: path.into(),
                                },
                            })
                        }
                    };

                    session.listener.peer_discovered(&session, PeerId(handle));
                }
                Ok(())
            },
            async {
                while let Some(msg) = device_lost.next().await {
                    let args = msg.args()?;
                    let peer_path = args.path();
                    trace!("Lost device at {peer_path}");

                    let (peer_id, groups_disconnected) = {
                        let mut peers = session.peers.write();
                        let id = match peers.id_by_path(peer_path) {
                            Some(id) => id,
                            None => {
                                error!("Got unknown device lost {peer_path}");
                                continue;
                            }
                        };

                        let peer = match peers.get_mut(id) {
                            Some(p) => p,
                            None => {
                                error!("Got unknown device lost {peer_path}, {id:?}");
                                continue;
                            }
                        };

                        trace!("Peer lost: {peer:?}");
                        (PeerId(id), std::mem::take(&mut peer.groups))
                    };

                    // Remove the peer for any outstanding groups before notifying the
                    // listener of the peer being lost.
                    if !groups_disconnected.is_empty() {
                        let mut groups = session.groups.write();
                        for group_id in &groups_disconnected {
                            let group = match groups.get_mut(group_id.0) {
                                Some(g) => g,
                                None => {
                                    error!("Tried to disconnect peer {peer_id:?} from {group_id:?}, but didn't find group");
                                    continue;
                                }
                            };
                            let removed = group.peers.remove(&peer_id);
                            debug_assert!(
                                removed.is_some(),
                                "Tried to disconnect peer {peer_id:?} from {group:?}, but didn't find peer in group peers"
                            );
                        }
                        for group_id in &groups_disconnected {
                            session
                                .listener
                                .peer_left_group(&session, *group_id, peer_id);
                        }
                    }

                    session.listener.peer_lost(&session, peer_id);
                    let removed = session.peers.write().remove(peer_id.0);
                    debug_assert!(removed.is_some(), "Found id but couldn't remove peer?");
                }
                Ok(())
            },
            async {
                while let Some(msg) = invitation_received.next().await {
                    let args = msg.args()?;
                    let props = args.properties();
                    trace!("Got invitation: {props:?}");
                }
                Ok(())
            },
            async {
                while let Some(msg) = invitation_result.next().await {
                    let args = msg.args()?;
                    let props = args.invite_result();
                    trace!("Got invitation result: {props:?}");
                }
                Ok(())
            },
            async {
                while let Some(msg) = group_started.next().await {
                    let args = msg.args()?;
                    let props = args.properties();
                    trace!("Group started: {props:?}");
                    let iface_path = match props.get("interface_object") {
                        Some(Value::ObjectPath(o)) => o.to_owned(),
                        other => {
                            error!("Expected an interface object path, got {other:?}");
                            continue;
                        }
                    };
                    trace!("Current interface path is {iface_path:?}");
                    let iface = wpa_supplicant::interface::InterfaceProxy::new(
                        &session.system_bus,
                        iface_path.clone(),
                    )
                    .await?;
                    trace!("Successfully created interface proxy");
                    let iface_name = iface.ifname().await?;
                    trace!("Interface name is {iface_name:?}");
                    let iface_state = iface.state().await?;
                    trace!("Interface state is {iface_state:?}");

                    let scope_id = unsafe {
                        libc::if_nametoindex(
                            std::ffi::CString::new(iface_name.clone()).unwrap().as_ptr(),
                        )
                    };
                    trace!("Interface scope id is {scope_id:?}");

                    let group_path = match props.get("group_object") {
                        Some(Value::ObjectPath(ref o)) => o.to_owned(),
                        other => {
                            error!("Expected a group object path, got {other:?}");
                            continue;
                        }
                    };

                    let group = wpa_supplicant::group::GroupProxy::new(
                        &session.system_bus,
                        group_path.clone(),
                    )
                    .await?;
                    let group_bssid = group.bssid().await?;
                    let Some(go_iface_addr) = utils::to_mac_addr(&group_bssid) else {
                        error!("Expected a valid mac address, got {group_bssid:?}");
                        continue;
                    };
                    trace!("Group BSSID (GO interface address) is {go_iface_addr:?}");

                    let go_dev_addr = group.go_device_address().await?;
                    let Some(go_dev_addr) = utils::to_mac_addr(&go_dev_addr) else {
                        error!("Expected a valid mac address, got {group_bssid:?}");
                        continue;
                    };
                    trace!("Group GO dev address is {go_dev_addr:?}");

                    let is_go = props.get("role") == Some(&Value::from("GO"));
                    let id = {
                        let mut groups = session.groups.write();
                        let data = DbusGroupData {
                            proxy: group,
                            iface,
                            iface_path: iface_path.into(),
                            path: group_path.into(),
                            go_dev_addr,
                        };
                        let handle = groups.insert(Group {
                            go_ip_address: IpAddr::V6(utils::mac_addr_to_local_link_address(
                                &go_iface_addr,
                            )),
                            iface_name,
                            scope_id,
                            is_go,
                            peers: Default::default(),
                            group_task: OnceLock::new(),
                            data,
                        });
                        let id = GroupId(handle);
                        groups.get_mut(handle).unwrap().group_task.get_or_init(|| {
                            let session = session.clone();
                            tokio::spawn(async move {
                                if let Err(e) = Session::group_task(session, id).await {
                                    error!("Group task for {id:?} failed with {e}");
                                    return Err(e);
                                }
                                Ok(())
                            })
                        });
                        id
                    };

                    session.listener.joined_group(&session, id, is_go);
                }
                Ok(())
            },
            async {
                while let Some(msg) = group_finished.next().await {
                    let args = msg.args()?;
                    let props = args.properties();
                    trace!("Group finished: {props:?}");

                    let group_path = match props.get("group_object") {
                        Some(Value::ObjectPath(ref o)) => o.to_owned(),
                        other => {
                            error!("Expected a group object path, got {other:?}");
                            continue;
                        }
                    };

                    let (group_id, is_go, peers_lost) = {
                        let mut groups = session.groups.write();
                        let id = match groups.id_by_path(&group_path) {
                            Some(id) => id,
                            None => {
                                error!("Got unknown group finished {group_path}");
                                continue;
                            }
                        };

                        let group = match groups.get_mut(id) {
                            Some(p) => p,
                            None => {
                                error!("Got unknown group finished {group_path}, {id:?}");
                                continue;
                            }
                        };

                        trace!("Group finished: {group:?}");
                        (GroupId(id), group.is_go, std::mem::take(&mut group.peers))
                    };

                    if !peers_lost.is_empty() {
                        let mut peers = session.peers.write();
                        for peer_id in peers_lost.keys() {
                            let peer = match peers.get_mut(peer_id.0) {
                                Some(g) => g,
                                None => {
                                    error!("Tried to disconnect peer {peer_id:?} from {group_id:?}, but didn't find group");
                                    continue;
                                }
                            };
                            // TODO: Use IndexSet or some more clever data structure? Or maybe just
                            // binary search.
                            let Some(index) = peer.groups.iter().position(|id| group_id == *id)
                            else {
                                error!("Tried to disconnect group {group_id:?} from {peer:?}, but didn't find peer in group peers");
                                continue;
                            };
                            peer.groups.remove(index);
                        }

                        for peer_id in peers_lost.keys() {
                            session
                                .listener
                                .peer_left_group(&session, group_id, *peer_id);
                        }
                    }

                    session.listener.left_group(&session, group_id, is_go);
                    let removed = session.groups.write().remove(group_id.0);
                    debug_assert!(removed.is_some(), "Found id but couldn't remove group?");
                }
                Ok(())
            },
            async {
                while let Some(msg) = group_formation_failure.next().await {
                    let args = msg.args()?;
                    let props = args.reason();
                    trace!("Group formation failure: {props:?}");
                }
                Ok(())
            },
            async {
                while let Some(msg) = go_negotiation_failure.next().await {
                    let args = msg.args()?;
                    let props = args.properties();
                    trace!("GO negotiation failed: {props:?}, retrying connection");

                    let peer_path = match props.get("peer_object") {
                        Some(Value::ObjectPath(o)) => o.to_owned(),
                        other => {
                            error!("Expected an peer object path, got {other:?}");
                            continue;
                        }
                    };

                    let session = Arc::clone(&session);
                    let peer = Value::from(peer_path);
                    let method = Value::from(WPS_METHOD);
                    let go_intent = Value::from(session.go_intent as i32);
                    let auto_join = Value::from(true);
                    tokio::spawn(async move {
                        let mut connect_args = HashMap::new();
                        connect_args.insert("peer", &peer);
                        connect_args.insert("wps_method", &method);
                        connect_args.insert("auto_join", &auto_join);
                        connect_args.insert("go_intent", &go_intent);
                        session.p2pdevice.connect(connect_args).await
                    });
                }
                Ok(())
            },
            async {
                while let Some(msg) = go_negotiation_request.next().await {
                    let args = msg.args()?;
                    let path = args.path();
                    let passwd_id = args.dev_passwd_id();
                    let go_intent = args.device_go_intent();
                    trace!("GO negotiation request from {path} ({passwd_id} / {go_intent})");
                    // Let's try to connect to the peer directly.
                    // TODO(emilio): Maybe confirm?
                    session
                        .connect_to_peer_by_path(path.to_owned().into())
                        .await?;
                }
                Ok(())
            },
            async {
                while let Some(msg) = go_negotiation_success.next().await {
                    let args = msg.args()?;
                    let props = args.properties();
                    trace!("GO negotiation succeeded: {props:?}");
                }
                Ok(())
            },
            async {
                while let Some(msg) = peers_changed.next().await {
                    let value = msg.get().await?;
                    trace!("Peers changed: {value:?}");
                }
                Ok(())
            },
            async {
                while let Some(msg) = persistent_group_added.next().await {
                    let args = msg.args()?;
                    let path = args.path();
                    let props = args.properties();
                    trace!("Persistent Group added ({path}): {props:?}");
                }
                Ok(())
            },
            async {
                while let Some(msg) = persistent_group_removed.next().await {
                    let args = msg.args()?;
                    let path = args.path();
                    trace!("Persistent Group removed: {path:?}");
                }
                Ok(())
            },
            async {
                while let Some(msg) = persistent_groups_changed.next().await {
                    let value = msg.get().await?;
                    trace!("Persistent Groups changed: {value:?}");
                }
                Ok(())
            },
            async {
                while let Some(msg) = pd_failure.next().await {
                    let args = msg.args()?;
                    let peer_object = args.peer_object();
                    let status = args.status();
                    trace!("Provision Discovery failure: {peer_object} ({status})");
                }
                Ok(())
            },
            async {
                while let Some(msg) = pd_req_display_pin.next().await {
                    let args = msg.args()?;
                    let peer_object = args.peer_object();
                    let pin = args.pin();
                    trace!("PD Request display pin: {peer_object} ({pin})");
                }
                Ok(())
            },
            async {
                while let Some(msg) = pd_rsp_display_pin.next().await {
                    let args = msg.args()?;
                    let peer_object = args.peer_object();
                    let pin = args.pin();
                    trace!("PD Response display pin: {peer_object} ({pin})");
                }
                Ok(())
            },
            async {
                while let Some(msg) = pd_req_enter_pin.next().await {
                    let args = msg.args()?;
                    let peer_object = args.peer_object();
                    trace!("PD Request enter pin: {peer_object}");
                }
                Ok(())
            },
            async {
                while let Some(msg) = pd_rsp_enter_pin.next().await {
                    let args = msg.args()?;
                    let peer_object = args.peer_object();
                    trace!("PD Response enter pin: {peer_object}");
                }
                Ok(())
            },
            async {
                while let Some(msg) = pd_pbc_req.next().await {
                    let args = msg.args()?;
                    let peer_object = args.peer_object();
                    trace!("PD PBC Request: {peer_object}, trying to authorize");

                    let peer_dev_addr = {
                        let peers = session.peers.read();
                        let Some(peer) = peers.get_by_path(peer_object) else {
                            error!("Can't found {peer_object} in peers map");
                            continue;
                        };
                        peer.identity.physical.dev_addr
                    };

                    let go_groups = {
                        let mut go_groups = vec![];
                        let groups = session.groups.read();
                        for group in groups.iter() {
                            go_groups.push(group.data.iface_path.clone());
                        }
                        go_groups
                    };

                    for group_iface_path in go_groups {
                        let Ok(wps) = wpa_supplicant::wps::WPSProxy::new(
                            &session.system_bus,
                            &group_iface_path,
                        )
                        .await
                        else {
                            error!(
                                "Couldn't find WPS interface for group iface {:?}",
                                group_iface_path
                            );
                            continue;
                        };
                        let mut params = HashMap::new();
                        let dev_addr = Value::from(peer_dev_addr.as_bytes());
                        let role = Value::from("registrar");
                        let ty = Value::from(WPS_METHOD);
                        params.insert("Role", &role);
                        params.insert("P2PDeviceAddress", &dev_addr);
                        params.insert("Type", &ty);
                        if let Err(e) = wps.start(params).await {
                            error!("Can't start wps authorization for {group_iface_path}: {e}");
                            continue;
                        }
                    }
                }
                Ok(())
            },
            async {
                while let Some(msg) = pd_pbc_rsp.next().await {
                    let args = msg.args()?;
                    let peer_object = args.peer_object();
                    trace!("PD PBC Response: {peer_object}");
                }
                Ok(())
            },
        )?;

        Ok(())
    }

    async fn connect_to_peer_by_path(&self, peer_path: OwnedObjectPath) -> Result<(), zbus::Error> {
        let mut args = HashMap::default();
        let method = Value::from(WPS_METHOD);
        let go_intent = Value::from(self.go_intent as i32);
        let auto_join = Value::from(true);
        let peer_path = Value::from(peer_path);
        args.insert("peer", &peer_path);
        args.insert("auto_join", &auto_join);
        args.insert("wps_method", &method);
        args.insert("go_intent", &go_intent);
        match self.p2pdevice.connect(args).await {
            Ok(pin) => trace!("Connected with pin: {pin}"),
            Err(e) => {
                error!("Failed to connect to peer: {e:?}");
                return Err(e);
            }
        }
        Ok(())
    }
}
