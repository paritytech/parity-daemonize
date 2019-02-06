use crate::{Error, ErrorKind, AsHandle};
use log::warn;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, Error>;

pub struct Handle;

impl AsHandle for Handle {
	type Fd = ();

	fn from_fd(_fd: Self::Fd) -> Self {
		warn!(target: "daemonize", "Only Linux daemons are supported, this has no effect");
		Handle
	}

	fn detach(&mut self) {
		let msg = ansi_term::Colour::Green.paint("Only Linux daemons are supported, this has no effect\n").to_string();
		self.detach_with_msg(msg);
	}

	fn detach_with_msg<T: AsRef<[u8]>>(&mut self, _msg: T) {}
}

pub fn daemonize<T: Into<PathBuf>>(_pid_file: T) -> Result<Handle> {
	Err(ErrorKind::UnsupportedPlatform)?
}
