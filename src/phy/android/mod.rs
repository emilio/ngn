//! JNI implementation of the physical layer, interfacing with Android's WifiP2PManager.
//!
//! Known limitation: Only current method of provisioning the GO IP address is via IPV6 link-local
//! addresses. This means that if you use something like private address assignment via something
//! like `slaac private`, it will fail to connect to the GO. Other neighbor discovery approaches
//! could be used in the future.

use super::{GenericResult, GroupId, P2PSession, P2PSessionListener, PeerId};
use handy::HandleMap;
use jni::{
    objects::{GlobalRef, JClass, JObject, JObjectArray, JString},
    JNIEnv, JavaVM,
};
use jni_sys::jlong;

use crate::{
    phy::protocol::{
        self, ControlMessage, P2pPorts, PeerAddress, PeerGroupInfo, PeerIdentity, PeerInfo,
        PeerOwnIdentifier, GO_CONTROL_PORT,
    },
    utils::{self, trivial_error},
};
use macaddr::MacAddr;

use log::{error, trace, warn};
use parking_lot::RwLock;
use std::{
    collections::hash_map::Entry,
    net::{IpAddr, Ipv6Addr, SocketAddr, SocketAddrV6},
    str::FromStr,
    sync::{Arc, OnceLock},
    time::Duration,
};
use tokio::{
    self, io,
    net::{TcpListener, TcpStream},
};
use tokio::{sync::mpsc, task::JoinHandle};

// const WPS_METHOD: &'static str = "pbc";

// TODO: Probably should allow to get an external runtime or so.
fn rt() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
    })
}

/// Representation of system messages that we need to handle
#[derive(Debug)]
enum JavaNotification {
    FindStopped,
    DeviceFound(PeerIdentity),
    DeviceLost(PeerIdentity),
    // InvitationReceived,
    // InvitationResult,
    // WpsFailed,
    GroupStarted {
        go_iface_addr: MacAddr,
        iface_name: String,
        is_go: bool,
    },
    // TODO: We're going to need some sort of identifier here.
    // GroupFinished(..),
    // GroupFormationFailure,
    // GoNegotiationRequest(..),
    // GoNegotiationFailure(..),
    // PersistentGroupAddeed(..),
    // PersistentGroupRemoved(..),
    // PersistentGroupsChanged(..),
    // ProvisionDiscoveryFailure(..),
    // ProvisionDiscoveryRequestDisplayPin(..),
    // ProvisionDiscoveryResponseDisplayPin(..),
    // ProvisionDiscoveryRequestEnterPin(..),
    // ProvisionDiscoveryPbcRequest(..),
    // ProvisionDiscoveryPbcResponse(..),
    // PeersChanged(..),
    // PersistentGroupsChanged(..),
}

#[derive(Debug)]
struct AndroidPeerData;
type Peer = protocol::PeerInfo<AndroidPeerData>;

#[derive(Debug)]
struct AndroidGroupData;
type Group = protocol::GroupInfo<AndroidGroupData>;

/// Global state for a P2P session.
#[derive(Debug)]
pub struct Session {
    vm: JavaVM,
    proxy: GlobalRef,
    go_intent: u32,
    peers: RwLock<HandleMap<Peer>>,
    groups: RwLock<HandleMap<Group>>,
    listener: Arc<dyn P2PSessionListener<Self>>,
    java_notification: mpsc::UnboundedSender<JavaNotification>,
    /// Task handle to our run loop. Canceled and awaited on drop.
    run_loop_task: RwLock<Option<JoinHandle<GenericResult<()>>>>,
    /// The name we expose to our P2P peers. We store it instead of the device address because the
    /// P2P device address is not exposed to non-privileged apps.
    name: String,
}

impl Drop for Session {
    fn drop(&mut self) {
        if let Some(ref t) = *self.run_loop_task.read() {
            t.abort();
        }
    }
}

pub struct SessionInit<'a> {
    /// JNI VM to be able to do java calls. TODO(emilio): JNI usage can most definitely be
    /// optimized.
    pub vm: JavaVM,
    /// Proxy object. Must be an NgnSessionProxy java object.
    pub proxy: GlobalRef,
    /// Name for our P2P operations.
    pub p2p_name: String,
    /// Our group owner intent, from 0 to 15.
    pub go_intent: u32,
    pub _phantom: std::marker::PhantomData<&'a ()>,
}

async fn send_message_to(addr: &SocketAddrV6, message: &[u8]) -> GenericResult<()> {
    trace!("send_message_to({addr:?}, {})", message.len());
    let mut stream =
        match tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(addr)).await? {
            Ok(stream) => stream,
            Err(e) => return Err(io::Error::new(io::ErrorKind::TimedOut, e).into()),
        };
    super::protocol::write_binary_message(&mut stream, message).await?;
    Ok(())
}

#[async_trait::async_trait]
impl P2PSession for Session {
    type InitArgs<'a> = SessionInit<'a>;

    /// Create a new P2P session.
    async fn new(
        init: SessionInit<'_>,
        listener: Arc<dyn P2PSessionListener<Self>>,
    ) -> GenericResult<Arc<Self>> {
        Ok(Self::new_sync(init, listener))
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
        let (tx, rx) = tokio::sync::oneshot::channel();
        {
            let mut env = self.vm.attach_current_thread()?;
            let tx_long = Box::leak(Box::new(tx)) as *mut _ as jlong;
            env.call_method(
                self.proxy.as_obj(),
                "(J)V",
                "discoverPeers",
                &[tx_long.into()],
            )?;
        }
        rx.await?
    }

    fn peer_name(&self, id: PeerId) -> Option<String> {
        Some(self.peers.read().get(id.0)?.identity.name.to_owned())
    }

    async fn connect_to_peer(&self, id: PeerId) -> GenericResult<()> {
        trace!("Session::connect_to_peer({id:?})");
        let device_address = {
            let peers = self.peers.read();
            let Some(peer) = peers.get(id.0) else {
                return Err(trivial_error!("Couldn't find peer id"));
            };
            peer.identity.dev_addr.clone()
        };
        let (tx, rx) = tokio::sync::oneshot::channel();
        {
            let mut env = self.vm.attach_current_thread()?;
            let tx_long = Box::leak(Box::new(tx)) as *mut _ as jlong;
            let peer_address = env.new_string(device_address.to_string())?;
            env.call_method(
                self.proxy.as_obj(),
                "(Ljava/lang/String;J)V",
                "connectToPeer",
                &[(&peer_address).into(), tx_long.into()],
            )?;
        }
        rx.await?
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
            let Some(address) = group.peers.get(&id).and_then(|info| info.address.clone()) else {
                return Err(trivial_error!(
                    "Peer doesn't have a link local address (yet?)"
                ));
            };
            (address, group.scope_id)
        };
        let socket_addr = SocketAddrV6::new(
            peer_address.link_local_address,
            peer_address.ports.p2p,
            /* flowinfo = */ 0,
            scope_id,
        );
        utils::retry_timeout(Duration::from_secs(2), 5, || {
            send_message_to(&socket_addr, message)
        })
        .await
    }
}

impl Session {
    fn own_identity(&self) -> PeerOwnIdentifier {
        PeerOwnIdentifier::Name(self.name.clone())
    }

    fn new_sync(init: SessionInit<'_>, listener: Arc<dyn P2PSessionListener<Self>>) -> Arc<Self> {
        let (tx, rx) = mpsc::unbounded_channel();
        let session = Arc::new(Self {
            peers: Default::default(),
            groups: Default::default(),
            java_notification: tx,
            vm: init.vm,
            proxy: init.proxy,
            go_intent: init.go_intent,
            listener,
            name: init.p2p_name,
            run_loop_task: RwLock::new(None),
        });

        let handle = rt().spawn(Session::run_loop(Arc::clone(&session), rx));
        *session.run_loop_task.write() = Some(handle);

        session
    }

    fn peer_id_from_address(&self, group_id: GroupId, address: &SocketAddr) -> Option<PeerId> {
        let address = match address {
            SocketAddr::V4(ref addr) => {
                error!("Unexpected message from ipv4 address {addr:?}");
                return None;
            }
            SocketAddr::V6(addr) => addr,
        };

        // TODO: This lookup could be faster, really.
        let groups = self.groups.read();
        let Some(group) = groups.get(group_id.0) else {
            error!("Group {group_id:?} lost?");
            return None;
        };
        for (peer_id, info) in &group.peers {
            if info
                .address
                .as_ref()
                .is_some_and(|addr| addr.link_local_address == *address.ip())
            {
                return Some(*peer_id);
            }
        }
        error!("Address {address:?} couldn't be mapped to a known peer in the group!");
        return None;
    }

    async fn send_control_message(
        &self,
        link_local_address: Ipv6Addr,
        control_port: u16,
        scope_id: u32,
        message: ControlMessage,
    ) -> GenericResult<()> {
        trace!("Session::send_control_message({link_local_address:?}, {scope_id}, {message:?})");
        let local_addr = SocketAddrV6::new(
            link_local_address,
            control_port,
            /* flowinfo = */ 0,
            scope_id,
        );
        let msg = bincode::encode_to_vec(message, bincode::config::standard())?;
        utils::retry_timeout(Duration::from_secs(2), 5, || {
            send_message_to(&local_addr, &msg)
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
                loop {
                    let buf = match super::protocol::read_binary_message(&mut stream).await {
                        Ok(buf) => buf,
                        Err(e) => {
                            // XXX EOF isn't really unexpected.
                            error!("Unexpected error reading message from {address:?}: {e:?}");
                            return;
                        }
                    };
                    let Ok((control_message, len)) = bincode::decode_from_slice::<ControlMessage, _>(
                        &buf,
                        bincode::config::standard(),
                    ) else {
                        error!("Failed to decode binary control message {buf:?}");
                        return;
                    };
                    if len != buf.len() {
                        error!("Unexpected decoded message length {} vs {}", len, buf.len());
                        return;
                    }
                    trace!("Got control message {control_message:?} on group {group_id:?}");
                    match control_message {
                        ControlMessage::Associate { id, ports } => {
                            let Some(peer_id) = session.find_peer_id_by_own_addr(&id) else {
                                error!("Couldn't associate with {id:?}");
                                continue;
                            };
                            let link_local_address = match address.ip() {
                                IpAddr::V4(v4addr) => {
                                    warn!("Unexpected ipv4 address {v4addr:?} in control message");
                                    continue;
                                }
                                IpAddr::V6(v6addr) => v6addr.clone(),
                            };

                            let is_new_connection = {
                                let mut groups = session.groups.write();
                                let Some(group) = groups.get_mut(group_id.0) else {
                                    warn!("Group {group_id:?} was torn down?");
                                    continue;
                                };
                                let address = PeerAddress {
                                    link_local_address,
                                    ports,
                                };
                                match group.peers.entry(peer_id) {
                                    Entry::Occupied(mut o) => {
                                        let info = o.get_mut();
                                        if info.address.as_ref().is_some_and(|a| *a != address) {
                                            error!("Forbidding re-association of {address:?}");
                                            continue;
                                        }
                                        o.get_mut().address = Some(address);
                                        false
                                    }
                                    Entry::Vacant(v) => {
                                        v.insert(PeerGroupInfo {
                                            address: Some(address),
                                        });
                                        true
                                    }
                                }
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
                                        link_local_address,
                                        ports.control,
                                        scope_id,
                                        ControlMessage::Associate {
                                            id: session.own_identity(),
                                            ports: own_ports,
                                        },
                                    )
                                    .await;
                                if let Err(e) = result {
                                    error!("Failed to send associate message from GO to {peer_id:?}: {e}");
                                }
                            }
                            if is_new_connection {
                                trace!(
                                    "Notifying of new association of {peer_id:?} to {group_id:?}"
                                );
                                session
                                    .listener
                                    .peer_joined_group(&session, group_id, peer_id);
                            }
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
            let session = Arc::clone(&session);
            tokio::spawn(async move {
                trace!("Incoming connection from {address:?}");
                loop {
                    let buf = match super::protocol::read_binary_message(&mut stream).await {
                        Ok(buf) => buf,
                        Err(e) => {
                            // XXX EOF isn't really unexpected.
                            error!("Unexpected error reading message from {address:?}: {e:?}");
                            return;
                        }
                    };
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

        let (is_go, go_iface_addr, scope_id) = match session.groups.read().get(group_id.0) {
            Some(g) => (g.is_go, g.go_iface_addr, g.scope_id),
            None => {
                error!("Didn't find {group_id:?} on group_task start!");
                return Err(trivial_error!("Didn't find group on group_task start!"));
            }
        };

        let go_local_addr = utils::mac_addr_to_local_link_address(&go_iface_addr);
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

        trace!(" > go = {is_go}, ports = {my_ports:?}, go_local_addr = {go_local_addr:?}");
        if !is_go {
            let control_message = ControlMessage::Associate {
                id: session.own_identity(),
                ports: my_ports,
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
                    is_go
                ),
                async move {
                    trace!("Trying to send control message to {go_local_addr:?}");
                    session
                        .send_control_message(
                            go_local_addr,
                            GO_CONTROL_PORT,
                            scope_id,
                            control_message,
                        )
                        .await
                },
            )?;
            return Ok(());
        }

        // TODO: peer_joined / peer_left? Also factor out with dbus code.
        tokio::try_join!(
            Self::listen_to_peer_messages(Arc::clone(&session), p2p_listener, group_id, scope_id),
            Self::establish_control_channel(
                Arc::clone(&session),
                control_listener,
                group_id,
                scope_id,
                my_ports,
                is_go
            ),
        )?;
        Ok(())
    }

    async fn run_loop(
        session: Arc<Self>,
        mut rx: mpsc::UnboundedReceiver<JavaNotification>,
    ) -> GenericResult<()> {
        trace!("Session::run_loop");
        while let Some(message) = rx.recv().await {
            trace!("run_loop: {:?}", message);
            match message {
                JavaNotification::FindStopped => {
                    session.listener.peer_discovery_stopped(&session);
                }
                JavaNotification::DeviceFound(identity) => {
                    let id = {
                        let mut peers = session.peers.write();
                        if let Some(id) =
                            Self::find_peer_id_by_dev_addr_in_map(&peers, &identity.dev_addr)
                        {
                            let existing = peers.get_mut(id.0).expect("DBUS store out of sync");
                            trace!("Peer was already registered (from previous scan?) with identity {:?}", existing.identity);
                            // TODO(emilio): Consider not notifying? Kinda puts the burden of
                            // preserving peer list to the parent.
                            id
                        } else {
                            PeerId(peers.insert(Peer {
                                identity,
                                groups: Vec::new(),
                                data: AndroidPeerData,
                            }))
                        }
                    };
                    session.listener.peer_discovered(&session, id);
                }
                JavaNotification::DeviceLost(identity) => {
                    let (peer_id, groups_disconnected) = {
                        let mut peers = session.peers.write();
                        let Some(id) =
                            Self::find_peer_id_by_dev_addr_in_map(&peers, &identity.dev_addr)
                        else {
                            error!("Got unknown device lost {identity:?}");
                            continue;
                        };

                        let Some(peer) = peers.get_mut(id.0) else {
                            error!("Got unknown device lost {identity:?}, {id:?}");
                            continue;
                        };

                        trace!("Peer lost: {peer:?}");
                        (id, std::mem::take(&mut peer.groups))
                    };

                    // Remove the peer for any outstanding groups before notifying the
                    // listener of the peer being lost.
                    if !groups_disconnected.is_empty() {
                        let mut groups = session.groups.write();
                        for group_id in &groups_disconnected {
                            let Some(group) = groups.get_mut(group_id.0) else {
                                error!("Tried to disconnect peer {peer_id:?} from {group_id:?}, but didn't find group");
                                continue;
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
                JavaNotification::GroupStarted {
                    go_iface_addr,
                    iface_name,
                    is_go,
                } => {
                    let scope_id = unsafe {
                        libc::if_nametoindex(
                            std::ffi::CString::new(iface_name.clone()).unwrap().as_ptr(),
                        )
                    };
                    trace!("Interface scope id is {scope_id:?}");
                    let id = {
                        let mut groups = session.groups.write();
                        let handle = groups.insert(Group {
                            go_iface_addr,
                            iface_name,
                            scope_id,
                            is_go,
                            peers: Default::default(),
                            group_task: OnceLock::new(),
                            data: AndroidGroupData,
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
            }
        }
        Ok(())
    }

    fn find_peer_id_by_dev_addr_in_map(
        peer_map: &HandleMap<Peer>,
        dev_addr: &MacAddr,
    ) -> Option<PeerId> {
        let own_id = PeerOwnIdentifier::DevAddr(dev_addr.clone().into());
        Self::find_peer_id_by_own_addr_in_map(peer_map, &own_id)
    }

    fn find_peer_id_by_own_addr_in_map(
        peer_map: &HandleMap<Peer>,
        own_id: &PeerOwnIdentifier,
    ) -> Option<PeerId> {
        trace!("find_peer_id_by_own_addr({own_id:?})");
        for (id, peer) in peer_map.iter_with_handles() {
            trace!(" {:?} -> {:?}", id, peer.identity);
            if peer.identity.matches(own_id) {
                return Some(PeerId(id));
            }
        }
        None
    }

    fn find_peer_id_by_own_addr(&self, own_id: &PeerOwnIdentifier) -> Option<PeerId> {
        Self::find_peer_id_by_own_addr_in_map(&self.peers.read(), own_id)
    }

    #[export_name = "Java_io_crisal_ngn_NgnSessionProxy_ngn_1session_1init"]
    extern "C" fn init<'l>(
        mut env: JNIEnv<'l>,
        _class: JClass<'l>,
        owner: JObject<'l>,
        name: JString<'l>,
    ) -> jlong {
        let name = env.get_string(&name).unwrap();
        let name = name.to_string_lossy();
        trace!("Session::init({name:?})");
        let init = SessionInit {
            vm: env.get_java_vm().unwrap(),
            proxy: env.new_global_ref(owner).unwrap(),
            p2p_name: name.into(),
            go_intent: 14,
            _phantom: std::marker::PhantomData,
        };

        // TODO(emilio): Need a listener that proxies to `NgnSessionProxy`.
        let session = Self::new_sync(init, Arc::new(super::LoggerListener::default()));
        Arc::into_raw(session) as jlong
    }

    // Breaks the cyclic owner <-> native listener.
    #[export_name = "Java_io_crisal_ngn_NgnSessionProxy_ngn_1session_1drop"]
    extern "C" fn drop<'l>(_env: JNIEnv<'l>, _class: JClass<'l>, raw: jlong) {
        trace!("Session::drop({raw:?})");
        let _ = unsafe { Arc::from_raw(raw as *const Self) };
    }

    // syncs the peer list. Expected an array of Strings.
    #[export_name = "Java_io_crisal_ngn_NgnSessionProxy_ngn_1session_1update_1peers"]
    extern "C" fn update_peers<'l>(
        mut env: JNIEnv<'l>,
        _class: JClass<'l>,
        raw: jlong,
        details: JObjectArray<'l>,
    ) {
        const STEP: usize = 2;
        trace!("Session::update_peers({raw:?})");
        let _session = unsafe { &*(raw as *const Self) };
        let len = env.get_array_length(&details).unwrap();
        assert!(len as usize % STEP == 0, "Should have the right step");
        let mut identities = Vec::<PeerIdentity>::with_capacity(len as usize / STEP);
        let mut get_string = |i| {
            let string = env.get_object_array_element(&details, i).unwrap();
            let string = unsafe { JString::from_raw(string.as_raw()) };
            let string = env.get_string(&string).unwrap();
            string.to_string_lossy().into_owned()
        };
        for i in (0..len).step_by(STEP) {
            let name = get_string(i);
            let dev_addr = MacAddr::from_str(&get_string(i + 1)).unwrap();
            identities.push(PeerIdentity {
                name,
                dev_addr,
            });
        }
        trace!(" > identities: {:?}", identities);
    }
}

#[export_name = "Java_io_crisal_ngn_NgnSessionProxy_ngn_1init"]
extern "C" fn ngn_init<'l>(_: JNIEnv<'l>) {
    trace!("ngn_init()\n");
    // Initialize our logging and panic hooks.
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Trace),
    );
    log_panics::init();
}
