//! AGAIN, I DO NOT OWN THIS CODE

//! A command line interface (CLI) tool to format [JSON5](https://json5.org) ("JSON for
//! Humans") documents to a consistent style, preserving comments.
//!
//! # Usage
//!
//!     formatjson5 [FLAGS] [OPTIONS] [files]...
//!
//!     FLAGS:
//!     -h, --help                  Prints help information
//!     -n, --no_trailing_commas    Suppress trailing commas (otherwise added by default)
//!     -o, --one_element_lines     Objects or arrays with a single child should collapse to a
//!                                 single line; no trailing comma
//!     -r, --replace               Replace (overwrite) the input file with the formatted result
//!     -s, --sort_arrays           Sort arrays of primitive values (string, number, boolean, or
//!                                 null) lexicographically
//!     -V, --version               Prints version information
//!
//!     OPTIONS:
//!     -i, --indent <indent>    Indent by the given number of spaces [default: 4]
//!
//!     ARGS:
//!     <files>...    Files to format (use "-" for stdin)

#![warn(missing_docs)]

use anyhow::{self, Result};
use json5format::*;
use std::{
  fs, io,
  io::{Read, Write},
  path::PathBuf,
};
use structopt::StructOpt;

/// Parses each file in the given `files` vector and returns a parsed object for each JSON5
/// document. If the parser encounters an error in any input file, the command aborts without
/// formatting any of the documents.
fn parse_documents(
  files: Vec<PathBuf>,
) -> Result<Vec<ParsedDocument>, anyhow::Error> {
  let mut parsed_documents = Vec::with_capacity(files.len());
  for file in files {
    let filename = file.clone().into_os_string().to_string_lossy().to_string();
    let mut buffer = String::new();
    if filename == "-" {
      Opt::from_stdin(&mut buffer)?;
    } else {
      fs::File::open(&file)?.read_to_string(&mut buffer)?;
    }

    parsed_documents.push(ParsedDocument::from_string(buffer, Some(filename))?);
  }
  Ok(parsed_documents)
}

/// Formats the given parsed documents, applying the given format `options`. If `replace` is true,
/// each input file is overwritten by its formatted version.
fn format_documents(
  parsed_documents: Vec<ParsedDocument>,
  options: FormatOptions,
  replace: bool,
) -> Result<(), anyhow::Error> {
  let format = Json5Format::with_options(options)?;
  for (index, parsed_document) in parsed_documents.iter().enumerate() {
    let filename = parsed_document.filename().as_ref().unwrap();
    let bytes = format.to_utf8(&parsed_document)?;
    if replace {
      Opt::write_to_file(filename, &bytes)?;
    } else {
      if index > 0 {
        println!();
      }
      if parsed_documents.len() > 1 {
        println!("{}:", filename);
        println!("{}", "=".repeat(filename.len()));
      }
      print!("{}", std::str::from_utf8(&bytes)?);
    }
  }
  Ok(())
}

/// The entry point for the [formatjson5](index.html) command line interface.
fn main() -> Result<()> {
  let args = Opt::args();

  if args.files.len() == 0 {
    return Err(anyhow::anyhow!("No files to format"));
  }

  let parsed_documents = parse_documents(args.files)?;

  let options = FormatOptions {
    indent_by: args.indent,
    trailing_commas: !args.no_trailing_commas,
    collapse_containers_of_one: args.one_element_lines,
    sort_array_items: args.sort_arrays,
    ..Default::default()
  };

  format_documents(parsed_documents, options, args.replace)
}

/// Command line options defined via the structopt! macrorule. These definitions generate the
/// option parsing, validation, and [usage documentation](index.html).
#[derive(Debug, StructOpt)]
#[structopt(
  name = "json5format",
  about = "Format JSON5 documents to a consistent style, preserving comments."
)]
struct Opt {
  /// Files to format (use "-" for stdin)
  #[structopt(parse(from_os_str))]
  files: Vec<PathBuf>,

  /// Replace (overwrite) the input file with the formatted result
  #[structopt(short, long)]
  replace: bool,

  /// Suppress trailing commas (otherwise added by default)
  #[structopt(short, long)]
  no_trailing_commas: bool,

  /// Objects or arrays with a single child should collapse to a single line; no trailing comma
  #[structopt(short, long)]
  one_element_lines: bool,

  /// Sort arrays of primitive values (string, number, boolean, or null) lexicographically
  #[structopt(short, long)]
  sort_arrays: bool,

  /// Indent by the given number of spaces
  #[structopt(short, long, default_value = "4")]
  indent: usize,
}

#[cfg(not(test))]
impl Opt {
  fn args() -> Self {
    Self::from_args()
  }

  fn from_stdin(mut buf: &mut String) -> Result<usize, io::Error> {
    io::stdin().read_to_string(&mut buf)
  }

  fn write_to_file(filename: &str, bytes: &[u8]) -> Result<(), io::Error> {
    fs::OpenOptions::new()
      .create(true)
      .truncate(true)
      .write(true)
      .open(filename)?
      .write_all(&bytes)
  }
}
