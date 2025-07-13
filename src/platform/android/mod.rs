//! JNI implementation of the physical layer, interfacing with Android's WifiP2PManager.
//!
//! Known limitation: Only current method of provisioning the GO IP address is via IPV6 link-local
//! addresses. This means that if you use something like private address assignment via something
//! like `slaac private`, it will fail to connect to the GO. Other neighbor discovery approaches
//! could be used in the future.

use handy::HandleMap;
use jni::{
    objects::{GlobalRef, JByteArray, JClass, JObject, JObjectArray, JString},
    JNIEnv, JavaVM,
};
use jni_sys::{jboolean, jlong};

use crate::{
    protocol::{
        self, identity::OwnIdentity, ControlMessage, P2pPorts, PeerAddress, PeerGroupInfo,
        PeerIdentity, PeerOwnIdentifier, PhysiscalPeerIdentity, GO_CONTROL_PORT,
    },
    utils::{self, trivial_error},
    GenericResult, GroupId, P2PSession, P2PSessionListener, PeerId,
};
use macaddr::MacAddr;

use log::{error, trace, warn};
use parking_lot::RwLock;
use std::{
    collections::{HashMap, HashSet},
    net::{IpAddr, Ipv6Addr, SocketAddr, SocketAddrV6},
    ptr,
    str::FromStr,
    sync::{Arc, OnceLock},
    time::Duration,
};
use tokio::{self, net::TcpListener};
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
    // FindStopped,
    UpdateDevices(Vec<PhysiscalPeerIdentity>),
    // InvitationReceived,
    // InvitationResult,
    // WpsFailed,
    GroupStarted {
        iface_name: String,
        is_go: bool,
        go_ip_address: IpAddr,
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

#[derive(Debug, Default)]
struct PeerStore {
    map: HandleMap<Peer>,
    /// This is only used to speed up device updates, we could also track name to id.
    mac_to_id: HashMap<MacAddr, PeerId>,
}

impl PeerStore {
    fn clear(&mut self) {
        self.map.clear();
        self.mac_to_id.clear();
    }
}

fn peer_identity_to_jni<'local>(
    env: &mut JNIEnv<'local>,
    id: &PeerIdentity,
) -> GenericResult<(JString<'local>, JString<'local>, JString<'local>)> {
    Ok((
        env.new_string(&id.physical.name)?,
        env.new_string(id.physical.dev_addr.to_string())?,
        if let Some(ref id) = id.logical {
            env.new_string(id.to_string())?
        } else {
            unsafe { JString::from_raw(ptr::null_mut()) }
        },
    ))
}

/// Global state for a P2P session.
#[derive(Debug)]
pub struct Session {
    vm: JavaVM,
    proxy: GlobalRef,
    peers: RwLock<PeerStore>,
    groups: RwLock<HandleMap<Group>>,
    listener: Arc<dyn P2PSessionListener<Self>>,
    identity: OwnIdentity,
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
    /// Identity for message signing and verification.
    pub identity: OwnIdentity,
    pub _phantom: std::marker::PhantomData<&'a ()>,
}

macro_rules! try_void {
    ($expr:expr, $msg:literal) => {
        match $expr {
            Ok(r) => r,
            Err(e) => {
                error!("{}: {e}", $msg);
                return;
            }
        }
    };
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
            let tx_long = Box::leak(Box::new(tx)) as *mut _ as jlong;
            let mut env = self.vm.attach_current_thread()?;
            self.call_proxy(&mut env, "(J)V", "discoverPeers", &[tx_long.into()])?;
        }
        rx.await?
    }

    fn peer_identity(&self, id: PeerId) -> Option<PeerIdentity> {
        Some(self.peers.read().map.get(id.0)?.identity.clone())
    }

    fn all_peers(&self) -> Vec<(PeerId, PeerIdentity)> {
        self.peers
            .read()
            .map
            .iter_with_handles()
            .map(|(id, info)| (PeerId(id), info.identity.clone()))
            .collect()
    }

    fn own_identity(&self) -> &OwnIdentity {
        &self.identity
    }

    async fn connect_to_peer(&self, id: PeerId) -> GenericResult<()> {
        trace!("Session::connect_to_peer({id:?})");
        let device_address = {
            let peers = self.peers.read();
            let Some(peer) = peers.map.get(id.0) else {
                return Err(trivial_error!("Couldn't find peer id"));
            };
            peer.identity.physical.dev_addr
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
            let Some(peer) = peers.map.get(id.0) else {
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
            let Some(address) = group.peers.get(&id).map(|info| info.address.clone()) else {
                return Err(trivial_error!(
                    "Peer doesn't have a link local address (yet?)"
                ));
            };
            (address, group.scope_id)
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
    fn own_physical_id(&self) -> PeerOwnIdentifier {
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
            identity: init.identity,
            listener,
            name: init.p2p_name,
            run_loop_task: RwLock::new(None),
        });

        let handle = rt().spawn(Session::run_loop(Arc::clone(&session), rx));
        *session.run_loop_task.write() = Some(handle);

        session
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
        address: IpAddr,
        control_port: u16,
        scope_id: u32,
        message: ControlMessage,
    ) -> GenericResult<()> {
        trace!("Session::send_control_message({address:?}, {scope_id}, {message:?})");
        let socket_addr = protocol::peer_to_socket_addr(address, scope_id, control_port);
        let msg = bincode::encode_to_vec(message, bincode::config::standard())?;
        utils::retry_timeout(Duration::from_secs(2), 5, || {
            protocol::send_message(None, &socket_addr, &msg)
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
                            ports,
                        } => {
                            let peer_id = {
                                let mut peers = session.peers.write();
                                let mut result = None;
                                for (id, peer) in peers.map.iter_mut_with_handles() {
                                    trace!(" {:?} -> {:?}", id, peer.identity.logical);
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
                                        result = Some(PeerId(id));
                                        break;
                                    }
                                }
                                match result {
                                    Some(id) => id,
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
                                            physical_id: session.own_physical_id(),
                                            logical_id: session.identity.to_public(),
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
                            session.peers_changed();
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
                .map
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
                    session.peer_messaged(peer_id, group_id, &buf);
                }
            });
        }
        #[allow(unreachable_code)]
        Ok(())
    }

    async fn group_task(session: Arc<Self>, group_id: GroupId) -> GenericResult<()> {
        trace!("Session::group_task({group_id:?})");

        let (is_go, go_ip, scope_id) = match session.groups.read().get(group_id.0) {
            Some(g) => (g.is_go, g.go_ip_address, g.scope_id),
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
            let control_message = ControlMessage::Associate {
                physical_id: session.own_physical_id(),
                logical_id: session.identity.to_public(),
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
                    trace!("Trying to send control message to {go_ip:?}");
                    session
                        .send_control_message(go_ip, GO_CONTROL_PORT, scope_id, control_message)
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
                JavaNotification::UpdateDevices(identities) => {
                    let mut peers_joined = vec![];
                    let mut peers_lost = vec![];
                    {
                        let mut peers = session.peers.write();
                        let peers = &mut *peers;
                        let mut seen_ids = HashSet::new();
                        for identity in identities {
                            let dev_addr = identity.dev_addr;
                            seen_ids.insert(dev_addr);
                            let id = peers.mac_to_id.get(&dev_addr).copied();
                            if let Some(id) = id {
                                trace!("Peer was already registered (from previous scan?) with identity {:?}", peers.map.get(id.0).map(|p| &p.identity));
                                continue;
                            }
                            let id = PeerId(peers.map.insert(Peer {
                                identity: PeerIdentity {
                                    physical: identity,
                                    logical: None,
                                },
                                groups: Vec::new(),
                                data: AndroidPeerData,
                            }));
                            peers.mac_to_id.insert(dev_addr, id);
                            peers_joined.push(id);
                        }
                        if seen_ids.len() != peers.mac_to_id.len() {
                            // Some device has been lost.
                            peers.mac_to_id.retain(|mac, id| {
                                if seen_ids.contains(mac) {
                                    return true;
                                }
                                let Some(peer) = peers.map.get_mut(id.0) else {
                                    error!("Store out of sync for {mac:?}, {id:?}!");
                                    return false;
                                };
                                trace!("Peer lost: {peer:?}");
                                peers_lost.push((*id, std::mem::take(&mut peer.groups)));
                                false
                            });
                        }
                    }
                    let changed = !peers_lost.is_empty() || !peers_joined.is_empty();
                    for (peer_id, groups_disconnected) in peers_lost {
                        // Remove the peer for any outstanding groups before notifying the listener
                        // of the peer being lost.
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
                        let removed = session.peers.write().map.remove(peer_id.0);
                        debug_assert!(removed.is_some(), "Found id but couldn't remove peer?");
                    }
                    for id in peers_joined {
                        session.listener.peer_discovered(&session, id);
                    }
                    if changed {
                        session.peers_changed();
                    }
                }
                JavaNotification::GroupStarted {
                    iface_name,
                    is_go,
                    go_ip_address,
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
                            go_ip_address,
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

    #[export_name = "Java_io_crisal_ngn_NgnSessionProxy_ngn_1session_1init"]
    extern "C" fn init<'l>(
        mut env: JNIEnv<'l>,
        _class: JClass<'l>,
        owner: JObject<'l>,
        device_name: JString<'l>,
        nickname: JString<'l>,
    ) -> jlong {
        let device_name = env.get_string(&device_name).unwrap();
        let device_name = device_name.to_string_lossy();
        let nickname = env.get_string(&nickname).unwrap();
        let nickname = nickname.to_string_lossy();
        trace!("Session::init({device_name:?}, {nickname:?})");

        // TODO(emilio): Get keys from caller.
        let identity = protocol::identity::new_own_id(nickname.into_owned()).unwrap();

        let init = SessionInit {
            vm: env.get_java_vm().unwrap(),
            proxy: env.new_global_ref(owner).unwrap(),
            p2p_name: device_name.into(),
            identity,
            _phantom: std::marker::PhantomData,
        };

        let session = Self::new_sync(init, Arc::new(crate::LoggerListener));
        Arc::into_raw(session) as jlong
    }

    fn call_proxy<'local>(
        &self,
        env: &mut JNIEnv<'local>,
        sig: &'static str,
        method: &'static str,
        args: &[jni::objects::JValue],
    ) -> GenericResult<jni::objects::JValueOwned<'local>> {
        let result = env.call_method(self.proxy.as_obj(), method, sig, args)?;
        Ok(result)
    }

    /// Resolves a java on-result function with a given boolean value.
    fn resolve_result(&self, on_result: &JObject<'_>, success: bool) -> GenericResult<()> {
        if on_result.is_null() {
            return Ok(());
        }
        let mut env = self.vm.attach_current_thread()?;
        self.call_proxy(
            &mut env,
            "(Ljava/lang/Object;Z)V",
            "resolveResult",
            &[on_result.into(), success.into()],
        )?;
        Ok(())
    }

    fn peers_changed(&self) {
        if let Err(e) = self.peers_changed_internal() {
            error!("Failed to broadcast peer changes to java: {e}");
        }
    }

    fn peers_changed_internal(&self) -> GenericResult<()> {
        const ENTRIES: i32 = 3;
        let mut env = self.vm.attach_current_thread()?;
        let peers = self.all_peers();
        let arr =
            env.new_object_array(peers.len() as i32 * ENTRIES, "java/lang/String", unsafe {
                JString::from_raw(ptr::null_mut())
            })?;
        let mut i = 0;
        for (_id, id) in peers {
            let (peer_name, peer_dev_addr, peer_logical_id) = peer_identity_to_jni(&mut env, &id)?;
            env.set_object_array_element(&arr, i, peer_name)?;
            env.set_object_array_element(&arr, i + 1, peer_dev_addr)?;
            env.set_object_array_element(&arr, i + 2, peer_logical_id)?;
            i += ENTRIES;
        }
        self.call_proxy(
            &mut env,
            "([Ljava/lang/String;)V",
            "notifyPeersChanged",
            &[(&arr).into()],
        )?;
        Ok(())
    }

    fn peer_messaged(&self, peer_id: PeerId, group_id: GroupId, buf: &[u8]) {
        if let Err(e) = self.peer_messaged_internal(peer_id, group_id, buf) {
            error!("Failed to broadcast peer message to java: {e}");
        }
    }

    fn peer_messaged_internal(
        &self,
        peer_id: PeerId,
        group_id: GroupId,
        buf: &[u8],
    ) -> GenericResult<()> {
        self.listener.peer_messaged(self, peer_id, group_id, buf);
        let mut env = self.vm.attach_current_thread()?;
        let (peer_name, peer_dev_addr, peer_logical_id) = {
            let peers = self.peers.read();
            let Some(peer) = peers.map.get(peer_id.0) else {
                return Err(trivial_error!("peer_messaged from gone peer"));
            };
            peer_identity_to_jni(&mut env, &peer.identity)?
        };
        let byte_array = env.byte_array_from_slice(buf)?;
        self.call_proxy(
            &mut env,
            "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;[B)V",
            "peerMessaged",
            &[
                (&peer_name).into(),
                (&peer_dev_addr).into(),
                (&peer_logical_id).into(),
                (&byte_array).into(),
            ],
        )?;
        Ok(())
    }

    /// Breaks the cyclic owner <-> native listener.
    #[export_name = "Java_io_crisal_ngn_NgnSessionProxy_ngn_1session_1drop"]
    extern "C" fn drop<'l>(_env: JNIEnv<'l>, _class: JClass<'l>, raw: jlong) {
        trace!("Session::drop({raw:?})");
        let _ = unsafe { Arc::from_raw(raw as *const Self) };
    }

    /// Syncs the peer list. Expected an array of Strings.
    #[export_name = "Java_io_crisal_ngn_NgnSessionProxy_ngn_1session_1update_1peers"]
    extern "C" fn update_peers<'l>(
        mut env: JNIEnv<'l>,
        _class: JClass<'l>,
        raw: jlong,
        details: JObjectArray<'l>,
    ) {
        const STEP: usize = 2;
        trace!("Session::update_peers({raw:?})");
        let session = unsafe { &*(raw as *const Self) };
        let len = env.get_array_length(&details).unwrap();
        assert!(len as usize % STEP == 0, "Should have the right step");
        let mut identities = Vec::<PhysiscalPeerIdentity>::with_capacity(len as usize / STEP);
        let mut get_string = |i| {
            let string = env.get_object_array_element(&details, i).unwrap();
            let string = unsafe { JString::from_raw(string.as_raw()) };
            let string = env.get_string(&string).unwrap();
            string.to_string_lossy().into_owned()
        };
        for i in (0..len).step_by(STEP) {
            let name = get_string(i);
            let dev_addr = MacAddr::from_str(&get_string(i + 1)).unwrap();
            identities.push(PhysiscalPeerIdentity { name, dev_addr });
        }
        trace!(" > identities: {:?}", identities);
        session
            .java_notification
            .send(JavaNotification::UpdateDevices(identities))
            .unwrap();
    }

    /// Signals the group start operation. Android only supports one physical group at a time.
    #[export_name = "Java_io_crisal_ngn_NgnSessionProxy_ngn_1session_1group_1joined"]
    extern "C" fn group_joined<'l>(
        mut env: JNIEnv<'l>,
        _class: JClass<'l>,
        raw: jlong,
        is_go: jboolean,
        iface_name: JString<'l>,
        go_device_address: JString<'l>,
        go_ip_address: JString<'l>,
    ) {
        let session = unsafe { &*(raw as *const Self) };

        let is_go = is_go != 0;

        let iface_name = env.get_string(&iface_name).unwrap();
        let iface_name = iface_name.to_string_lossy().into_owned();

        let go_device_address = env.get_string(&go_device_address).unwrap();
        let go_device_address = MacAddr::from_str(&go_device_address.to_string_lossy()).unwrap();

        let go_ip_address = env.get_string(&go_ip_address).unwrap();
        let go_ip_address = std::net::IpAddr::from_str(&go_ip_address.to_string_lossy()).unwrap();

        trace!("Session::group_joined({is_go:?}, {iface_name:?}, {go_device_address:?}, {go_ip_address:?})");
        session
            .java_notification
            .send(JavaNotification::GroupStarted {
                is_go,
                iface_name,
                go_ip_address,
            })
            .unwrap();
    }

    #[export_name = "Java_io_crisal_ngn_NgnSessionProxy_ngn_1session_1message_1peer"]
    extern "C" fn message_peer<'l>(
        mut env: JNIEnv<'l>,
        _class: JClass<'l>,
        raw: jlong,
        peer_physical_address: JString<'l>,
        message: JByteArray<'l>,
        on_result: JObject<'l>,
    ) {
        let session = unsafe { &*(raw as *const Self) };
        let peer_physical_address = env.get_string(&peer_physical_address).unwrap();
        let peer_physical_address =
            MacAddr::from_str(&peer_physical_address.to_str().unwrap()).unwrap();
        let message = {
            let msg_len = env.get_array_length(&message).unwrap();
            let mut buf = vec![0u8; msg_len as usize];
            {
                // SAFETY: u8 and i8 share representation, it's just more convenient for us to use
                // u8.
                let signed_buf = unsafe {
                    std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut i8, buf.len())
                };
                env.get_byte_array_region(&message, 0, signed_buf).unwrap();
            }
            buf
        };
        let peer_id = session
            .peers
            .read()
            .mac_to_id
            .get(&peer_physical_address)
            .copied();
        let Some(peer_id) = peer_id else {
            error!("Failed to find peer {peer_physical_address} to message");
            try_void!(
                session.resolve_result(&on_result, false),
                "Failed to resolve on_result"
            );
            return;
        };
        let session = session.to_strong();
        let on_result = env.new_global_ref(on_result).unwrap();
        rt().spawn(async move {
            let result = session.message_peer(peer_id, &message).await;
            let success = result.is_ok();
            if let Err(e) = result {
                error!("Failed to message {peer_id:?}: {e}");
            }
            try_void!(
                session.resolve_result(on_result.as_obj(), success),
                "Failed to resolve on_result"
            );
        });
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
