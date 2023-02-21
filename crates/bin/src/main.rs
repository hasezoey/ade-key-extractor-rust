#[macro_use]
extern crate log;

use std::io::{
	BufWriter,
	Write,
};

use flexi_logger::LogSpecification;
use libade_extract_key::*;

use anyhow::Context;

mod clap_conf;
mod logger;

pub type Error = libade_extract_key::error::ExtractorError;

fn main() -> anyhow::Result<()> {
	let logger_handle = logger::setup_logger().context("Failed to set-up logger")?;
	flexi_logger::Logger::try_with_env()?;

	let cli_matches = clap_conf::CliDerive::custom_parse();

	if cli_matches.debug_enabled() {
		warn!("Requesting Debugger");

		#[cfg(debug_assertions)]
		{
			invoke_vscode_debugger();
		}
	}

	log::info!("CLI Verbosity is {}", cli_matches.verbosity);

	// dont do anything if "-v" is not specified (use env / default instead)
	if cli_matches.verbosity > 0 {
		// apply cli "verbosity" argument to the log level
		logger_handle.set_new_spec(
			match cli_matches.verbosity {
				0 => unreachable!("Unreachable because it should be tested before that it is higher than 0"),
				1 => LogSpecification::parse("info"),
				2 => LogSpecification::parse("debug"),
				3 => LogSpecification::parse("trace"),
				_ => {
					return Err(
						crate::Error::other("Expected verbosity integer range between 0 and 3 (inclusive)").into(),
					)
				},
			}
			.expect("Expected LogSpecification to parse correctly"),
		);
	}

	trace!("CLI setup done");

	let drive_info = decrypt::get_drive_info()?;
	let cpu_info = decrypt::get_cpu_info()?;
	let username = decrypt::get_win_username()?;
	let adept_info = decrypt::get_adept_information()?;

	let key = decrypt::decrypt(&drive_info, &cpu_info, &username, &adept_info)?;

	let file_path = cli_matches
		.output_file_name
		.expect("Expected output_file_name to be set at this point");

	let mut file = BufWriter::new(std::fs::File::create(&file_path)?);

	file.write_all(&key)?;

	println!("Wrote key to {}", file_path.to_string_lossy());

	return Ok(());
}
