//! Miscellaneous utilities.

use log::error;
use macaddr::MacAddr;
use std::net::Ipv6Addr;
use std::time::Duration;

#[macro_export]
macro_rules! trivial_error {
    ($($args:tt)*) => {{
        struct TrivialError;
        impl std::error::Error for TrivialError {}
        impl std::fmt::Debug for TrivialError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(self, f)
            }
        }
        impl std::fmt::Display for TrivialError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, $($args)*)
            }
        }
        Box::new(TrivialError) as _
    }}
}

pub use trivial_error;

pub async fn retry_timeout<T, E, Fut>(
    timeout: Duration,
    mut count: usize,
    mut thing: impl FnMut() -> Fut,
) -> Result<T, E>
where
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    assert!(count > 0);
    loop {
        match thing().await {
            Ok(r) => return Ok(r),
            Err(e) => {
                count -= 1;
                error!("retry: {e}, {count} retries left");
                if count == 0 {
                    return Err(e);
                }
                tokio::time::sleep(timeout).await;
            }
        }
    }
}

pub async fn retry<T, E, Fut>(count: usize, thing: impl FnMut() -> Fut) -> Result<T, E>
where
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    retry_timeout(Duration::from_millis(500), count, thing).await
}

/// Turns a raw buffer into a mac address.
pub fn to_mac_addr(buff: &[u8]) -> Option<MacAddr> {
    let len = buff.len();
    if len != 6 && len != 8 {
        return None;
    }
    // FIXME: Seems this could be cleaner.
    Some(if len == 6 {
        let buff6: [u8; 6] = std::array::from_fn(|i| buff[i]);
        MacAddr::from(buff6)
    } else {
        let buff8: [u8; 8] = std::array::from_fn(|i| buff[i]);
        MacAddr::from(buff8)
    })
}

/// Convert eui48 mac to eui64 per:
/// https://www.rfc-editor.org/rfc/rfc4291#:~:text=%5BEUI64%5D%20defines%20a%20method%20to%20create%20an%20IEEE%20EUI%2D64%20identifier%20from%20an%0A%20%20%20IEEE%2048%2Dbit%20MAC%20identifier.
fn to_eui64(mac_addr: &MacAddr) -> [u8; 8] {
    let a = match mac_addr {
        MacAddr::V6(a) => a.as_bytes(),
        MacAddr::V8(a) => return a.clone().into_array(),
    };
    [a[0], a[1], a[2], 0xff, 0xfe, a[3], a[4], a[5]]
}

/// Turns a mac address into a Ipv6 local-link address as per:
/// https://www.rfc-editor.org/rfc/rfc4862#section-5.3
/// https://cs.android.com/android/platform/superproject/main/+/main:packages/modules/Connectivity/framework/src/android/net/MacAddress.java;drc=725fc18d701f2474328b8f21710da13d9bbb7eaf;l=379 for eui48
pub fn mac_addr_to_local_link_address(mac_addr: &MacAddr) -> Ipv6Addr {
    let mut addr = [0u8; 16];
    let eui64 = to_eui64(mac_addr);

    // fe80 is the local link prefix.
    addr[0] = 0xfe;
    addr[1] = 0x80;

    addr[8] = eui64[0] ^ 0x02; // flip the link-local bit
    addr[9] = eui64[1];
    addr[10] = eui64[2];
    addr[11] = eui64[3];
    addr[12] = eui64[4];
    addr[13] = eui64[5];
    addr[14] = eui64[6];
    addr[15] = eui64[7];

    Ipv6Addr::from(addr)
}
