use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExtractorError {
	#[error("Failed to get regex captures for {0}")]
	NoCaptures(&'static str),

	#[error("Could not find binary \"{0}\" in \"{1}\"")]
	NoBinary(&'static str, PathBuf),

	#[error("Could not find {0} Registry key \"{1}\"")]
	RegistryKeyNotFound(&'static str, &'static str),

	#[error("{0}")]
	AssertionFailed(String),

	#[error("{0}")]
	Other(String),
}

impl ExtractorError {
	/// Error for when regex captures fails
	pub fn no_captures(msg: &'static str) -> Self {
		return Self::NoCaptures(msg);
	}

	/// Error for when not being able to find a specific registry key
	pub fn no_adept_reg_key(key: &'static str) -> Self {
		return Self::RegistryKeyNotFound("Adept", key);
	}

	/// Error for when a assertion / expectation failed, without panicing
	pub fn assertion_failed(msg: String) -> Self {
		return Self::AssertionFailed(msg);
	}

	/// Error with arbitrary, one-off meaning
	pub fn other<M>(msg: M) -> Self
	where
		M: Into<String>,
	{
		return Self::Other(msg.into());
	}
}
