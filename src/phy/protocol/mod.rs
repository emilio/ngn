//! Binary message protocol for our library. Basically a trivial 64-bit header, and a bunch of
//! bytes.
//!
//! The header of a message contains:
//!    magic: u16
//!    version: u16
//!    len: u32
//! Followed by `len` bytes.
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::trivial_error;

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
