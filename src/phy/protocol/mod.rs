//! Binary message protocol for our library. Basically a trivial 64-bit header, and a bunch of
//! bytes.
//!
//! The header of a message contains:
//!    magic: u16
//!    version: u16
//!    len: u32
//! Followed by `len` bytes.
use super::{GroupId, PeerId};
use bincode::{Decode, Encode};
use macaddr::MacAddr;
use std::{collections::HashMap, net::Ipv6Addr, sync::OnceLock};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    task::JoinHandle,
};

use crate::{trivial_error, utils};

use super::GenericResult;

const MAGIC: u16 = 0xdead;
const CURRENT_VERSION: u16 = 1;

pub async fn read_binary_message(mut reader: impl AsyncReadExt + Unpin) -> GenericResult<Vec<u8>> {
    let magic = reader.read_u16().await?;
    if magic != MAGIC {
        return Err(trivial_error!("Wrong message magic"));
    }
    let version = reader.read_u16().await?;
    if version != CURRENT_VERSION {
        return Err(trivial_error!("Wrong message version"));
    }
    let len = reader.read_u32().await?;
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

pub async fn write_binary_message(
    mut writer: impl AsyncWriteExt + Unpin,
    msg: &[u8],
) -> GenericResult<()> {
    let Ok(len) = u32::try_from(msg.len()) else {
        return Err(trivial_error!("Huge length for binary message"));
    };

    // Write the header.
    writer.write_u16(MAGIC).await?;
    writer.write_u16(CURRENT_VERSION).await?;
    writer.write_u32(len).await?;
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
pub struct PeerIdentity {
    /// Name of the device.
    pub name: String,
    /// Device address of the P2P device. Note this is _not_ usable to get a link-local
    /// address.
    pub dev_addr: MacAddr,
}

impl PeerIdentity {
    /// Whether this peer matches its self-reported own identifier.
    pub fn matches(&self, own_id: &PeerOwnIdentifier) -> bool {
        match own_id {
            PeerOwnIdentifier::Name(ref n) => self.name == *n,
            PeerOwnIdentifier::DevAddr(ref a) => self.dev_addr == a.to_mac_addr(),
        }
    }
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
    /// Identity of this peer. Note that even tho the mac address is indeed
    pub identity: PeerIdentity,
    /// Current list of groups the peer is connected to.
    pub groups: Vec<GroupId>,
    /// Back-end specific data.
    pub data: BackendData,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeerAddress {
    pub link_local_address: Ipv6Addr,
    pub ports: P2pPorts,
}

/// Per group association for a given peer.
#[derive(Debug, Default)]
pub struct PeerGroupInfo {
    /// The link-local address of this peer, and the ports it uses to listen to connections, if
    /// known.
    pub address: Option<PeerAddress>,
}

/// Information about the current group.
#[derive(Debug)]
pub struct GroupInfo<BackendData> {
    /// Mac address of the interface, useful to get a local link address for a group owner.
    pub go_iface_addr: MacAddr,
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
        id: PeerOwnIdentifier,
        /// The ports the peer is listening to.
        ports: P2pPorts,
    },
}
