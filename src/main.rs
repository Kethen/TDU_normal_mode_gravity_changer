mod util;

use iced::widget::{button, row, column, text, text_input};
use iced::{Alignment, Element, Sandbox, Settings};

use rfd::FileDialog;

fn get_gravity_from_file(path:&String) -> Result<f32, String>{
	let file_path = std::path::Path::new(&path);
	if file_path.is_file() != true{
		return Err(format!("file {} not found", path));
	}

	let file_content = match std::fs::read(&path){
		Ok(content) => content,
		Err(e) => {return Err(format!("cannot open {}: {}", path, e))}
	};
	
	let offset = match util::identify_file(&file_content){
		Ok(offset) => offset,
		Err(e) => {return Err(format!("failed identifying exe: {}", e))},
	};
	
	match util::read_current_gravity(&file_content, offset){
		Ok(gravity) => Ok(gravity),
		Err(e) => Err(format!("failed retriving gravity from file: {}", e)),
	}
}

fn pick_file() -> Result<std::path::PathBuf, &'static str>{
	match FileDialog::new()
		.add_filter("TestDriveUnlimited.exe", &["exe"])
		.set_directory(match std::env::current_dir(){
			Ok(path) => path,
			Err(e) => panic!("cannot get pwd..?"),
		})
		.pick_file(){
		Some(file) => Ok(file),
		None => Err("no file picked"),
	}
}

fn patch_file(path:&std::path::Path, gravity:f32) -> Result<(), String>{
	let mut file_content = match std::fs::read(&path){
		Ok(content) => content,
		Err(e) => {return Err(format!("cannot open {}: {}", path.display(), e))}
	};

	let backup_path_string = format!("{}", path.display()) + ".bak";
	let backup_path = std::path::Path::new(&backup_path_string);
	if backup_path.exists() != true{
		match std::fs::write(&backup_path, &file_content){
			Ok(_) => (),
			Err(e) => {return Err(format!("cannot backup exe to {}: {}", backup_path.display(), e))},
		};
	}

	let offset = match util::identify_file(&file_content){
		Ok(offset) => offset,
		Err(e) => {return Err(format!("failed identifying exe: {}", e))},
	};

	match util::change_gravity(&mut file_content, offset, gravity){
		Ok(_) => (),
		Err(e) => {return Err(format!("failed changing gravity: {}", e))},
	};

	match std::fs::write(&path, file_content){
		Ok(_) => {return Ok(())},
		Err(e) => {return Err(format!("failed writing patched file to {}: {}", path.display(), e))},
	};
}

struct UserInterface {
	path:String,
	log:String,
	gravity:String,
}

#[derive(Debug, Clone)]
enum Message {
	FilePicker,
	Patch,
	ChangeGravity(String),
}

impl Sandbox for UserInterface {
	type Message = Message;

	fn new() -> Self {
		Self {
			path: String::from(""),
			log: String::from(""),
			gravity: String::from("0.0"),
		}
	}

	fn title(&self) -> String {
		String::from("Test Drive Unlimited Normal Mode Gravity Patcher")
	}

	fn update(&mut self, message: Message){
		match message{
			Message::FilePicker => {
				match pick_file(){
					Ok(path) => {
						self.path = format!("{}", path.display());
						match get_gravity_from_file(&self.path){
							Ok(g) => {self.gravity = format!("{:.1}", g)},
							Err(_) => (),
						};
					},
					Err(e) => {
						()
						//self.log = format!("{}{}", self.log, format!("file picking failed/canceled:\n{}\n", e));
					}
				}
			},
			Message::Patch => {
				let gravity_float = match self.gravity.parse::<f32>() {
					Ok(float) => float,
					Err(e) => {
						self.log = format!("{}{}", self.log, format!("failed patching {}:\n{} is not a float value\n", self.path, self.gravity));
						return;
					}
				};
				match patch_file(&std::path::Path::new(&self.path), gravity_float){
					Ok(_) => {
						self.log = format!("{}{}", self.log, format!("successfully patched {}\n", self.path));
					},
					Err(e) => {
						self.log = format!("{}{}", self.log, format!("failed patching {}:\n{}\n", self.path, e));
					},
				};
			},
			Message::ChangeGravity(g) => {
				self.gravity = g;
			},
		}
	}

	fn view(&self) -> Element<Message>{
		let gravity_is_float = match self.gravity.parse::<f32>(){
			Ok(_) => true,
			Err(_) => false
		};
		column![
			row![
				text(format!(
					"{}\n{}\n{}\n{}\n{}",
					"Select your TestDriveUnlimited.exe, set a floating point gravity coefficient value then click Patch.",
					"This gravity value is only applied when a wheel is lifted off the ground.",
					"0.0 should remove any extra downforce, running up a ramp at 200 kph will get quite some air time.",
					"1.0 is the default value.",
					"Negative values that overcomes the game's gravity like -10.0 will send the vehicle flying upward on curbs and jumps.",
				)),
			].align_items(Alignment::Start),
			row![
				text("Patching target:"),
				text_input("TestDriveUnlimited.exe", &self.path),
				button("...").on_press(Message::FilePicker),
			],
			row![
				text("Gravity: "),
				text_input("Floating point gravity value", &self.gravity.to_string()).on_input(|a|Message::ChangeGravity(a)),
			].align_items(Alignment::Start),			
			row![
				if gravity_is_float && std::path::Path::new(&self.path).is_file(){
					button("Patch").on_press(Message::Patch)
				}else{
					button("Patch")					
				},
			].align_items(Alignment::Start),			
			row![
				text(&self.log),
			].align_items(Alignment::Start),
		]
		.align_items(Alignment::Center)
		.into()
	}
}

fn gui(){
	UserInterface::run(Settings::default());
}

fn simple(){
	let path = match pick_file(){
		Ok(path) => path,
		Err(e) => panic!("{}", e),
	};

	match patch_file(&path, 0.2){
		Ok(_) => {println!("success")},
		Err(e) => panic!("{}", e),
	};
}

fn main(){
	// simple();
	gui();
}
