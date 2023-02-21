use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExtractorError {
	#[error("{0}")]
	Other(String),
}

impl ExtractorError {
	pub fn other<M>(msg: M) -> Self
	where
		M: Into<String>,
	{
		return Self::Other(msg.into());
	}
}
