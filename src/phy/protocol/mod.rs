//! Binary message protocol for our library. Basically a trivial 64-bit header, and a bunch of
//! bytes.
//!
//! The header of a message contains:
//!    magic: u16
//!    version: u16
//!    len: u32
//! Followed by `len` bytes.
use bincode::{Decode, Encode};
use macaddr::MacAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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

/// A MacAddr-like type that we can easily binary encode / decode.
#[derive(Encode, Decode, Debug)]
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
        }).unwrap()
    }
}

/// Control messages defined for the IPv6-based protocol. Note this must be independent of the
/// underlying platform (e.g. dbus vs. android).
///
/// TODO(emilio): The binary message should probably be stable. Maybe use protobuf or (god forbid)
/// json or something?
#[derive(Encode, Decode, Debug)]
pub enum ControlMessage {
    /// Associate this sender with a pre-existing WifiP2P peer.
    /// The mac address is the device address of the P2P interface used for discovery and
    /// communication.
    Associate(DecodableMacAddr),
}
