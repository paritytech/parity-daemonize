use std::{fs, io::Read, process};

#[test]
fn test_simple() {
	let output = process::Command::new("target/debug/examples/simple").output().unwrap();

	assert_eq!(
		"Count: 0\nCount: 1\nCount: 2\nCount: 3\nCount: 4\nCount: 5\ncount has reached 5, continuing in background",
		String::from_utf8(output.stdout).unwrap()
	);

	let mut file = fs::File::open("pid_file").unwrap();
	let mut pid = String::new();
	file.read_to_string(&mut pid).unwrap();

	let _ = process::Command::new("kill")
		.arg("-9")
		.arg(&pid)
		.output()
		.expect("");
}
