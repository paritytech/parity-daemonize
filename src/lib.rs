// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use crate::error::{Error, ErrorKind};
use std::path::PathBuf;

pub mod error;

mod platform;

type Result<T> = std::result::Result<T, Error>;

#[macro_export]
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

pub fn daemonize<T: Into<PathBuf>>(pid_file: T) -> Result<platform::Handle> {
	platform::daemonize(pid_file)
}
