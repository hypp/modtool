use std::fs::File;
use std::io::BufWriter;
use std::io::BufReader;
use anyhow::{Context, Result, anyhow};
use std::io::Read;
// Command line
use docopt::Docopt;
// JSON
use serde::Deserialize;

// ProTracker and ThePlayer
use modfile::ptmf;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

static USAGE: &'static str = "
p612mod.

Usage: 
    p612mod (-h | --help)
    p612mod (-V | --version)
    p612mod <source> <destination>

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
    flag_help: bool,
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
	
	let read_fn:fn (&mut dyn Read) -> Result<ptmf::PTModule, ptmf::PTMFError> = ptmf::read_p61;
		
	let ref filename = args.arg_source;
	let file = File::open(filename)
			.with_context(|| format!("Failed to open file: '{}'", filename))?;

		
	let mut reader = BufReader::new(&file);
	let mut module = match read_fn(&mut reader) {
		Ok(module) => module,
		Err(e) => {
			return Err(anyhow!("Failed to parse file: '{}' Error: '{:?}'", filename, e))
		}
	};
		
	let ref filename = args.arg_destination;
	let file = File::create(&filename)
		.with_context(|| format!("Failed to open file: '{}'", filename))?;

	let mut writer = BufWriter::new(&file);		
	match ptmf::write_mod(&mut writer,&mut module) {
		Ok(_) => (),
		Err(e) => {
			return Err(anyhow!("Failed to write module : '{}' Error: '{:?}'", filename, e))
		}
	}

	Ok(())
}
