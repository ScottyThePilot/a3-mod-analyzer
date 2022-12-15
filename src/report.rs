use chrono::{DateTime, Utc};
use chrono_humanize::Humanize;
use quick_xml::Writer;
use quick_xml::events::BytesText;

use crate::analysis::AddonAnalysis;
use crate::error::Error;

use std::fmt;
use std::io::{Cursor, Write};

const REPORT_HEAD: &str = include_str!("./report/head.html");

pub fn create_report(analysis: Vec<AddonAnalysis>) -> Result<String, Error> {
  let now = Utc::now();
  let mut buffer = Vec::new();
  let mut document = Writer::new_with_indent(Cursor::new(&mut buffer), b' ', 2);
  document.create_element("table")
    .with_attribute(("class", "sortable"))
    .write_inner_content(|document_table| {
      document_table.create_element("thead")
        .write_inner_content(write_document_table_header)?;
      document_table.create_element("tbody")
        .write_inner_content(|document_table_body| {
          for addon_analysis in analysis.iter() {
            write_document_table_row(document_table_body, addon_analysis, now)?;
          };

          Ok(())
        })?;
      Ok(())
    })?;
  let contents = String::from_utf8(buffer)
    .map_err(Error::ReportGenerationFailedUtf8)?;
  let document = format!("<!DOCTYPE html>\n{}", Tag("html", {
    format_args!("{}\n{}", REPORT_HEAD.trim(), Tag("body", contents.trim()))
  }));

  Ok(document)
}

fn write_document_table_header(
  document_table: &mut Writer<impl Write>,
) -> quick_xml::Result<()> {
  document_table.create_element("tr")
    .write_inner_content(|document_table_row| {
      for &(cell_text, cell_description) in HEADER_CELLS {
        document_table_row.create_element("th")
          .with_attribute(("title", cell_description))
          .write_text_content(BytesText::new(cell_text))?;
      };

      Ok(())
    })?;
  Ok(())
}

fn write_document_table_row(
  document_table: &mut Writer<impl Write>,
  addon_analysis: &AddonAnalysis,
  now: DateTime<Utc>
) -> quick_xml::Result<()> {
  document_table.create_element("tr")
    .write_inner_content(|document_table_row| {
      let last_update = addon_analysis.last_update.signed_duration_since(now);
      let last_usage = addon_analysis.last_usage.map(|last_usage| {
        last_usage.signed_duration_since(now)
      });

      let last_update_text = last_update.humanize();
      let last_update_hover = addon_analysis.last_update.to_rfc2822();
      let last_usage_text = last_usage.map_or("Unknown".to_owned(), |lu| lu.humanize());
      let last_usage_hover = addon_analysis.last_usage.map_or("Unknown".to_owned(), |lu| lu.to_rfc2822());
      let size_on_disk_text = bytesize::to_string(addon_analysis.file_size, true);
      let presets_text = addon_analysis.preset_count.to_string();
      let dependencies_text = addon_analysis.dependency_count.to_string();
      let dependents_text = addon_analysis.dependents_count.to_string();

      let last_update_sort = last_update.num_milliseconds().to_string();
      let last_usage_sort = last_usage.map_or(0, |lu| lu.num_milliseconds()).to_string();
      let size_on_disk_sort = addon_analysis.file_size.to_string();

      document_table_row.create_element("td")
        .write_inner_content(|document_table_cell| {
          document_table_cell.create_element("a")
            .with_attribute(("href", addon_analysis.link.as_str()))
            .write_text_content(BytesText::new(&addon_analysis.name))?;
          Ok(())
        })?;
      document_table_row.create_element("td")
        .with_attribute(("data-sort", last_update_sort.as_str()))
        .with_attribute(("title", last_update_hover.as_str()))
        .write_text_content(BytesText::new(&last_update_text))?;
      document_table_row.create_element("td")
        .with_attribute(("data-sort", last_usage_sort.as_str()))
        .with_attribute(("title", last_usage_hover.as_str()))
        .write_text_content(BytesText::new(&last_usage_text))?;
      document_table_row.create_element("td")
        .with_attribute(("data-sort", size_on_disk_sort.as_str()))
        .write_text_content(BytesText::new(&size_on_disk_text))?;
      document_table_row.create_element("td")
        .write_text_content(BytesText::new(&presets_text))?;
      document_table_row.create_element("td")
        .write_text_content(BytesText::new(&dependencies_text))?;
      document_table_row.create_element("td")
        .write_text_content(BytesText::new(&dependents_text))?;
      Ok(())
    })?;
  Ok(())
}

const HEADER_CELLS: &[(&str, &str)] = &[
  ("Name", "The addon's name"),
  ("Last Update", "The last time an update was published for this addon"),
  ("Last Usage", "The last time one of your presets containing this addon was updated"),
  ("Size On Disk", "The file size of this addon on disk"),
  ("Presets", "The number of presets containing this addon"),
  ("Dependencies", "The number of addons this addon depends on"),
  ("Dependents", "The number of addons (that you're subscribed to) that depend on this addon")
];

#[derive(Debug, Clone, Copy)]
struct Tag<T>(&'static str, T);

impl<T: fmt::Display> fmt::Display for Tag<T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<{tag}>\n{body}\n</{tag}>", tag = self.0, body = self.1)
  }
}
