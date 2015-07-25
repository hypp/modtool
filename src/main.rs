use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::io::BufReader;
use std::cmp;
use std::str::FromStr;

extern crate modfile;
use modfile::ptmf;

extern crate rustc_serialize;
extern crate docopt;
use docopt::Docopt;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

static USAGE: &'static str = "
modtool.

Usage: 
    modtool (-h | --help)
    modtool (-V | --version)
    modtool show [--show-summary] [--show-sample-info] [--show-sample-stats] [--show-pattern-info] <file>...
    modtool save <number> <filename> <file>

Options:
    -V, --version         Show version info.
    -h, --help            Show this text.

    --show-summary        Show summary info.
    --show-sample-info    Show info about samples.
    --show-sample-stats   Show sample statistics.
    --show-pattern-info   Show info about patterns.

    <number>              Select sample NUMBER to save.
    <filename>            Save selected sample to FILENAME.
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_file: Vec<String>,
    flag_help: bool,
	flag_version: bool,
	
	cmd_show: bool,
	flag_show_summary: bool,
    flag_show_sample_info: bool,
	flag_show_sample_stats: bool,
	flag_show_pattern_info: bool,
	
	cmd_save: bool,
	arg_number: String,
	arg_filename: String
}

#[derive(Debug)]
struct Stats {
	pub min: u32,
	pub max: u32,
	pub sum: usize,
	pub avg: usize,
	pub num_values: u32
}

impl Stats {
	pub fn new() -> Stats {
		Stats{min: u32::max_value(), max: u32::min_value(), sum: 0, avg: 0, num_values: 0}
	}

	pub fn update(&mut self, val: u32) {
		self.min = cmp::min(self.min, val);
		self.max = cmp::max(self.max, val);
		self.sum += val as usize;
		
		self.num_values += 1;
	}
	
	pub fn done(&mut self) {
		self.avg = self.sum / (self.num_values as usize);
	}
}

fn show_summary(module: &ptmf::PTModule) {
	println!("Song summary");
	println!("\tSongname: {}", module.name);
	println!("\tLength: {}", module.length);
	let used_samples = module.sample_info.iter().filter(|si| si.length > 0);
	println!("\tNumber of samples with length > 0: {}", used_samples.count());
	println!("\tNumber of patterns: {}", module.patterns.len());
	println!("");
}

fn show_sample_info(module: &ptmf::PTModule) {
	let mut number = 1;
	for sample in module.sample_info.iter() {
		println!("Sample number {} details", number);
		println!("\tName: {}", sample.name);
		println!("\tLength: {}b", sample.length * 2);
		println!("\tFinetune: {}", sample.finetune);
		println!("\tVolume: {}", sample.volume);
		println!("\tRepeat start: {}b", sample.repeat_start * 2);
		println!("\tRepeat length: {}b", sample.repeat_length * 2);
		number += 1;
	}
	println!("");
}

fn show_sample_stats(module: &ptmf::PTModule) {
	let used_samples = module.sample_info.iter().filter(|si| si.length > 0);
	
	let mut len = Stats::new();
	let mut finetune = Stats::new();
	let mut volume = Stats::new();
	let mut repeat_start = Stats::new();
	let mut repeat_length = Stats::new();

	for sample in used_samples {
		let slen = sample.length as u32 * 2;
		let sfinetune = sample.finetune as u32;
		let svolume = sample.volume as u32;
		let srepeat_start = sample.repeat_start as u32 * 2;
		let srepeat_length = sample.repeat_length as u32 * 2;
	
		len.update(slen);
		finetune.update(sfinetune);
		volume.update(svolume);
		repeat_start.update(srepeat_start);
		repeat_length.update(srepeat_length);
	}
	
	len.done();
	finetune.done();
	volume.done();
	repeat_start.done();
	repeat_length.done();
	
	println!("Sample statistics");
	println!("\tLength min: {} max: {} avg: {}", len.min, len.max, len.avg);
	println!("\tFinetune min: {} max: {} avg: {}", finetune.min, finetune.max, finetune.avg);
	println!("\tVolume min: {} max: {} avg: {}", volume.min, volume.max, volume.avg);
	println!("\tRepeat start min: {} max: {} avg: {}", repeat_start.min, repeat_start.max, repeat_start.avg);
	println!("\tRepeat length min: {} max: {} avg: {}", repeat_length.min, repeat_length.max, repeat_length.avg);
	println!("");
}

fn show_pattern_info(module: &ptmf::PTModule) {

	println!("Pattern info");
	print!("\tPattern play order: ");
	let positions = &module.positions.data[0..module.length as usize];
	for pos in positions.iter() {
		print!("{} ",pos);
	}
	println!("");
	
	print!("\tUnused patterns: ");
	let num_patterns = module.patterns.len();
	for i in 0..num_patterns as u8 {
		if !positions.contains(&i) {
			print!("{} ",i);
		}
	}
	println!("");
	
	print!("\tEmpty patterns: ");
	for i in 0..module.patterns.len() {
		let mut empty = true;
		for row in &module.patterns[i].rows {
			for channel in &row.channels {
				if channel.period != 0 ||
					channel.sample_number != 0 ||
					channel.effect != 0 {
					empty = false;
				}
				if !empty {
					break;
				}
			}
			if !empty {
				break;
			}
		}
		if empty {
			print!("{} ",i);
		}
	}
	println!("");
}



fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
//    println!("{:?}", args);	
	
	if args.flag_version {
		println!("Version: {}", VERSION);
		return;
	}
	
	if args.arg_number.len() > 0 {
		let number = usize::from_str(&args.arg_number).unwrap();
		if number < 1 || number > 31 {
			println!("Invalid sample number '{}'", number);
			return;
		}
	}

	if args.cmd_show {
		for ref filename in args.arg_file {
			let file = match File::open(filename) {
				Ok(file) => file,
				Err(e) => {
					println!("Failed to open file: '{}' Error: '{}'", filename, e);
					continue
				}
			};
			
			let mut reader = BufReader::new(&file);
			let module = match ptmf::read_mod(&mut reader) {
				Ok(module) => module,
				Err(e) => {
					println!("Failed to parse file: '{}' Error: '{:?}'", filename, e);
					continue
				}
			};
			
			println!("Processing: {}", filename);
				
			if args.flag_show_summary {
				show_summary(&module);
			}
			
			if args.flag_show_sample_info {
				show_sample_info(&module);
			}
			
			if args.flag_show_sample_stats {
				show_sample_stats(&module);
			}
			
			if args.flag_show_pattern_info {
				show_pattern_info(&module);
			}
		}
	} else if args.cmd_save {
		let ref filename = args.arg_file[0];
		let file = match File::open(filename) {
			Ok(file) => file,
			Err(e) => {
				println!("Failed to open file: '{}' Error: '{}'", filename, e);
				return
			}
		};
		
		let mut reader = BufReader::new(&file);
		let module = match ptmf::read_mod(&mut reader) {
			Ok(module) => module,
			Err(e) => {
				println!("Failed to parse file: '{}' Error: '{:?}'", filename, e);
				return
			}
		};

		println!("Processing: {}", filename);

		let ref filename = args.arg_filename;
		let file = match File::create(filename) {
			Ok(file) => file,
			Err(e) => {
				println!("Failed to open file: '{}' Error: '{:?}'", filename, e);
				return
			}
		};

		let number = usize::from_str(&args.arg_number).unwrap() - 1;
		if number >= module.sample_data.len() {
			println!("Invalid sample number. Only {} samples available.", module.sample_data.len());
			return
		}
		
		let mut writer = BufWriter::new(&file);		
		match writer.write_all(&module.sample_data[number]) {
			Ok(_) => (),
			Err(e) => {
				println!("Failed to write sample. Error: '{:?}'", e);
				return
			}
		}
	}
	
	println!("Done!");
}
