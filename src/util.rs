use checksums;

struct FileParams {
	pub check_sum: &'static str,
	pub offset: usize,
}

const recognized_files : [FileParams; 1] = [
	FileParams {
		check_sum: "1408778DE813902587AD83438BCE52A606F8895C5D74478762AD1C9CF92D5A96",
		offset: 0xB8A21C
	}
];

pub fn identify_file(file_content:&std::vec::Vec<u8>) -> Result<usize, &'static str>{	
	for file_param in recognized_files.iter() {
		if file_content.len() <= (file_param.offset + 4) {
			continue;
		}

		let mut file_content_copy = file_content.clone();
		file_content_copy[file_param.offset] = 0x0;
		file_content_copy[file_param.offset + 1] = 0x0;
		file_content_copy[file_param.offset + 2] = 0x80;
		file_content_copy[file_param.offset + 3] = 0x3f;
		
		let mut file_content_copy_cursor = std::io::Cursor::new(file_content_copy);
		let file_hash = checksums::hash_reader(& mut file_content_copy_cursor, checksums::Algorithm::SHA2256);
		if file_hash.eq(&file_param.check_sum){
			return Ok(file_param.offset);
		}
	}
	Err("file did not match any of the checksums")
}

pub fn read_current_gravity(file_content:&std::vec::Vec<u8>, offset:usize) -> f32{
	let to_float:[u8; 4] = [file_content[offset], file_content[offset+1], file_content[offset+2], file_content[offset+3]];
	return f32::from_le_bytes(to_float);
}

pub fn change_gravity(file_content:&mut std::vec::Vec<u8>, offset:usize, gravity:f32) -> Result<(), &'static str>{
	if file_content.len() <= (offset + 4){
		return Err("file smaller than offset");
	}

	let bytes = gravity.to_le_bytes();
	for i in 0..4{
		file_content[offset + i] = bytes[i]
	}
	Ok(())
}
