# Daemonize-rs ![Crates.io](https://img.shields.io/crates/d/daemonize-rs.svg) [![Released API docs](https://docs.rs/daemonize-rs/badge.svg)](https://docs.rs/daemonize-rs) [![Build Status][travis-image]][travis-url] [![Build Status][appveyor-image]][appveyor-url]

[travis-image]: https://travis-ci.org/paritytech/parity-daemonize.svg?branch=master
[travis-url]: https://travis-ci.org/paritytech/parity-daemonize
[appveyor-image]: https://ci.appveyor.com/api/projects/status/github/paritytech/parity-daemonize?svg=true
[appveyor-url]: https://ci.appveyor.com/project/paritytech/parity-daemonize

## Example

```rust

extern crate parity_daemonize;

use parity_daemonize::{AsHandle, daemonize};
use std::{thread, time, process, io};
use io::Write;

fn main() {
    match daemonize("pid_file.txt") {
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
            let _  = io::stderr().flush();
            process::exit(1);
        }
    }
}

```

## License

This crate is distributed under the terms of GNU GENERAL PUBLIC LICENSE version 3.0.

See [LICENSE](LICENSE) for details.
