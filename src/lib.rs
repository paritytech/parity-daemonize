// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Library for running a process as a daemon.
//! Currently, only Linux is supported.

#![warn(missing_docs, rust_2018_idioms)]

use crate::error::{Error, ErrorKind};
use std::path::PathBuf;

/// Error types
pub mod error;

mod platform;

type Result<T> = std::result::Result<T, Error>;

/// Handle returned from `daemonize` to the daemon process
/// the daemon should use this handle to detach itself from the
/// parent process, In cases where your program needs to run set up before starting
/// this can be useful, as the daemon will pipe it's stdout/stderr to the parent process
/// to communicate if start up was successful
pub trait AsHandle {
	/// File descriptor type
	type Fd;

	/// Creates a `Handle` from a raw file descriptor
	fn from_fd(fd: Self::Fd) -> Self;

	/// Detach the daemon from the parent process
	/// this will write "Daemon started successfully" to stdout
	/// before detaching
	///
	/// # panics
	/// if detach is called more than once
	fn detach(&mut self);

	/// Detach the daemon from the parent process
	/// with a custom message to be printed to stdout before detaching
	///
	/// # panics
	/// if detach_with_msg is called more than once
	fn detach_with_msg<T: AsRef<[u8]>>(&mut self, msg: T);
}

#[macro_export]
#[doc(hidden)]
/// Macro for calling `c-style functions` and wrapping the return status in a `Result`
/// If the function return `-1` it will return `Err<$err:expr>` otherwise `Ok(int)`
// FIXME: this is not platform independent: `https://github.com/paritytech/parity-daemonize/issues/14`
macro_rules! map_err {
	($e:expr, $err:expr) => {
		match $e {
			-1 => {
				Err::<_, crate::error::Error>(From::from($err))
			}
			other => Ok(other),
		}
	};
}

/// this will fork the calling process twice and return a handle to the
/// grandchild process aka daemon, use the handle to detach from the parent process
///
/// before `Handle::detach` is called the daemon process has it's STDOUT/STDERR
/// piped to the parent process' STDOUT/STDERR, this way any errors encountered by the
/// daemon during start up is reported.
pub fn daemonize<T: Into<PathBuf>>(pid_file: T) -> Result<impl AsHandle> {
	platform::daemonize(pid_file)
}
