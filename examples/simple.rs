use daemonize_rs::daemonize;
use std::{thread, time, process, io};
use self::io::Write;

fn main() {
	match daemonize("pid_file") {
		// we are now in the daemon, use this handle to detach from the parent process
		Ok(mut handle) => {
			let mut count = 0;
			loop {
				// the daemon's output is piped to the parent process' stdout
				println!("Count: {}", count);
				if count == 5 {
					handle.detach_with_msg("count has reached 5, continuing in background");
				}
				thread::sleep(time::Duration::from_secs(1));
				count += 1;
			}
		}
		// the daemon or the parent process may receive this error,
		// just print it and exit
		Err(e) => {
			// if this is the daemon, this is piped to the parent's stderr
			eprintln!("{}", e);
			// don't forget to flush
			let _ = io::stderr().flush();
			process::exit(1);
		}
	}
}
