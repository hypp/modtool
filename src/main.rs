use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::io::BufReader;
use std::io::Read;
use std::cmp;
use std::str::FromStr;
use std::collections::BTreeMap;

extern crate modfile;
use modfile::ptmf;

#[macro_use]
extern crate serde_derive;
extern crate docopt;
extern crate rustc_serialize;

use docopt::Docopt;

// TODO Refactor this to several files
// TODO Move some of the functions to the modfile crate

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

static USAGE: &'static str = "
modtool.

Usage: 
    modtool (-h | --help)
    modtool (-V | --version)
    modtool show [--summary] [--sample-info] [--sample-stats] [--pattern-info] [--use-spn] [--in-p61] <file>...
    modtool save (--number=<number> | --all) [--in-p61] <fileprefix> <file>
    modtool convert [--unused-patterns] [--unused-samples] [--in-p61] <fileprefix> <file>...

Options:
    -V, --version         Show version info.
    -h, --help            Show this text.

    show                  Show various info and statistics.
      --summary           Show summary info.
      --sample-info       Show info about samples.
      --sample-stats      Show sample statistics.
      --pattern-info      Show info about patterns.
      --use-spn           Use scientific pitch notation where middle C is C4.
      --in-p61            Input file format is The Player 6.1A.
      <file>              File(s) to process.

    save                  Save samples, RAW 8-bit signed.
      --all               Save all samples.
      --number=<number>   Save only sample <number>.
      --in-p61            Input file format is The Player 6.1A.
      <fileprefix>        Use <fileprefix> as prefix to filenames when saving.
      <file>              File to process.

    convert               Remove unused/samples and or patterns. 
                          Can also convert from The Player to ProTracker.
                          Including 8-bit and 4-bit delta packed samples.
      --unused-patterns   Remove unused patterns.
      --unused-samples    Remove unused samples. 
      --in-p61            Input file format is The Player 6.1A.
      <fileprefix>        Use <fileprefix> as prefix to filenames when saving.
      <file>              File(s) to process.
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_file: Vec<String>,
    flag_help: bool,
	flag_version: bool,
	
	// Common for all sub commands
	flag_in_p61: bool,

	cmd_show: bool,
	flag_summary: bool,
    flag_sample_info: bool,
	flag_sample_stats: bool,
	flag_pattern_info: bool,
	flag_use_spn: bool,
	
	cmd_save: bool,
	flag_all: bool,
	flag_number: String,
	arg_fileprefix: String,
	
	cmd_convert: bool,
	flag_unused_patterns: bool,
	flag_unused_samples: bool,
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
	
	let unused = find_unused_samples(module);
	
	println!("Sample statistics");
	println!("\tLength min: {} max: {} avg: {}", len.min, len.max, len.avg);
	println!("\tFinetune min: {} max: {} avg: {}", finetune.min, finetune.max, finetune.avg);
	println!("\tVolume min: {} max: {} avg: {}", volume.min, volume.max, volume.avg);
	println!("\tRepeat start min: {} max: {} avg: {}", repeat_start.min, repeat_start.max, repeat_start.avg);
	println!("\tRepeat length min: {} max: {} avg: {}", repeat_length.min, repeat_length.max, repeat_length.avg);
	print!("\tUnused samples: ");
	for i in unused {
		print!("{} ", i);
	}
	println!("");
	println!("");
}

fn find_unused_patterns(module: &ptmf::PTModule) -> Vec<u8> {
	let mut unused:Vec<u8> = Vec::new();
	let positions = &module.positions.data[0..module.length as usize];
	let num_patterns = module.patterns.len();
	for i in 0..num_patterns as u8 {
		if !positions.contains(&i) {
			unused.push(i);
		}
	}

	unused
}

fn show_pattern_info(module: &ptmf::PTModule, use_spn: bool) {

	println!("Pattern info");
	print!("\tPattern play order: ");
	let positions = &module.positions.data[0..module.length as usize];
	for pos in positions.iter() {
		print!("{} ",pos);
	}
	println!("");
	
	print!("\tUnused patterns: ");
	let unused = find_unused_patterns(module);
	for i in unused {
		print!("{} ",i);
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
	
	println!("\tUsed periods: ");
	let mut map = BTreeMap::<u16,u16>::new();
	for pattern_no in 0..module.patterns.len() {
		let ref pattern = module.patterns[pattern_no];
		for row_no in 0..pattern.rows.len() {
			let ref row = pattern.rows[row_no];
			for channel_no in 0..row.channels.len() {
				let ref channel = row.channels[channel_no];
				if channel.period > 0 {
					let count = map.entry(channel.period).or_insert(0);
					*count += 1;
				}
			}
		}
	}
	for key in map.keys() {
		// Find the position in PERIODS with the
		// smallest difference
		let mut found:i32 = -1;
		let mut min_diff = 65536;
		let key = *key as i32;
		for i in 0..ptmf::PERIODS.len() {
			let diff = (key as i32 - ptmf::PERIODS[i] as i32).abs();
			if diff < min_diff {
				min_diff = diff;
				found = i as i32;
			}
		}
		
		let note = if found == -1 {
			println!("Failed to find note name");
			String::new()
		} else {
			let mut octave = found / 12;
			if use_spn {
				octave += 2;
			}
			let note = (found % 12) as usize;
			let prefix = match min_diff {
				0 => "",
				_ => "~"
			};
			format!("{}{}-{}",prefix,ptmf::NOTE_NAMES[note],octave)
		};
		
		println!("\t {}({}) ",key,note);
	}
//	println!("");
	
	println!("\tUsed effects: ");
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
			let name = ptmf::EFFECT_NAMES[i];
			println!("\t {}",name);
			
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
	
	println!("\tThe Player usecode: ${:X}",usecode);
	println!("");
}

fn save_samples(module: &ptmf::PTModule,range: &Vec<usize>,prefix: &String) {
	for i in range {	
		let filename = format!("{}_{}.raw",prefix,i+1);
	
		let file = match File::create(&filename) {
			Ok(file) => file,
			Err(e) => {
				println!("Failed to open file: '{}' Error: '{:?}'", filename, e);
				return
			}
		};

		let mut writer = BufWriter::new(&file);		
		match writer.write_all(&module.sample_info[*i].data) {
			Ok(_) => (),
			Err(e) => {
				println!("Failed to write sample {}. Error: '{:?}'", i, e);
			}
		}
	}
}

fn remove_unused_patterns(module: &mut ptmf::PTModule) {
	let mut unused = find_unused_patterns(module);
	unused.reverse();
	
	// MUST Remove highest pattern first
	for i in unused {
		// Remove pattern
		module.patterns.remove(i as usize);
	
		// Adjust play positions
		for j in 0..module.length {
			let j = j as usize;
			if module.positions.data[j] > i {
				module.positions.data[j] -= 1;
			}
		}
	}
}	

fn find_unused_samples(module: &ptmf::PTModule) -> Vec<u8> {
	let mut unused:Vec<u8> = Vec::new();
	let mut used = [0u8;32];

	// Find all used samples
	for pattern_no in 0..module.patterns.len() {
		let ref pattern = module.patterns[pattern_no];
		for row_no in 0..pattern.rows.len() {
			let ref row = pattern.rows[row_no];
			for channel_no in 0..row.channels.len() {
				let ref channel = row.channels[channel_no];
				let number = channel.sample_number as usize;
				if number > 0 {
					if number > 31 {
						println!("Error: Invalid sample number in Pattern '{}' Row '{}' Channel '{}' Sample number '{}'",pattern_no,row_no,channel_no,number);
					} else {
						used[number] = 1;
					}
				}				
			}
		}
	}

	// Find all unused samples
	for i in 1..module.sample_info.len()+1 {
		if used[i] == 0 {
			unused.push(i as u8);
		}
	}
	
	unused
}

fn remove_unused_samples(module: &mut ptmf::PTModule) {
	let mut unused = find_unused_samples(module);
	unused.reverse();

	// MUST remove highest sample first
	for i in unused {
		let index = i as usize - 1;
		
		// Remove sample info and put it last
		let mut si = module.sample_info.remove(index);
				
		si.length = 0;
		si.repeat_start = 0;
		si.repeat_length = 0;
		si.data.clear();
		module.sample_info.push(si);
	
		// Rewrite instrument references
		// TODO optimize this
		for pattern in &mut module.patterns {
			for row in &mut pattern.rows {
				for channel in &mut row.channels {
					let number = channel.sample_number;
					if number > i {
						channel.sample_number -= 1;
					}				
				}
			}
		}
	}
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
	
	if args.flag_number.len() > 0 {
		let number = usize::from_str(&args.flag_number).unwrap();
		if number < 1 || number > 31 {
			println!("Invalid sample number '{}'", number);
			return;
		}
	}

	let p61 = args.flag_in_p61;
	let read_fn:fn (&mut Read) -> Result<ptmf::PTModule, ptmf::PTMFError> = 
		if p61 {
			ptmf::read_p61
		} else {
			ptmf::read_mod
		};
		
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
			let module = match read_fn(&mut reader) {
				Ok(module) => module,
				Err(e) => {
					println!("Failed to parse file: '{}' Error: '{:?}'", filename, e);
					continue
				}
			};
			
			println!("Processing: {}", filename);
				
			if args.flag_summary {
				show_summary(&module);
			}
			
			if args.flag_sample_info {
				show_sample_info(&module);
			}
			
			if args.flag_sample_stats {
				show_sample_stats(&module);
			}
			
			if args.flag_pattern_info {
				show_pattern_info(&module, args.flag_use_spn);
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
		let module = match read_fn(&mut reader) {
			Ok(module) => module,
			Err(e) => {
				println!("Failed to parse file: '{}' Error: '{:?}'", filename, e);
				return
			}
		};

		println!("Processing: {}", filename);
		
		let range = if args.flag_all {
			0..module.sample_info.len()
		} else {
			let number = usize::from_str(&args.flag_number).unwrap() - 1;
			if number >= module.sample_info.len() {
				println!("Invalid sample number. Only {} samples available.", module.sample_info.len());
				return
			}
			number..number+1
		};
		
		save_samples(&module,&(range.collect()),&args.arg_fileprefix);
	} else if args.cmd_convert {
		for ref filename in args.arg_file {
			let file = match File::open(filename) {
				Ok(file) => file,
				Err(e) => {
					println!("Failed to open file: '{}' Error: '{}'", filename, e);
					continue
				}
			};
			
			let mut reader = BufReader::new(&file);
			let mut module = match read_fn(&mut reader) {
				Ok(module) => module,
				Err(e) => {
					println!("Failed to parse file: '{}' Error: '{:?}'", filename, e);
					continue
				}
			};
			
			println!("Processing: {}", filename);
			
			if args.flag_unused_patterns {
				remove_unused_patterns(&mut module);
			}
			
			if args.flag_unused_samples {
				remove_unused_samples(&mut module);
			}
			
			let filename = format!("{}_{}",args.arg_fileprefix,filename);
		
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
	}
	
	println!("Done!");
}
