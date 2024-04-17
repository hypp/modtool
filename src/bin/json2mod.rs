use std::fs::File;
use std::io::BufWriter;
use std::io::BufReader;
use anyhow::{Context, Result, anyhow};
// Command line
use docopt::Docopt;
// JSON
use serde::Deserialize;

// ProTracker and ThePlayer
use modfile::ptmf;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

static USAGE: &'static str = "
json2mod.

Usage: 
    json2mod (-h | --help)
    json2mod (-V | --version)
    json2mod <source> <destination>

Options:
    -V, --version         Show version info.
    -h, --help            Show this text.

    <source>              Input file.
    <destination>         Output file.
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_source: String,
    arg_destination: String,
	flag_version: bool,
}

fn main() -> Result<()> {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.deserialize())
                            .unwrap_or_else(|e| e.exit());
	
	if args.flag_version {
		println!("Version: {}", VERSION);
		return Ok(());
	}

	// Open json file
	let ref first_filename = args.arg_source;
	let file = File::open(first_filename)
				.with_context(|| format!("Failed to open file: '{}'", first_filename))?;
	
	let reader = BufReader::new(&file);
	let mut module: ptmf::PTModule = serde_json::from_reader(reader)
					.with_context(|| format!("Failed to parse file: '{}'", first_filename))?;


	let ref filename = args.arg_destination;
	let file = File::create(&filename)
				.with_context(|| format!("Failed to open file: '{}'", filename))?;


	let mut writer = BufWriter::new(&file);		
	match ptmf::write_mod(&mut writer,&mut module) {
		Ok(_) => (),
		Err(e) => {
			return Err(anyhow!("Failed to write module {}. Error: '{:?}'", filename, e))
		}
	}

	Ok(())
}
