#[cfg(unix)]
mod unix_pipe;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(not(target_os = "linux"))]
mod unsupported;

#[cfg(target_os = "linux")]
pub use self::linux::*;

#[cfg(not(target_os = "linux"))]
pub use self::unsupported::*;
