use std::fs::File;
use std::io::BufWriter;
use std::io::BufReader;
use anyhow::{Context, Result, anyhow};
use std::io::Read;
use std::io::Write;
use std::io::Cursor;

// Command line
use docopt::Docopt;
// JSON
use serde::{Deserialize};

// ProTracker and ThePlayer
use modfile::ptmf;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

static USAGE: &'static str = "
mod2p61.

Usage: 
    mod2p61 (-h | --help)
    mod2p61 (-V | --version)
    mod2p61 [--skip-filesize-check] [--sample-file=<samplefile>] <source> <destination>

Options:
    -V, --version          Show version info.
    -h, --help             Show this text.
    --skip-filesize-check  Skip check if all data has been parsed.
    --sample-file=<samplefile>  Write sample data to separate file <samplefile>

    <source>               Input file.
    <destination>          Output file.
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_source: String,
    arg_destination: String,
	flag_version: bool,
	flag_skip_filesize_check: bool,
	flag_sample_file: String
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

	let read_fn:fn (&mut dyn Read) -> Result<ptmf::PTModule, ptmf::PTMFError> = 
		if skip_file_size_check {
			mod_fn_true
		} else {
			mod_fn_false
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

	// Messy way of getting The Player use code
	let mut p61data = Vec::new();
	let mut p61stream = Cursor::new(&mut p61data);

	let mut p61samples = Vec::new();
	let mut p61samplestream = Cursor::new(&mut p61samples);

	match ptmf::write_p61(&mut p61stream, Option::Some(&mut p61samplestream), &module) {
		Ok(_) => (),
		Err(e) => {
			return Err(anyhow!("Failed to convert module. Error: '{:?}'",e))
		}
	}

	// read_p61 must have samples and data in the same buffer
	let mut p61alldata: Vec<u8> = Vec::new();
	for b in &p61data
	{
		p61alldata.push(*b)
	};
	for b in &p61samples
	{
		p61alldata.push(*b)
	};
	let mut p61alldatastream = Cursor::new(&mut p61alldata);

	let p61module = match ptmf::read_p61(&mut p61alldatastream) {
		Ok(module) => module,
		Err(e) => {
			return Err(anyhow!("Failed to convert data. Error: '{:?}'",e))
		}
	};

	let usecode = get_usecode(&p61module);

	let ref filename = args.arg_destination;
	let file = File::create(&filename)
		.with_context(|| format!("Failed to open file: '{}'", filename))?;

	let mut writer = BufWriter::new(&file);
	writer.write_all(&p61data)
		.with_context(|| format!("Failed to write module {}", filename))?;

	if args.flag_sample_file.len() <= 0 
	{
		// One file for all data
		writer.write_all(&p61samples)
		.with_context(|| format!("Failed to write module {}", filename))?;
	}
	else
	{
		// Separate file for samples
		let ref filename = args.flag_sample_file;
		let file = File::create(&filename)
			.with_context(|| format!("Failed to open file: '{}'", filename))?;

		let mut writer = BufWriter::new(&file);
		writer.write_all(&p61samples)
		.with_context(|| format!("Failed to write samples {}", filename))?;
	} 

	println!("Usecode: ${:08x}",usecode);
	Ok(())
}

/// Gets The Player usecode
fn get_usecode(module: &ptmf::PTModule) -> u32 {

	let mut effects = [false; 32]; // 32 effects
	for pattern_no in 0..module.patterns.len() {
		let ref pattern = module.patterns[pattern_no];
		for row_no in 0..pattern.rows.len() {
			let ref row = pattern.rows[row_no];
			for channel_no in 0..row.channels.len() {
				let ref channel = row.channels[channel_no];
				let mut effect = (channel.effect & 0x0f00) >> 8;
				if effect == 0 {
					if channel.effect & 0x00ff == 0 {
						// Not really an effect
						continue;
					}
				}
				if effect == 0xe {
					effect = ((channel.effect & 0x00f0) >> 4) + 16;
				}
				effects[effect as usize] = true;
			}
		}
	}

	let mut usecode:u32 = 0;
	for i in 0..effects.len() {
		if effects[i] {
			// Figure out the player usecode
			// Some have special handling
			if i == 0 {
				// 
				usecode |= 1 << 8; // The player converts 0 to 8
			} else {
				usecode |= 1 << i;
			} 
		}
	}
	
	// Check if finetune is used
	for si in module.sample_info.iter() {
		if si.finetune != 0 {
			usecode |= 1;
			break;
		}
	}

	usecode
}