use checksums;

#[derive(Debug, Clone)]
pub struct FileParams {
	pub check_sum: &'static str,
	pub normal_mode_offset: usize,
	pub normal_mode_original_bytes: [u8; 6],
	pub havok_offset: usize,
	pub havok_original_bytes: [u8; 4],
	pub name: &'static str,
}

pub fn identify_file(file_content:&std::vec::Vec<u8>) -> Result<FileParams, &'static str>{
	let recognized_files = [
		FileParams {
			check_sum: "1408778DE813902587AD83438BCE52A606F8895C5D74478762AD1C9CF92D5A96",
			normal_mode_offset: 0x5E2710,
			// mov ecx,dword ptr ds:[F8A21C]
			normal_mode_original_bytes: [0x8B, 0x0D, 0x1C, 0xA2, 0xF8, 0x00],
			havok_offset: 0xB41CC4,
			// -9.81 le f32
			havok_original_bytes: [0xC3, 0xF5, 0x1C, 0xC1],
			name: "1.66a",
		}
	];

	for file_params in recognized_files.iter() {
		if file_content.len() <= file_params.normal_mode_offset + 6 || file_content.len() <= file_params.havok_offset + 4 {
			continue;
		}

		let mut file_content_copy = file_content.clone();
		for (i, byte) in file_params.normal_mode_original_bytes.iter().enumerate(){
			file_content_copy[file_params.normal_mode_offset + i] = *byte;
		}

		for (i, byte) in file_params.havok_original_bytes.iter().enumerate(){
			file_content_copy[file_params.havok_offset + i] = *byte;
		}
		
		let mut file_content_copy_cursor = std::io::Cursor::new(file_content_copy);
		let file_hash = checksums::hash_reader(& mut file_content_copy_cursor, checksums::Algorithm::SHA2256);
		if file_hash.eq(&file_params.check_sum){
			return Ok(file_params.clone());
		}
	}
	Err("file did not match any of the checksums")
}

pub fn read_current_gravity(file_content:&std::vec::Vec<u8>, file_params:&FileParams) -> Result<(f32, f32), &'static str>{
	if file_content.len() <= file_params.normal_mode_offset + 6 || file_content.len() <= file_params.havok_offset + 4 {
		return Err("file smaller than expected");
	}
	let mut modified = false;
	for (i, byte) in file_params.normal_mode_original_bytes.iter().enumerate(){
		if file_content[file_params.normal_mode_offset + i] != *byte{
			modified = true;
			break;
		}
	}

	let mut normal_mode_gravity_modifier = 1.0;
	if modified == true{
		let to_float:[u8; 4] = [file_content[file_params.normal_mode_offset+2], file_content[file_params.normal_mode_offset+3], file_content[file_params.normal_mode_offset+4], file_content[file_params.normal_mode_offset+5]];
		normal_mode_gravity_modifier = f32::from_le_bytes(to_float);
	}

	let to_float:[u8; 4] = [file_content[file_params.havok_offset], file_content[file_params.havok_offset+1], file_content[file_params.havok_offset+2], file_content[file_params.havok_offset+3]];

	Ok((normal_mode_gravity_modifier, f32::from_le_bytes(to_float)))
}

pub fn change_gravity(file_content:&mut std::vec::Vec<u8>, file_params:&FileParams, gravity:(f32, f32)) -> Result<(), &'static str>{
	if file_content.len() <= file_params.normal_mode_offset + 6 || file_content.len() <= file_params.havok_offset + 4 {
		return Err("file smaller than expected");
	}

	let (normal_mode_gravity_modifier, havok_gravity) = gravity;

	// two bytes of "mov ecx,"
	file_content[file_params.normal_mode_offset] = 0xc7;
	file_content[file_params.normal_mode_offset + 1] = 0xc1;

	// value
	let bytes = normal_mode_gravity_modifier.to_le_bytes();
	for (i, byte) in bytes.iter().enumerate(){
		file_content[file_params.normal_mode_offset + 2 + i] = *byte;
	}

	// havok value
	let bytes = havok_gravity.to_le_bytes();
	for (i, byte) in bytes.iter().enumerate(){
		file_content[file_params.havok_offset + i] = *byte;
	}
	Ok(())
}
