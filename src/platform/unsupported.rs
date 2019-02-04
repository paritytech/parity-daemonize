use crate::{Error, ErrorKind};
use log::warn;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, Error>;

pub struct Handle;

impl Handle {
	pub fn from_fd(_fd: ()) -> Self {
		warn!(target: "daemonize", "Only Linux daemons are supported, this has no effect");
		Handle
	}

	/// detach the daemon from the parent process
	/// this will write "Daemon started successfully" to stdout
	/// before detaching
	///
	/// # panics
	/// if detach is called more than once
	pub fn detach(&mut self) {
		let msg = ansi_term::Colour::Green.paint("Only Linux daemons are supported, this has no effect\n").to_string();
		self.detach_with_msg(msg);
	}

	/// detach the daemon from the parent process
	/// with a custom message to be printed to stdout before detaching
	///
	/// # panics
	/// if detach_with_msg is called more than once
	pub fn detach_with_msg<T: AsRef<[u8]>>(&mut self, _msg: T) {}
}

pub fn daemonize<T: Into<PathBuf>>(_pid_file: T) -> Result<Handle> {
	Err(ErrorKind::UnsupportedPlatform)?
}
