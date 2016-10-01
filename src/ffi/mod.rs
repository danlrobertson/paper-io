#[cfg(target_os = "linux")]
mod epoll;
#[cfg(target_os = "linux")]
pub use self::epoll::Selector;

#[cfg(target_os = "freebsd")]
mod kqueue;
#[cfg(target_os = "freebsd")]
pub use self::kqueue::Selector;
