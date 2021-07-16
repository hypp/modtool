use std::fs::File;
use std::io::BufWriter;
use std::io::BufReader;
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
    flag_help: bool,
	flag_version: bool,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.deserialize())
                            .unwrap_or_else(|e| e.exit());
//    println!("{:?}", args);	
	
	if args.flag_version {
		println!("Version: {}", VERSION);
		return;
	}

	// Open json file
	let ref first_filename = args.arg_source;
	let file = match File::open(first_filename) {
		Ok(file) => file,
		Err(e) => {
			println!("Failed to open file: '{}' Error: '{}'", first_filename, e);
			return
		}
	};
	
	let reader = BufReader::new(&file);
	let mut module: ptmf::PTModule = match serde_json::from_reader(reader) {
		Ok(module) => module,
		Err(e) => {
			println!("Failed to parse file: '{}' Error: '{:?}'", first_filename, e);
			return
		}
	};

	let ref filename = args.arg_destination;
	let file = match File::create(&filename) {
		Ok(file) => file,
		Err(e) => {
			println!("Failed to open file: '{}' Error: '{:?}'", filename, e);
			return
		}
	};

	let mut writer = BufWriter::new(&file);		
	match ptmf::write_mod(&mut writer,&mut module) {
		Ok(_) => (),
		Err(e) => {
			println!("Failed to write module {}. Error: '{:?}'", filename, e);
		}
	}

	println!("Done!");
}
