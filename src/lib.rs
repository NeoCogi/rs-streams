#![no_std]
pub mod memorystream;
pub mod file;

pub use libc::*;
pub use file::*;
pub use memorystream::*;

use cfg_if::*;

// scavanged from nix crate!
cfg_if! {
    if #[cfg(any(target_os = "freebsd",
                 target_os = "ios",
                 target_os = "macos"))] {
        unsafe fn errno_location() -> *mut c_int {
            libc::__error()
        }
    } else if #[cfg(target_os = "dragonfly")] {
        // DragonFly uses a thread-local errno variable, but #[thread_local] is
        // feature-gated and not available in stable Rust as of this writing
        // (Rust 1.21.0). We have to use a C extension to access it
        // (src/errno_dragonfly.c).
        //
        // Tracking issue for `thread_local` stabilization:
        //
        //     https://github.com/rust-lang/rust/issues/29594
        //
        // Once this becomes stable, we can remove build.rs,
        // src/errno_dragonfly.c, and use:
        //
        //     extern { #[thread_local] static errno: c_int; }
        //
        #[link(name="errno_dragonfly", kind="static")]
        extern {
            pub fn errno_location() -> *mut c_int;
        }
    } else if #[cfg(any(target_os = "android",
                        target_os = "netbsd",
                        target_os = "openbsd"))] {
        unsafe fn errno_location() -> *mut c_int {
            libc::__errno()
        }
    } else if #[cfg(any(target_os = "linux", target_os = "redox", target_os="emscripten"))] {
        unsafe fn errno_location() -> *mut libc::c_int {
            libc::__errno_location()
        }
    }
}

pub enum ErrorKind {
    NotFound,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    ConnectionAborted,
    NotConnected,
    AddrInUse,
    AddrNotAvailable,
    BrokenPipe,
    AlreadyExists,
    WouldBlock,
    InvalidInput,
    InvalidData,
    TimedOut,
    WriteZero,
    Interrupted,
    Other,
    UnexpectedEof,
}

impl ErrorKind {
    pub fn as_str(&self) -> &'static str {
        match *self {
            ErrorKind::NotFound => "entity not found",
            ErrorKind::PermissionDenied => "permission denied",
            ErrorKind::ConnectionRefused => "connection refused",
            ErrorKind::ConnectionReset => "connection reset",
            ErrorKind::ConnectionAborted => "connection aborted",
            ErrorKind::NotConnected => "not connected",
            ErrorKind::AddrInUse => "address in use",
            ErrorKind::AddrNotAvailable => "address not available",
            ErrorKind::BrokenPipe => "broken pipe",
            ErrorKind::AlreadyExists => "entity already exists",
            ErrorKind::WouldBlock => "operation would block",
            ErrorKind::InvalidInput => "invalid input parameter",
            ErrorKind::InvalidData => "invalid data",
            ErrorKind::TimedOut => "timed out",
            ErrorKind::WriteZero => "write zero",
            ErrorKind::Interrupted => "operation interrupted",
            ErrorKind::Other => "other os error",
            ErrorKind::UnexpectedEof => "unexpected end of file",
        }
    }
}

enum Repr {
    Os(i32),
    Simple(ErrorKind),
}

pub struct Error {
    repr: Repr,
}

impl From<ErrorKind> for Error {
    /// Converts an [`ErrorKind`] into an [`Error`].
    ///
    /// This conversion allocates a new error with a simple representation of error kind.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Error, ErrorKind};
    ///
    /// let not_found = ErrorKind::NotFound;
    /// let error = Error::from(not_found);
    /// assert_eq!("entity not found", format!("{}", error));
    /// ```
    ///
    /// [`ErrorKind`]: ../../std/io/enum.ErrorKind.html
    /// [`Error`]: ../../std/io/struct.Error.html
    #[inline]
    fn from(kind: ErrorKind) -> Error {
        Error { repr: Repr::Simple(kind) }
    }
}

impl Error {
    pub fn last_os_error() -> Error {
        Error::from_raw_os_error(unsafe { *(errno_location()) })
    }

    pub fn from_raw_os_error(code: i32) -> Error {
        Error { repr: Repr::Os(code) }
    }
}

pub trait Stream : Drop {
    /// get the current position
    fn tell(&self) -> usize;

    /// get the size of the stream
    fn size(&self) -> usize;
}

pub trait StreamReader : Stream {
    fn read(&mut self, buff: &mut [u8]) -> Result<usize, ()>;
    fn is_eof(&self) -> bool;
}

pub trait StreamWriter : Stream {
    fn write(&mut self, buff: &[u8]) -> Result<usize, ()>;
}

pub trait StreamSeek : Stream {
    fn seek(&mut self, cursor: usize) -> Result<usize, ()>;
}
