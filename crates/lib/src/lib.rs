/*!
 * TODO
 */

#[macro_use]
extern crate log;

pub mod decrypt;
pub mod error;

pub type Error = error::ExtractorError;

pub fn test() {
	println!("lib");
}

/// Debug function to start vscode-lldb debugger from external console
/// Only compiled when the target is "debug"
#[cfg(debug_assertions)]
pub fn invoke_vscode_debugger() {
	println!("Requesting Debugger");
	// Request VSCode to open a debugger for the current PID
	let url = format!(
		"vscode://vadimcn.vscode-lldb/launch/config?{{'request':'attach','pid':{}}}",
		std::process::id()
	);
	std::process::Command::new("code")
		.arg("--open-url")
		.arg(url)
		.output()
		.unwrap();

	println!("Press ENTER to continue");
	let _ = std::io::stdin().read_line(&mut String::new()); // wait until attached, then press ENTER to continue
}
