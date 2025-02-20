use std::path::PathBuf;

use clap::{
	ArgAction,
	Parser,
	Subcommand,
};

/// Trait to check and transform all Command Structures
trait Check {
	/// Check and transform self to be correct
	fn check(&mut self) -> Result<(), crate::Error>;
}

#[derive(Debug, Parser, Clone, PartialEq)]
#[command(author, version, about, version = env!("PROJECT_VERSION"), long_about = None)]
#[command(bin_name("ade-extract-key"))]
#[command(disable_help_subcommand(true))] // Disable subcommand "help", only "-h --help" should be used
#[command(subcommand_negates_reqs(true))]
pub struct CliDerive {
	/// Set Loggin verbosity (0 - Default - WARN, 1 - INFO, 2 - DEBUG, 3 - TRACE)
	#[arg(short, long, action = ArgAction::Count)]
	pub verbosity:        u8,
	/// Request vscode lldb debugger before continuing to execute.
	/// Only available in debug target
	#[arg(long)]
	#[cfg(debug_assertions)]
	pub debugger:         bool,
	/// Change output file name / directory
	pub output_file_name: Option<PathBuf>,

	#[command(subcommand)]
	pub subcommands: Option<SubCommands>,
}

impl CliDerive {
	/// Execute clap::Parser::parse and apply custom validation and transformation logic
	#[must_use]
	pub fn custom_parse() -> Self {
		let mut parsed = Self::parse();

		Check::check(&mut parsed).expect("Expected the check to not fail"); // TODO: this should maybe be actually handled

		return parsed;
	}

	/// Get if debug is enabled
	/// Only able to be "true" in "debug" target
	#[must_use]
	#[cfg(debug_assertions)]
	pub fn debug_enabled(&self) -> bool {
		return self.debugger;
	}
}

/// Default filename for the output
const KEY_DEFAULT_FILENAME: &str = "ade_key.der";

impl Check for CliDerive {
	fn check(&mut self) -> Result<(), crate::Error> {
		if let Some(p) = self.output_file_name.as_mut() {
			if p.file_name().is_none() {
				info!("Output Path was set, but without filename");
				p.set_file_name(KEY_DEFAULT_FILENAME);
			}
		}
		if self.output_file_name.is_none() {
			self.output_file_name = Some(PathBuf::from(KEY_DEFAULT_FILENAME));
		}

		return Ok(());
	}
}

#[derive(Debug, Subcommand, Clone, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum SubCommands {
	AES(AESCli),
}

/// Resume at the AES decryption stage with the winapi decrypted key
#[derive(Debug, Parser, Clone, PartialEq)]
pub struct AESCli {
	/// The key from the winapi-bin
	pub key:       String,
	/// The adept key from a previous run of the binary
	pub adept_key: String,
}

impl Check for AESCli {
	fn check(&mut self) -> Result<(), crate::Error> {
		if self.key.is_empty() {
			return Err(crate::Error::other("Key cannot be empty"));
		}

		if self.adept_key.is_empty() {
			return Err(crate::Error::other("Adept Key cannot be empty"));
		}

		return Ok(());
	}
}
