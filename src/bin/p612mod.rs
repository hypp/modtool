use std::fs::File;
use std::io::BufWriter;
use std::io::BufReader;
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

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.deserialize())
                            .unwrap_or_else(|e| e.exit());
//    println!("{:?}", args);	
	
	if args.flag_version {
		println!("Version: {}", VERSION);
		return;
	}
	
	let read_fn:fn (&mut dyn Read) -> Result<ptmf::PTModule, ptmf::PTMFError> = ptmf::read_p61;
		
	let ref filename = args.arg_source;
	let file = match File::open(filename) {
		Ok(file) => file,
		Err(e) => {
			println!("Failed to open file: '{}' Error: '{}'", filename, e);
			return
		}
	};
		
	let mut reader = BufReader::new(&file);
	let mut module = match read_fn(&mut reader) {
		Ok(module) => module,
		Err(e) => {
			println!("Failed to parse file: '{}' Error: '{:?}'", filename, e);
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

}
