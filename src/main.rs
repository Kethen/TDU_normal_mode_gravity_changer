mod util;

use std::io::Write;

use iced::widget::{button, row, column, text, text_input, scrollable};
use iced::{Alignment, Element, Sandbox, Settings};
use iced::clipboard;
use iced::Length;

use rfd::FileDialog;

fn get_log_path() -> String{
	match std::env::current_dir(){
			Ok(path) => format!("{}/tdu_gravity_patcher.log", path.display()),
			Err(e) => panic!("cannot get pwd..? {}", e),
	}
}

fn log_to_file<S:AsRef<str>>(message:S){
	let message_ref = message.as_ref();
	let log_path = get_log_path();
	let mut log_file = match std::fs::OpenOptions::new()
		.write(true)
		.create(true)
		.append(true)
		.open(&log_path){
		Ok(file) => file,
		Err(e) => {
			println!("cannot open {}: {}", log_path, e);
			return;
		}
	};
	match log_file.write_fmt(format_args!("{}\n", message_ref)){
		Ok(_) => (),
		Err(e) => println!("cannot write to {}: {}", log_path, e),
	};
}

fn get_information_from_file(path:&String) -> Result<(f32, util::FileParams), String>{
	let file_path = std::path::Path::new(&path);
	if file_path.is_file() != true{
		return Err(format!("file {} not found", path));
	}

	let file_content = match std::fs::read(&path){
		Ok(content) => content,
		Err(e) => {return Err(format!("cannot open {}: {}", path, e))}
	};
	
	let file_params = match util::identify_file(&file_content){
		Ok(params) => params,
		Err(e) => {return Err(format!("failed identifying exe: {}", e))},
	};
	
	match util::read_current_gravity(&file_content, &file_params){
		Ok(gravity) => Ok((gravity, file_params)),
		Err(e) => Err(format!("failed retriving gravity from file: {}", e)),
	}
}

fn pick_file() -> Result<std::path::PathBuf, &'static str>{
	match FileDialog::new()
		.add_filter("TestDriveUnlimited.exe", &["exe"])
		.set_directory(match std::env::current_dir(){
			Ok(path) => path,
			Err(e) => panic!("cannot get pwd..? {}", e),
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

	let file_params = match util::identify_file(&file_content){
		Ok(params) => params,
		Err(e) => {return Err(format!("failed identifying exe: {}", e))},
	};

	match util::change_gravity(&mut file_content, &file_params, gravity){
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
	log_display:String,
	gravity:String,
	file_name:String,
	file_recognized:bool,
}

trait SandboxWithLog {
	fn log<S: AsRef<str>>(&mut self, message: S);
}

impl SandboxWithLog for UserInterface {
	fn log<S: AsRef<str>>(&mut self, message:S){
		let message_ref = message.as_ref();
		self.log_display = format!("{}{}\n", self.log_display, message_ref);
		log_to_file(message_ref);
	}
}

#[derive(Debug, Clone)]
enum Message {
	FilePicker,
	Patch,
	ChangeGravity(String),
	Ignore,
	CopyLogToClipboard,
}
impl Sandbox for UserInterface {
	type Message = Message;

	fn new() -> Self {
		Self {
			path: String::from(""),
			log_display: String::from(""),
			gravity: String::from("1.0"),
			file_name: String::from(""),
			file_recognized: false,
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
						match get_information_from_file(&self.path){
							Ok((g, p)) => {
								self.gravity = format!("{:.1}", g);
								self.file_name = p.name.to_string();
								self.file_recognized = true;
							},
							Err(_) => {
								self.file_name = String::from("");
								self.file_recognized = false;
							},
						};
					},
					Err(e) => {
						println!("file picking failed/canceled:\n{}", e);
					}
				}
			},
			Message::Patch => {
				let gravity_float = match self.gravity.parse::<f32>() {
					Ok(float) => float,
					Err(_) => {
						self.log(format!("failed patching {}:\n{} is not a float value", self.path, self.gravity));
						return;
					}
				};
				match patch_file(&std::path::Path::new(&self.path), gravity_float){
					Ok(_) => {
						self.log(format!( "successfully patched {}", self.path));
					},
					Err(e) => {
						self.log(format!("failed patching {}:\n{}", self.path, e));
					},
				};
			},
			Message::ChangeGravity(g) => {
				self.gravity = g;
			},
			Message::Ignore => {();},
			Message::CopyLogToClipboard => {
				println!("copying log to clipboard");
				// this doesn't quite work
				clipboard::write::<Message>(self.log_display.to_string());
			},
		}
	}

	fn view(&self) -> Element<Message>{
		let gravity_is_float = match self.gravity.parse::<f32>(){
			Ok(_) => true,
			Err(_) => false
		};
		column![
			column![
				row![
					text(format!(
						"{}\n{}\n{}\n{}\n{}",
						"Select your TestDriveUnlimited.exe, set a floating point gravity coefficient value then click Patch.",
						"This gravity value is only applied when a wheel is lifted off the ground.",
						"0.0 should remove any extra downforce, running up a ramp at 200 kph will get quite some air time.",
						"1.0 is the default value.",
						"Negative values that overcomes the game's gravity like -10.0 will send the vehicle flying upward on curbs and jumps.",
					)).width(Length::Fill),
				],
				row![
					text("Patching target:"),
					text_input("TestDriveUnlimited.exe", &self.path).on_input(|_|Message::Ignore),
					button("...").on_press(Message::FilePicker),
				].align_items(Alignment::Center),
				row![
					if self.file_recognized{
						text(format!("File recognized as {}", self.file_name))
					}else{
						if self.path.len() == 0{
							text("No file selected yet")
						}else{
							text("File not recognized")
						}
					}.width(Length::Fill),
				].align_items(Alignment::Center),
				row![
					text("Gravity: "),
					text_input("Floating point gravity value", &self.gravity.to_string()).on_input(|a|Message::ChangeGravity(a)),
					if gravity_is_float && self.file_recognized {
						button("Patch").on_press(Message::Patch)
					}else{
						button("Patch")
					},
				].align_items(Alignment::Center),
			]
			.width(Length::Fill)
			.align_items(Alignment::Start),
			row![
				scrollable(
					text(&self.log_display).width(Length::Fill)
				).height(160),
			].align_items(Alignment::Start),
			row![
				column![
					if self.log_display.len() != 0{
						button("Copy logs to clipboard").on_press(Message::CopyLogToClipboard)
					}else{
						button("Copy logs to clipboard")
					},
				]
				.width(Length::Fill)
				.align_items(Alignment::End),
			]
			.height(0)
			.width(Length::Fill)
			.align_items(Alignment::Center),
			row![
				text(format!("Logs are also written to {} if possible", get_log_path())).width(Length::Fill),
			].width(Length::Fill),
		]
		.padding(10)
		.align_items(Alignment::Center)
		.into()
	}
}

fn gui(){
	let mut settings = Settings::default();
	settings.window.size = (800, 400);
	settings.window.resizable = false;
	UserInterface::run(settings);
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
