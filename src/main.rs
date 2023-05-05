use std::path::Path;
mod util;

fn main() {
	reference()
}

fn reference() {
	let path_unpatched = Path::new("test_unpatched.exe");
	let path_patched = Path::new("test_patched.exe");

	let mut file_content_unpatched = match std::fs::read(&path_unpatched){
		Err(why) => panic!("coult not read from {}: {}", path_unpatched.display(), why),
		Ok(file) => file,
	};

	let mut file_content_patched = match std::fs::read(&path_patched){
		Err(why) => panic!("coult not read from {}: {}", path_patched.display(), why),
		Ok(file) => file,
	};

	let offset = match util::identify_file(&file_content_patched){
		Err(why) => panic!("file not recognized: {}", why),
		Ok(offset) => offset,
	};

	println!("{}", util::read_current_gravity(&file_content_patched, offset));

	let mut file_content_test = file_content_unpatched.clone();
	util::change_gravity(&mut file_content_test, offset, 0.5);
	println!("{}", util::read_current_gravity(&file_content_test, offset));

	let path_changed = Path::new("test_changed.exe");
	match std::fs::write(&path_changed, &file_content_test){
		Err(why) => panic!("failed writing {}: {}", path_changed.display(), why),
		Ok(_) => (),
	};

	let offset = match util::identify_file(&file_content_unpatched){
		Err(why) => panic!("file not recognized: {}", why),
		Ok(offset) => offset,
	};

	println!("{}", util::read_current_gravity(&file_content_unpatched, offset));
}
