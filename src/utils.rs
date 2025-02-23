//! Miscellaneous utilities.

use std::time::Duration;
use log::error;

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
    E: std::error::Error,
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
    E: std::error::Error,
{
    retry_timeout(Duration::from_millis(500), count, thing).await
}
