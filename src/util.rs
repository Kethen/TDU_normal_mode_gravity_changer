use checksums;

#[derive(Debug, Clone)]
pub struct FileParams {
	pub check_sum: &'static str,
	pub normal_mode_offset: usize,
	pub normal_mode_original_bytes: [u8; 6],
	pub havok_offset: usize,
	pub havok_original_bytes: [u8; 4],
	pub physics_mode_offset: usize,
	pub physics_mode_original_bytes: [u8; 23],
	pub physics_mode_force_hc_bytes: [u8; 23],
	pub name: &'static str,
}

fn file_is_too_small(file_content:&std::vec::Vec<u8>, file_params:&FileParams) -> bool{
	file_content.len() <= file_params.normal_mode_offset + 6
	|| file_content.len() <= file_params.havok_offset + 4
	|| file_content.len() <= file_params.physics_mode_offset + 5
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
			physics_mode_offset:0x4348A1,
			// mov al, [010e777c] ...
			physics_mode_original_bytes: [0xA0, 0x7C, 0x77, 0x0E, 0x01, 0x84, 0xC0, 0x74, 0x15, 0x8A, 0x45, 0x18, 0x84, 0xC0, 0x75, 0x0E, 0x8A, 0x45, 0x1C, 0x84, 0xC0, 0x75, 0x07],
			// mov al, 1; nop; nop ...
			physics_mode_force_hc_bytes: [0xC6, 0xC0, 0x01, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90],
			name: "1.66a",
		},
		FileParams {
			check_sum: "F452AFCDACC3D69858EA8188BEFD4764B0994C72120892D78A7CAC98C77DE6ED",
			normal_mode_offset: 0x5E2710,
			// mov ecx,dword ptr ds:[F8A21C]
			normal_mode_original_bytes: [0x8B, 0x0D, 0x1C, 0xA2, 0xF8, 0x00],
			havok_offset: 0xB41CC4,
			// -9.81 le f32
			havok_original_bytes: [0xC3, 0xF5, 0x1C, 0xC1],
			physics_mode_offset:0x4348A1,
			// mov al, [010e777c] ...
			physics_mode_original_bytes: [0xA0, 0x7C, 0x77, 0x0E, 0x01, 0x84, 0xC0, 0x74, 0x15, 0x8A, 0x45, 0x18, 0x84, 0xC0, 0x75, 0x0E, 0x8A, 0x45, 0x1C, 0x84, 0xC0, 0x75, 0x07],
			// mov al, 1; nop; nop ...
			physics_mode_force_hc_bytes: [0xC6, 0xC0, 0x01, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90],
			name: "1.66a platinum",
		},
	];

	for file_params in recognized_files.iter() {
		if file_is_too_small(file_content, file_params) {
			continue;
		}

		let mut file_content_copy = file_content.clone();
		for (i, byte) in file_params.normal_mode_original_bytes.iter().enumerate(){
			file_content_copy[file_params.normal_mode_offset + i] = *byte;
		}

		for (i, byte) in file_params.havok_original_bytes.iter().enumerate(){
			file_content_copy[file_params.havok_offset + i] = *byte;
		}

		for (i, byte) in file_params.physics_mode_original_bytes.iter().enumerate(){
			file_content_copy[file_params.physics_mode_offset + i] = *byte;
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
	if file_is_too_small(file_content, file_params){
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
	if file_is_too_small(file_content, file_params) {
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

pub fn hc_mode_physics_is_forced(file_content:&std::vec::Vec<u8>, file_params:&FileParams) -> Result<bool, &'static str>{
	if file_is_too_small(file_content, file_params) {
		return Err("file smaller than expected");
	}

	for (i, byte) in file_params.physics_mode_original_bytes.iter().enumerate(){
		if file_content[file_params.physics_mode_offset + i] != *byte{
			return Ok(true);
		}
	}
	return Ok(false);
}

pub fn toggle_force_hc_mode_physics(file_content:&mut std::vec::Vec<u8>, file_params:&FileParams, force_hc_mode_physics:bool) -> Result<(), &'static str>{
	if file_is_too_small(file_content, file_params) {
		return Err("file smaller than expected");
	}

	let mut patch_bytes = &file_params.physics_mode_original_bytes;
	if force_hc_mode_physics{
		patch_bytes = &file_params.physics_mode_force_hc_bytes;
	}

	for (i, byte) in patch_bytes.iter().enumerate(){
		file_content[file_params.physics_mode_offset + i] = *byte;
	}
	return Ok(());
}
