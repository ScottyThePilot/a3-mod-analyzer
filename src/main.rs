extern crate bytesize;
extern crate chrono;
extern crate chrono_humanize;
extern crate dirs;
extern crate open;
extern crate quick_xml;
extern crate rfd;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate thiserror;

mod addons;
mod analysis;
mod error;
mod presets;
mod report;

use rfd::{MessageDialog, MessageLevel, MessageButtons};

use crate::error::{Error, Contextualize};

use std::path::PathBuf;

fn main() {
  match run() {
    Ok(()) => match open::that("report.html") {
      Ok(()) => (),
      Err(err) => application_error({
        Error::FileError(err, "Failed to open report.html in browser".to_string())
      })
    },
    Err(err) => application_error(err)
  }
}

fn run() -> Result<(), Error> {
  let addons = crate::addons::get_addons()?;
  let presets = crate::presets::get_presets()?;
  let analysis = crate::analysis::perform_analysis(&addons, &presets);
  let report = crate::report::create_report(analysis)?;
  std::fs::write("report.html", report)
    .context("Failed to save report.html")?;
  Ok(())
}

fn application_error(err: impl std::error::Error) -> ! {
  MessageDialog::new()
    .set_title("Error")
    .set_description(&err.to_string())
    .set_level(MessageLevel::Error)
    .set_buttons(MessageButtons::Ok)
    .show();
  panic!("{err}");
}

fn launcher_dir() -> Result<PathBuf, Error> {
  let path = dirs::data_local_dir()
    .ok_or(Error::NoLauncherDirFound)?
    .join("Arma 3 Launcher");
  Ok(path)
}
