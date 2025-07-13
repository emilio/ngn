//! Binary message protocol for our library. Basically a trivial 64-bit header, and a bunch of
//! bytes.
//!
//! The header of a message contains:
//!    magic: u16
//!    version: u16
//!    len: u32
//! Followed by `len` bytes.
use crate::{trivial_error, utils, GenericResult, GroupId, PeerId};
use bincode::{Decode, Encode};
use log::{error, trace};
use macaddr::MacAddr;
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr, SocketAddrV6},
    sync::OnceLock,
    time::Duration,
};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    task::JoinHandle,
};

pub mod identity;
use identity::{LogicalPeerIdentity, MaybeInvalidSignature, OwnIdentity};

const MAGIC: u16 = 0xdead;
const CURRENT_VERSION: u16 = 1;

async fn read_binary_message(
    mut reader: impl AsyncReadExt + Unpin,
    signature: Option<&mut MaybeInvalidSignature>,
) -> GenericResult<Vec<u8>> {
    let magic = reader.read_u16().await?;
    if magic != MAGIC {
        return Err(trivial_error!("Wrong message magic"));
    }
    let version = reader.read_u16().await?;
    if version != CURRENT_VERSION {
        return Err(trivial_error!("Wrong message version"));
    }

    let len = reader.read_u32().await?;

    // TODO(emilio): If zeroing shows up in the profile, we could use MaybeUninit + unsafe to work
    // around it.
    if let Some(signature) = signature {
        reader.read_exact(&mut signature.0).await?;
    }

    let mut buf = vec![];
    if len == 0 {
        return Ok(buf);
    }
    if buf.try_reserve(len as usize).is_err() {
        return Err(trivial_error!("OOM reading message"));
    }
    unsafe {
        // SAFETY: We have enough capacity as per the try_reserve call above. u8 doesn't have
        // invalid representations.
        // NOTE: We could avoid this unsafe usage by using buf.resize(len, 0), but that zeroes a
        // potentially large message, which is a bit wasteful.
        reader
            .read_exact(std::slice::from_raw_parts_mut(
                buf.as_mut_ptr(),
                len as usize,
            ))
            .await?;
        buf.set_len(len as usize);
    }
    Ok(buf)
}

/// Control messages are unsigned.
pub async fn read_control_message(
    reader: impl AsyncReadExt + Unpin,
    source_address: &SocketAddr,
) -> GenericResult<ControlMessage> {
    let buf = match read_binary_message(reader, None).await {
        Ok(buf) => buf,
        Err(e) => {
            log_error(&*e, source_address);
            return Err(e);
        }
    };
    let (control_message, len) =
        match bincode::decode_from_slice::<ControlMessage, _>(&buf, bincode::config::standard()) {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to decode binary control message {buf:?} {e:?}");
                return Err(e.into());
            }
        };
    if len != buf.len() {
        error!("Unexpected decoded message length {} vs {}", len, buf.len());
        return Err(trivial_error!("Invalid message length"));
    }
    Ok(control_message)
}

// TODO: In the future use OwnIdentity to also decrypt, not only check the signature from the peer.
pub async fn read_peer_message(
    _: &OwnIdentity,
    id: &LogicalPeerIdentity,
    reader: impl AsyncReadExt + Unpin,
    source_address: &SocketAddr,
) -> GenericResult<Vec<u8>> {
    // TODO: If zeroing somehow shows up it can be optimized via MaybeUninit + unsafe.
    let mut signature = MaybeInvalidSignature([0; identity::SIGNATURE_LEN]);
    let buf = match read_binary_message(reader, Some(&mut signature)).await {
        Ok(buf) => buf,
        Err(e) => {
            log_error(&*e, source_address);
            return Err(e);
        }
    };
    if let Err(e) = identity::verify(&id.key, &signature, &buf) {
        log_error(&*e, source_address);
        return Err(e);
    }
    Ok(buf)
}

pub fn log_error(e: &(dyn std::error::Error + 'static), source_address: &SocketAddr) {
    if let Some(io) = e.downcast_ref::<std::io::Error>() {
        if io.kind() == io::ErrorKind::UnexpectedEof {
            return trace!("Got EOF from {source_address:?}");
        }
    }
    error!("Unexpected error from {source_address:?}: {e}");
}

async fn write_binary_message(
    mut writer: impl AsyncWriteExt + Unpin,
    msg: &[u8],
    signing_key: Option<&identity::KeyPair>,
) -> GenericResult<()> {
    let Ok(len) = u32::try_from(msg.len()) else {
        return Err(trivial_error!("Huge length for binary message"));
    };

    // Write the header.
    writer.write_u16(MAGIC).await?;
    writer.write_u16(CURRENT_VERSION).await?;
    writer.write_u32(len).await?;
    if let Some(k) = signing_key {
        let signature = identity::sign(k, msg);
        writer.write_all(signature.as_ref()).await?;
    }
    // Write the message.
    writer.write_all(msg).await?;
    Ok(())
}

#[derive(Encode, Decode, Debug, Clone, Copy, PartialEq, Eq)]
pub struct P2pPorts {
    /// The port for the control channel.
    pub control: u16,
    /// The port for the p2p communication channel.
    pub p2p: u16,
}

/// A MacAddr-like type that we can easily binary encode / decode.
#[derive(Encode, Decode, Debug, Clone)]
pub struct DecodableMacAddr {
    is_v8: bool,
    bytes: [u8; 8],
}

impl From<MacAddr> for DecodableMacAddr {
    fn from(addr: MacAddr) -> Self {
        let is_v8 = addr.is_v8();
        let mac_bytes = addr.as_bytes();
        let mut bytes = [0u8; 8];
        bytes[..mac_bytes.len()].copy_from_slice(mac_bytes);
        Self { is_v8, bytes }
    }
}

impl DecodableMacAddr {
    pub fn to_mac_addr(&self) -> MacAddr {
        utils::to_mac_addr(if self.is_v8 {
            &self.bytes
        } else {
            &self.bytes[..6]
        })
        .unwrap()
    }
}

/// ID of a given device.
#[derive(Debug, Clone)]
pub struct PhysiscalPeerIdentity {
    /// Name of the device.
    pub name: String,
    /// Device address of the P2P device. Note this is _not_ usable to get a link-local
    /// address.
    pub dev_addr: MacAddr,
}

impl PhysiscalPeerIdentity {
    /// Whether this peer matches its self-reported own identifier.
    pub fn matches(&self, own_id: &PeerOwnIdentifier) -> bool {
        match own_id {
            PeerOwnIdentifier::Name(ref n) => self.name == *n,
            PeerOwnIdentifier::DevAddr(ref a) => self.dev_addr == a.to_mac_addr(),
        }
    }
}

impl std::fmt::Display for PhysiscalPeerIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        f.write_str(&self.name)?;
        f.write_str(" (")?;
        self.dev_addr.fmt(f)?;
        f.write_char(')')
    }
}

/// A peer identity at the current time
#[derive(Debug, Clone)]
pub struct PeerIdentity {
    pub physical: PhysiscalPeerIdentity,
    /// The logical identity to be able to verify messages from a given peer. Unknown until
    /// association.
    pub logical: Option<LogicalPeerIdentity>,
}

/// A single self-reported identifier for a peer. Note that ideally this should always be the mac
/// address, but:
///
///   * The DBUS backend doesn't easily expose it (yet[1]).
///   * Android restricts it to non-privileged apps because it's considered a persistent identifier
///     (though realistically it could be randomized and exposed I guess?).
///
/// So for now we allow to self-report the name instead. This is fine because it's not intended to
/// be a hard security boundary, that is expected to be implemented at a different layer (either by
/// not connecting physically to this device, or by authenticating to this device).
///
/// [1]: https://lists.infradead.org/pipermail/hostap/2025-May/043428.html
#[derive(Encode, Decode, Debug, Clone)]
pub enum PeerOwnIdentifier {
    Name(String),
    DevAddr(DecodableMacAddr),
}

/// Information from a peer (other than ourselves).
#[derive(Debug)]
pub struct PeerInfo<BackendData> {
    /// Identity of this peer.
    pub identity: PeerIdentity,
    /// Current list of groups the peer is connected to.
    pub groups: Vec<GroupId>,
    /// Back-end specific data.
    pub data: BackendData,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeerAddress {
    pub address: IpAddr,
    pub ports: P2pPorts,
}

/// Given a peer and scope id, get an appropriate SocketAddr to send messages to.
pub fn peer_to_socket_addr(addr: IpAddr, scope_id: u32, port: u16) -> SocketAddr {
    match addr {
        IpAddr::V4(..) => SocketAddr::new(addr, port),
        IpAddr::V6(a) => SocketAddr::V6(SocketAddrV6::new(
            a, port, /* flowinfo = */ 0, scope_id,
        )),
    }
}

/// Send a signed (if with own identity) or unsigned (otherwise) message to a given peer address.
pub async fn send_message(
    from: Option<&OwnIdentity>,
    to: &SocketAddr,
    message: &[u8],
) -> GenericResult<()> {
    trace!("send_message_to({to:?}, {})", message.len());
    let mut stream =
        match tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(to)).await? {
            Ok(stream) => stream,
            Err(e) => return Err(io::Error::new(io::ErrorKind::TimedOut, e).into()),
        };
    let key_pair = from.map(|f| &f.key_pair);
    write_binary_message(&mut stream, message, key_pair).await
}

/// Read a signed message from a given peer address.
pub async fn recv_message(
    from: Option<&OwnIdentity>,
    to: &SocketAddr,
    message: &[u8],
) -> GenericResult<()> {
    trace!("send_message_to({to:?}, {})", message.len());
    let mut stream =
        match tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(to)).await? {
            Ok(stream) => stream,
            Err(e) => return Err(io::Error::new(io::ErrorKind::TimedOut, e).into()),
        };
    let key_pair = from.map(|f| &f.key_pair);
    write_binary_message(&mut stream, message, key_pair).await
}

/// Per group association for a given peer.
#[derive(Debug)]
pub struct PeerGroupInfo {
    /// The link-local address of this peer, and the ports it uses to listen to connections, if
    /// known.
    pub address: PeerAddress,
}

/// Information about the current group.
#[derive(Debug)]
pub struct GroupInfo<BackendData> {
    /// Ip address of the group owner.
    pub go_ip_address: IpAddr,
    /// Name of the interface.
    pub iface_name: String,
    /// Scope id of the interface. This can be derived by the name via if_nametoindex.
    pub scope_id: u32,
    /// Whether we're the group owner.
    pub is_go: bool,
    /// The current peers we have. Only around if we're a GO
    ///
    /// TODO(emilio): Broadcast these to non-GO members?
    pub peers: HashMap<PeerId, PeerGroupInfo>,
    /// Task handle to our connection loop. Canceled and awaited on drop.
    pub group_task: OnceLock<JoinHandle<GenericResult<()>>>,
    /// Back-end specific data for this group.
    pub data: BackendData,
}

impl<B> Drop for GroupInfo<B> {
    fn drop(&mut self) {
        if let Some(task) = self.group_task.take() {
            task.abort();
        }
    }
}

/// The port the GO of the group listens to.
pub const GO_CONTROL_PORT: u16 = 9001;

/// Control messages defined for the IPv6-based protocol. Note this must be independent of the
/// underlying platform (e.g. dbus vs. android).
///
/// TODO(emilio): The binary message should probably be stable. Maybe use protobuf or (god forbid)
/// json or something?
#[derive(Encode, Decode, Debug)]
pub enum ControlMessage {
    /// Associate this sender with a pre-existing WifiP2P peer, communicating the ports we're
    /// listening to.
    Associate {
        /// The identifier used to associate back to the peer.
        physical_id: PeerOwnIdentifier,
        /// The logical identity of our peer.
        logical_id: LogicalPeerIdentity,
        /// The ports the peer is listening to.
        ports: P2pPorts,
    },
}
