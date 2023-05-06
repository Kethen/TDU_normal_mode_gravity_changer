use checksums;

#[derive(Debug, Clone)]
pub struct FileParams {
	pub check_sum: &'static str,
	pub offset: usize,
	pub original_bytes: [u8; 6],
	pub name: &'static str,
}

pub fn identify_file(file_content:&std::vec::Vec<u8>) -> Result<FileParams, &'static str>{
	let recognized_files = [
		FileParams {
			check_sum: "1408778DE813902587AD83438BCE52A606F8895C5D74478762AD1C9CF92D5A96",
			offset: 0x5E2710,
			// mov ecx,dword ptr ds:[F8A21C]
			original_bytes: [0x8B, 0x0D, 0x1C, 0xA2, 0xF8, 0x00],
			name: "1.66a",
		}
	];

	for file_params in recognized_files.iter() {
		if file_content.len() <= (file_params.offset + 6) {
			continue;
		}

		let mut file_content_copy = file_content.clone();
		for (i, byte) in file_params.original_bytes.iter().enumerate(){
			file_content_copy[file_params.offset + i] = *byte;
		}
		
		let mut file_content_copy_cursor = std::io::Cursor::new(file_content_copy);
		let file_hash = checksums::hash_reader(& mut file_content_copy_cursor, checksums::Algorithm::SHA2256);
		if file_hash.eq(&file_params.check_sum){
			return Ok(file_params.clone());
		}
	}
	Err("file did not match any of the checksums")
}

pub fn read_current_gravity(file_content:&std::vec::Vec<u8>, file_params:&FileParams) -> Result<f32, &'static str>{
	if file_content.len() <= (file_params.offset + 6) {
		return Err("file smaller than offset ");
	}
	let mut modified = false;
	for (i, byte) in file_params.original_bytes.iter().enumerate(){
		if file_content[file_params.offset + i] != *byte{
			modified = true;
			break;
		}
	}
	if modified == false{
		return Ok(1.0);
	}
	let to_float:[u8; 4] = [file_content[file_params.offset+2], file_content[file_params.offset+3], file_content[file_params.offset+4], file_content[file_params.offset+5]];
	return Ok(f32::from_le_bytes(to_float));
}

pub fn change_gravity(file_content:&mut std::vec::Vec<u8>, file_params:&FileParams, gravity:f32) -> Result<(), &'static str>{
	if file_content.len() <= (file_params.offset + 6){
		return Err("file smaller than offset");
	}

	// two bytes of "mov ecx,"
	file_content[file_params.offset] = 0xc7;
	file_content[file_params.offset + 1] = 0xc1;

	// value
	let bytes = gravity.to_le_bytes();
	for (i, byte) in bytes.iter().enumerate(){
		file_content[file_params.offset + 2 + i] = *byte;
	}
	Ok(())
}
