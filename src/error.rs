use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Error)]
pub enum Error {
  #[error("Could not locate launcher dir")]
  NoLauncherDirFound,
  #[error("{1}: {0}")]
  FileError(io::Error, String),
  #[error("Failed to parse Steam.json: {0}")]
  AddonsParsingFailed(serde_json::Error),
  #[error("Failed to parse preset '{1}': {0}")]
  PresetParsingFailed(quick_xml::de::DeError, String),
  #[error("Faield to generate report document: {0}")]
  ReportGenerationFailed(#[from] quick_xml::Error),
  #[error("Failed to generate report document: {0}")]
  ReportGenerationFailedUtf8(std::string::FromUtf8Error),
  #[error("Invalid preset path {}", .0.display())]
  InvalidPresetPath(PathBuf)
}

pub trait Contextualize<Context> {
  type Output;

  fn context(self, ctx: Context) -> Self::Output;
}

impl<T> Contextualize<&str> for io::Result<T> {
  type Output = Result<T, Error>;

  fn context(self, ctx: &str) -> Self::Output {
    self.map_err(|err| Error::FileError(err, ctx.to_owned()))
  }
}

impl<T> Contextualize<&Path> for io::Result<T> {
  type Output = Result<T, Error>;

  fn context(self, ctx: &Path) -> Self::Output {
    self.map_err(|err| {
      let reason = format!("Failed to access {}", ctx.display());
      Error::FileError(err, reason)
    })
  }
}

impl<T> Contextualize<&PathBuf> for io::Result<T> {
  type Output = Result<T, Error>;

  fn context(self, ctx: &PathBuf) -> Self::Output {
    self.context(Path::new(ctx))
  }
}

impl<T> Contextualize<&str> for Result<T, quick_xml::de::DeError> {
  type Output = Result<T, Error>;

  fn context(self, ctx: &str) -> Self::Output {
    self.map_err(|err| Error::PresetParsingFailed(err, ctx.to_owned()))
  }
}
