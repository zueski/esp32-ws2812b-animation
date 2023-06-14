use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use regex::Regex;
use std::collections::HashMap;

// Necessary because of this issue: https://github.com/rust-lang/cargo/issues/9641
fn main() -> Result<(), Box<dyn std::error::Error>> {
	// generate images as code
	let filename_re = Regex::new(r"^([^_]+)_(\d+)\.png$").unwrap();
	let mut log_file = fs::File::create("images_build.log")?;
	let mut genfile:Vec<u8> = Vec::new();
	writeln!(genfile, "#[automatically_derived]")?;
	writeln!(genfile, "#[allow(unused_attributes)]")?;
	writeln!(genfile, "#[allow(dead_code)]")?;
	writeln!(log_file, "Looking for images in \"resources/img\"")?;
	let mut paths: Vec<_> = fs::read_dir("resources/img").unwrap().map(|r| r.unwrap()).collect();
	paths.sort_by_key(|dir| dir.path());
	
	let mut image_seqs: HashMap<String,Vec<PathBuf>> = HashMap::new();
	//let mut names: Vec<String>;
	//let mut dir_entry: &DirEntry;
	//let mut path: PathBuf;
	for dir_entry in paths {
		//dir_entry = &path_buf;
		let path = dir_entry.path().to_owned();
		let filename = &path.file_name().ok_or("")?.to_str().ok_or("")?;
		let filename_capt = filename_re.captures(&filename).expect(format!("file name {:?} does not match pattern {:?}", filename, filename_re).as_str());
		let name = filename_capt.get(1).unwrap().as_str().to_uppercase().to_owned();
		let mut seq_files: Vec<PathBuf>;
		if image_seqs.contains_key(&name) {
			seq_files = image_seqs.get_mut(&name).unwrap().to_vec();
		} else {
			writeln!(log_file, "found image name {}", &name)?;
			seq_files = Vec::new();
		}
		seq_files.push(path.to_owned());
		writeln!(log_file, "adding image path {:?} ({})", &path, seq_files.len())?;
		image_seqs.insert(name.to_owned(), seq_files.to_owned());
	}
	let mut is_first_name = true;
	for name in image_seqs.keys() {
		let seq_files = image_seqs.get(name).unwrap().to_vec();
		let frame_count = &seq_files.len();
		writeln!(log_file, "writing bytes for image name {} with {} frames", &name, frame_count)?;
		if !is_first_name {
			writeln!(genfile, "];")?;
		}
		is_first_name = false;
		let mut is_first_image = true;
		for path in seq_files {
			let decoder = png::Decoder::new(fs::File::open(&path).unwrap());
			let mut reader = decoder.read_info().unwrap();
			let mut buf = vec![0; reader.output_buffer_size()];
			let info = &reader.info();
			writeln!(log_file, "Reading d {}, found png sized {} {} with {} bytes", path.display(), &info.width, &info.height, reader.output_buffer_size())?;
			if is_first_image {
				writeln!(genfile, "pub const {}_COUNT: usize = {};", name, frame_count)?;
				//writeln!(genfile, "pub const {}_FRAME_SIZE: usize = {};", name, reader.output_buffer_size()/4)?;
				writeln!(genfile, "pub const {}_X_LEN: usize = {};", name, info.width)?;
				writeln!(genfile, "pub const {}_Y_LEN: usize = {};", name, info.height)?;
				//writeln!(genfile, "pub const {}_FRAMES: [[u8; {}]; {}] = [", name, reader.output_buffer_size(), frame_count)?;
				writeln!(genfile, "pub const {}_FRAMES: [[u8; {}]; {}] = [", name, reader.output_buffer_size(), frame_count)?;
				is_first_image = false;
			}
			let _frameinfo = &reader.next_frame(&mut buf).unwrap();
			writeln!(genfile, "\t{:?},", &buf)?;
		}
	}
	writeln!(genfile, "];")?;
	//let out_dir = env::var_os("OUT_DIR").unwrap();
	//let dest_path = Path::new(&out_dir).join("images.rs");
	let dest_path = Path::new("src").join("images.rs");
	fs::write(&dest_path,genfile)?;
	// esp
	embuild::build::CfgArgs::output_propagated("ESP_IDF")?;
	embuild::build::LinkArgs::output_propagated("ESP_IDF")?;
	Ok(())
}
