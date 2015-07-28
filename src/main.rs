use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::io::BufReader;
use std::cmp;
use std::str::FromStr;
use std::collections::BTreeMap;

extern crate modfile;
use modfile::ptmf;

extern crate rustc_serialize;
extern crate docopt;
use docopt::Docopt;

// TODO Refactor this to several files
// TODO Move some of the functions to the modfile crate

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

static USAGE: &'static str = "
modtool.

Usage: 
    modtool (-h | --help)
    modtool (-V | --version)
    modtool show [--summary] [--sample-info] [--sample-stats] [--pattern-info] [--use-spn] <file>...
    modtool save (--number=<number> | --all) <fileprefix> <file>
    modtool remove [--unused-patterns] [--unused-samples] <fileprefix> <file>...

Options:
    -V, --version         Show version info.
    -h, --help            Show this text.

    show                  Show various info and statistics.
      --summary           Show summary info.
      --sample-info       Show info about samples.
      --sample-stats      Show sample statistics.
      --pattern-info      Show info about patterns.
      --use-spn           Use scientific pitch notation where middle C is C4.
      <file>              File(s) to process.

    save                  Save samples, RAW 8-bit signed.
      --all               Save all samples.
      --number=<number>   Save only sample <number>.
      <fileprefix>        Use <fileprefix> as prefix to filenames when saving.
      <file>              File to process.

    remove                Remove unused/samples and or patterns.
      --unused-patterns   Remove unused patterns.
      --unused-samples    Remove unused samples. 
      <fileprefix>        Use <fileprefix> as prefix to filenames when saving.
      <file>              File(s) to process.
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_file: Vec<String>,
    flag_help: bool,
	flag_version: bool,
	
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
	
	cmd_remove: bool,
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

///
/// Periods from http://greg-kennedy.com/tracker/modformat.html
///
///          C    C#   D    D#   E    F    F#   G    G#   A    A#   B
/// Octave 1: 856, 808, 762, 720, 678, 640, 604, 570, 538, 508, 480, 453
/// Octave 2: 428, 404, 381, 360, 339, 320, 302, 285, 269, 254, 240, 226
/// Octave 3: 214, 202, 190, 180, 170, 160, 151, 143, 135, 127, 120, 113
///
/// Octave 0:1712,1616,1525,1440,1357,1281,1209,1141,1077,1017, 961, 907
/// Octave 4: 107, 101,  95,  90,  85,  80,  76,  71,  67,  64,  60,  57
///
static PERIODS: &'static [u16] = &[
    1712,1616,1525,1440,1357,1281,1209,1141,1077,1017, 961, 907,
    856,  808, 762, 720, 678, 640, 604, 570, 538, 508, 480, 453,
    428,  404, 381, 360, 339, 320, 302, 285, 269, 254, 240, 226,
    214,  202, 190, 180, 170, 160, 151, 143, 135, 127, 120, 113,
	107,  101,  95,  90,  85,  80,  76,  71,  67,  64,  60,  57,
];

static NOTE_NAMES: &'static [&'static str] = &["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

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
	
	print!("\tUsed periods: ");
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
		for i in 0..PERIODS.len() {
			let diff = (key as i32 - PERIODS[i] as i32).abs();
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
			format!("{}{}-{}",prefix,NOTE_NAMES[note],octave)
		};
		
		print!("{}({}) ",key,note);
	}
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
		match writer.write_all(&module.sample_data[*i]) {
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
		
		if si.length > 0 {
			// Remove sample data
			if index < module.sample_data.len() {
				module.sample_data.remove(index);	
			}		
		}
		
		si.length = 0;
		si.repeat_start = 0;
		si.repeat_length = 0;
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
                            .and_then(|d| d.decode())
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
		let module = match ptmf::read_mod(&mut reader) {
			Ok(module) => module,
			Err(e) => {
				println!("Failed to parse file: '{}' Error: '{:?}'", filename, e);
				return
			}
		};

		println!("Processing: {}", filename);
		
		let range = if args.flag_all {
			0..module.sample_data.len()
		} else {
			let number = usize::from_str(&args.flag_number).unwrap() - 1;
			if number >= module.sample_data.len() {
				println!("Invalid sample number. Only {} samples available.", module.sample_data.len());
				return
			}
			number..number+1
		};
		
		save_samples(&module,&(range.collect()),&args.arg_fileprefix);
	} else if args.cmd_remove {
		for ref filename in args.arg_file {
			let file = match File::open(filename) {
				Ok(file) => file,
				Err(e) => {
					println!("Failed to open file: '{}' Error: '{}'", filename, e);
					continue
				}
			};
			
			let mut reader = BufReader::new(&file);
			let mut module = match ptmf::read_mod(&mut reader) {
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
