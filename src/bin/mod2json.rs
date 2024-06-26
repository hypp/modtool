use std::fs::File;
use std::io::BufWriter;
use std::io::BufReader;
use anyhow::{Context, Result, anyhow};
use std::io::Read;
// Command line
use docopt::Docopt;
// JSON
use serde::{Serialize, Deserialize};

// ProTracker and ThePlayer
use modfile::ptmf;
// Pretty printing of JSON
use modtool::pretty::PrettyFormatter2;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

static USAGE: &'static str = "
mod2json.

Usage: 
    mod2json (-h | --help)
    mod2json (-V | --version)
    mod2json [--in-p61] [--skip-filesize-check] <source> <destination>

Options:
    -V, --version          Show version info.
    -h, --help             Show this text.
    --in-p61               Input file format is The Player 6.1A.
    --skip-filesize-check  Skip check if all data has been parsed.

    <source>               Input file.
    <destination>          Output file.
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_source: String,
    arg_destination: String,
	flag_version: bool,
	flag_in_p61: bool,
	flag_skip_filesize_check: bool,
}

fn main() -> Result<()> {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.deserialize())
                            .unwrap_or_else(|e| e.exit());
	
	if args.flag_version {
		println!("Version: {}", VERSION);
		return Ok(());
	}
	
	fn mod_fn_true(reader: &mut dyn Read) -> Result<ptmf::PTModule, ptmf::PTMFError> {
		return ptmf::read_mod(reader, true);
	}

	fn mod_fn_false(reader: &mut dyn Read) -> Result<ptmf::PTModule, ptmf::PTMFError> {
		return ptmf::read_mod(reader, false);
	}

	let skip_file_size_check = args.flag_skip_filesize_check;

	let p61 = args.flag_in_p61;
	let read_fn:fn (&mut dyn Read) -> Result<ptmf::PTModule, ptmf::PTMFError> = 
		if p61 {
			ptmf::read_p61
		} else {
			if skip_file_size_check {
				mod_fn_true
			} else {
				mod_fn_false
			}
		};
		
	// Open the module
	let ref first_filename = args.arg_source;
	let file = File::open(first_filename)
		.with_context(|| format!("Failed to open file: '{}'", first_filename))?;
	
	let mut reader = BufReader::new(&file);
	let module = match read_fn(&mut reader) {
		Ok(module) => module,
		Err(e) => {
			return Err(anyhow!("Failed to parse file: '{}' Error: '{:?}'", first_filename, e))
		}
	};

	// Close file
	drop(file);

	let ref filename = args.arg_destination;
	let file = File::create(&filename)
		.with_context(|| format!("Failed to open file: '{}'", filename))?;

	let writer = BufWriter::new(&file);
	let format = PrettyFormatter2::default();
	let mut out = serde_json::Serializer::with_formatter(writer, format);		
	module.serialize(&mut out)
		.with_context(|| format!("Failed to serialize module to file: '{}'", filename))?;

	Ok(())
}
