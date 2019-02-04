use libc::{
	close, dup2, fork, getpid, ioctl, pipe, splice, setsid, FIONREAD, STDERR_FILENO,
	STDIN_FILENO, STDOUT_FILENO, c_int, umask, open, gid_t, uid_t, setgid, setuid
};
use mio::*;
use std::{
	fs,
	env::set_current_dir,
	path::PathBuf,
	ffi::CString,
	io::{self, Write},
	os::unix::{
		ffi::OsStringExt,
		io::{FromRawFd, RawFd}
	},
	ptr
};
use log::{trace, error};

use crate::{AsHandle, Error, ErrorKind, map_err};
use super::unix_pipe::*;

type Result<T> = std::result::Result<T, Error>;

pub struct Handle {
	file: Option<fs::File>
}

impl AsHandle for Handle {
	type Fd = RawFd;

	fn from_fd(fd: Self::Fd) -> Self {
		unsafe {
			Self {
				file: Some(fs::File::from_raw_fd(fd))
			}
		}
	}

	fn detach(&mut self) {
		let msg = ansi_term::Colour::Green.paint("Daemon started successfully, detaching ...\n").to_string();
		self.detach_with_msg(msg);
	}

	fn detach_with_msg<T: AsRef<[u8]>>(&mut self, msg: T) {
		let mut file = self.file.take().expect("detach should only be called once");

		// redirect stdout/stderr to dev/null
		unsafe {
			let fd = open(b"/dev/null\0" as *const u8 as *const _, libc::O_RDWR);
			let result = map_err!(dup2(fd, STDERR_FILENO), ErrorKind::Dup2(io::Error::last_os_error())).and_then(
				|_| map_err!(dup2(fd, STDOUT_FILENO), ErrorKind::Dup2(io::Error::last_os_error()))
			);
			if result.is_err() {
				error!(target: "daemonize", "Couldn't redirect STDOUT/STDERR to /dev/null, daemon will panic")
			}
		}

		file.write_all(msg.as_ref())
			.expect("Parent process won't exit until detach is called; \
			write can only fail if the read end of pipe is closed; qed");
	}
}


/// this will fork the calling process twice and return a handle to the
/// grandchild process aka daemon, use the handle to detach from the parent process
///
/// before `Handle::detach` is called the daemon process has it's STDOUT/STDERR
/// piped to the parent process' STDOUT/STDERR, this way any errors encountered by the
/// daemon during start up is reported.
pub fn daemonize<T: Into<PathBuf> + Sized>(pid_file: T) -> Result<Handle> {
	unsafe {
		let mut chan = [-1 as c_int, -1 as c_int];
		let mut out_chan = [-1 as c_int, -1 as c_int];
		let mut err_chan = [-1 as c_int, -1 as c_int];

		map_err!(pipe(&mut chan[0] as *mut c_int), ErrorKind::Pipe(io::Error::last_os_error()))?;
		map_err!(pipe(&mut out_chan[0] as *mut c_int), ErrorKind::Pipe(io::Error::last_os_error()))?;
		map_err!(pipe(&mut err_chan[0] as *mut c_int), ErrorKind::Pipe(io::Error::last_os_error()))?;

		let path = pid_file.into();
		let path_c = CString::new(path.clone().into_os_string().into_vec())
			.map_err(|_| ErrorKind::PathContainsNul)?;

		// create the pid file
		let pid_fd = map_err!(
			open(path_c.as_ptr(), libc::O_WRONLY | libc::O_CREAT, 0o666),
			ErrorKind::OpenPidfile(io::Error::last_os_error())
		)?;

		let (rx, tx) = (chan[0], chan[1]);
		let (out_rx, out_tx) = (out_chan[0], out_chan[1]);
		let (err_rx, err_tx) = (err_chan[0], err_chan[1]);

		// fork once
		let pid = map_err!(fork(), ErrorKind::Fork(io::Error::last_os_error()))?;

		if pid == 0 {
			// redirect stderr/stdout to out/err pipe
			// incase we get an error before forking
			map_err!(dup2(err_tx, STDERR_FILENO), ErrorKind::Dup2(io::Error::last_os_error()))?;
			map_err!(dup2(out_tx, STDOUT_FILENO), ErrorKind::Dup2(io::Error::last_os_error()))?;
			trace!(target: "daemonize", "created child Process! {}", getpid());

			set_current_dir("/").map_err(|_| ErrorKind::ChangeDirectory)?;
			set_sid()?;
			umask(0o027);
			// fork again
			let pid = map_err!(fork(), ErrorKind::Fork(io::Error::last_os_error()))?;

			// kill the the old parent
			if pid != 0 {
				trace!(target: "daemonize", "exiting child process! {}", getpid());
				::std::process::exit(0)
			}

			// we are now in the grandchild process aka daemon
			// close unused fds
			for fd in &[
				rx,
				out_rx,
				err_rx,
				STDERR_FILENO,
				STDIN_FILENO,
				STDOUT_FILENO,
			] {
				close(*fd);
			}

			// redirect stderr/stdout to out/err pipe
			map_err!(dup2(err_tx, STDERR_FILENO), ErrorKind::Dup2(io::Error::last_os_error()))?;
			map_err!(dup2(out_tx, STDOUT_FILENO), ErrorKind::Dup2(io::Error::last_os_error()))?;

			let gid = gid_t::max_value() - 1;
			let uid = uid_t::max_value() - 1;
			// set the process group_id and user_id
			setgid(gid);
			setuid(uid);

			// write the pid to the pid file
			let mut pid_f = fs::File::from_raw_fd(pid_fd);
			pid_f.write_all(
				format!("{}", getpid()).as_bytes()
			).map_err(ErrorKind::WritePid)?;

			close(err_tx);
			close(out_tx);

			trace!(target: "daemonize", "grandchild process {}, aka daemon", getpid());

			Ok(AsHandle::from_fd(tx))
		} else {
			// parent process
			trace!(target: "daemonize", "Parent process {}", getpid());

			for fd in &[tx, out_tx, err_tx] {
				close(*fd);
			}

			// use mio to listen for events on all pipes
			const STDOUT_READ_PIPE: Token = Token(0);
			const STDERR_READ_PIPE: Token = Token(1);
			const STATUS_REPORT_PIPE: Token = Token(3);

			let poll = mio::Poll::new().unwrap();

			let (stdout_read, stderr_read, status_read) = (
				EventedPipe::from_fd(out_rx)?,
				EventedPipe::from_fd(err_rx)?,
				EventedPipe::from_fd(rx)?
			);

			poll.register(
				&stdout_read,
				STDOUT_READ_PIPE,
				Ready::readable(),
				PollOpt::edge(),
			).map_err(ErrorKind::RegisterationError)?;

			poll.register(
				&stderr_read,
				STDERR_READ_PIPE,
				Ready::readable(),
				PollOpt::edge(),
			).map_err(ErrorKind::RegisterationError)?;

			poll.register(
				&status_read,
				STATUS_REPORT_PIPE,
				Ready::readable(),
				PollOpt::edge(),
			).map_err(ErrorKind::RegisterationError)?;

			let mut events = Events::with_capacity(1024);

			loop {
				poll.poll(&mut events, None).expect("");

				for event in events.iter() {
					match event.token() {
						STDOUT_READ_PIPE => {
							let size = get_pending_data_size(out_rx)?;

							map_err!(
								splice(out_rx, ptr::null_mut(), STDOUT_FILENO, ptr::null_mut(), size, 0),
								ErrorKind::SpliceError(io::Error::last_os_error())
							)?;
						}
						STDERR_READ_PIPE => {
							let size = get_pending_data_size(err_rx)?;

							map_err!(
								splice(err_rx, ptr::null_mut(), STDERR_FILENO, ptr::null_mut(), size, 0),
								ErrorKind::SpliceError(io::Error::last_os_error())
							)?;
						}
						STATUS_REPORT_PIPE => {
							let size = get_pending_data_size(rx)?;

							map_err!(
								splice(rx, ptr::null_mut(), STDOUT_FILENO, ptr::null_mut(), size, 0),
								ErrorKind::SpliceError(io::Error::last_os_error())
							)?;

							trace!(target: "daemonize", "Exiting Parent Process");
							for fd in &[rx, out_rx, err_rx] {
								close(*fd);
							}
							::std::process::exit(0);
						}
						_ => unreachable!(),
					}
				}
			}
		}
	}
}

// helpers
unsafe fn set_sid() -> Result<()> {
	map_err!(setsid(), ErrorKind::DetachSession(io::Error::last_os_error()))?;
	Ok(())
}

unsafe fn get_pending_data_size(fd: RawFd) -> Result<usize> {
	let mut size: usize = 0;
	map_err!(
		ioctl(fd, FIONREAD, &mut size),
		ErrorKind::Ioctl(io::Error::last_os_error())
	)?;
	Ok(size)
}
