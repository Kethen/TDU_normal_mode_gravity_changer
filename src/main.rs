#![windows_subsystem = "windows"]
mod util;

use std::io::Write;

use iced::widget::{button, row, column, text, text_input, scrollable, checkbox};
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

fn get_information_from_file(path:&String) -> Result<((f32, f32), bool, util::FileParams), String>{
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

	let gravity = match util::read_current_gravity(&file_content, &file_params){
		Ok(g) => g,
		Err(e) => {return Err(format!("failed retriving gravity from file: {}", e))},
	};

	let hc_physics_is_forced = match util::hc_mode_physics_is_forced(&file_content, &file_params){
		Ok(b) => b,
		Err(e) => {return Err(format!("failed checking if hc mode physics is forced: {}", e))},
	};

	return Ok((gravity, hc_physics_is_forced, file_params));
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

fn patch_file(path:&std::path::Path, gravity:(f32,f32), force_hc_mode_physics_on_normal:bool) -> Result<(), String>{
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

	match util::toggle_force_hc_mode_physics(&mut file_content, &file_params, force_hc_mode_physics_on_normal){
		Ok(_) => (),
		Err(e) => {return Err(format!("failed toggling force hc mode physics: {}", e))},
	};

	match std::fs::write(&path, file_content){
		Ok(_) => {return Ok(())},
		Err(e) => {return Err(format!("failed writing patched file to {}: {}", path.display(), e))},
	};
}

struct UserInterface {
	path:String,
	log_display:String,
	havok_gravity:String,
	normal_mode_gravity_modifier:String,
	file_name:String,
	file_recognized:bool,
	force_hc_mode_physics_on_normal:bool,
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

fn format_f32(value:f32, min_precision:usize) -> String{
	let base = format!("{}", value);
	for c in base.chars(){
		if c == '.'{
			return base;
		}
	}
	return format!("{}.{}", base, "0".repeat(min_precision))
}

#[derive(Debug, Clone)]
enum Message {
	FilePicker,
	Patch,
	ChangeGravity(String),
	ChangeNormalModeGravityModifier(String),
	Ignore,
	ToggleForceHCModePhysics(bool),
	CopyLogToClipboard,
}
impl Sandbox for UserInterface {
	type Message = Message;

	fn new() -> Self {
		Self {
			path: String::from(""),
			log_display: String::from(""),
			normal_mode_gravity_modifier: String::from("1.0"),
			havok_gravity: String::from("-9.81"),
			file_name: String::from(""),
			file_recognized: false,
			force_hc_mode_physics_on_normal: false,
		}
	}

	fn title(&self) -> String {
		String::from("Test Drive Unlimited Gravity Patcher")
	}

	fn update(&mut self, message: Message){
		match message{
			Message::FilePicker => {
				match pick_file(){
					Ok(path) => {
						self.path = format!("{}", path.display());
						match get_information_from_file(&self.path){
							Ok((g, fhc, p)) => {
								let (normal_mode_gravity_modifier, havok_gravity) = g;
								self.normal_mode_gravity_modifier = format_f32(normal_mode_gravity_modifier, 1);
								self.havok_gravity = format_f32(havok_gravity, 2);
								self.file_name = p.name.to_string();
								self.file_recognized = true;
								self.force_hc_mode_physics_on_normal = fhc;
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
				let havok_gravity_float = match self.havok_gravity.parse::<f32>() {
					Ok(float) => float,
					Err(_) => {
						self.log(format!("failed patching {}:\n{} is not a float value", self.path, self.havok_gravity));
						return;
					}
				};
				let normal_mode_gravity_modifier_float = match self.normal_mode_gravity_modifier.parse::<f32>() {
					Ok(float) => float,
					Err(_) => {
						self.log(format!("failed patching {}:\n{} is not a float value", self.path, self.normal_mode_gravity_modifier));
						return;
					}
				};
				match patch_file(&std::path::Path::new(&self.path), (normal_mode_gravity_modifier_float, havok_gravity_float), self.force_hc_mode_physics_on_normal){
					Ok(_) => {
						self.log(format!( "successfully patched {}", self.path));
					},
					Err(e) => {
						self.log(format!("failed patching {}:\n{}", self.path, e));
					},
				};
			},
			Message::ChangeGravity(g) => {
				self.havok_gravity = g;
			},
			Message::ChangeNormalModeGravityModifier(g) => {
				self.normal_mode_gravity_modifier = g;
			},
			Message::Ignore => {();},
			Message::ToggleForceHCModePhysics(t) => {
				self.force_hc_mode_physics_on_normal = t;
			},
			Message::CopyLogToClipboard => {
				println!("copying log to clipboard");
				// this doesn't quite work
				clipboard::write::<Message>(self.log_display.to_string());
			},
		}
	}

	fn view(&self) -> Element<Message>{
		let gravities_are_float =
			match self.normal_mode_gravity_modifier.parse::<f32>(){
				Ok(_) => true,
				Err(_) => false,
			}
		&&
			match self.havok_gravity.parse::<f32>(){
				Ok(_) => true,
				Err(_) => false,
			};
		column![
			column![
				row![
					text(format!(
						"{}\n{}\n{}\n{}\n{}",
						"Select your TestDriveUnlimited.exe, then set normal mode gravity modifier and overall gravity (y axis, down is negative) then click Patch.",
						"The modifier is only applied when a wheel is lifted off the ground in normal mode, the overall gravity is used everywhere.",
						"A 0.0 modifier should remove any extra downforce in normal mode, running up a ramp at 200 kph will get quite some air time.",
						"1.0 is the default modifier, -9.81 is the default gravity.",
						"Negative modifiers or positive gravity will send the vehicle flying upward.",
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
					text_input("Floating point gravity value", &self.havok_gravity).on_input(|a|Message::ChangeGravity(a)),
					text("Normal mode Gravity Modifier: "),
					text_input("Floating point gravity modifier value", &self.normal_mode_gravity_modifier).on_input(|a|Message::ChangeNormalModeGravityModifier(a)),
					text("Force HC physics on normal mode: "),
					checkbox("", self.force_hc_mode_physics_on_normal, Message::ToggleForceHCModePhysics),
					if gravities_are_float && self.file_recognized {
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
	settings.window.size = (800, 440);
	settings.window.resizable = false;
	UserInterface::run(settings);
}

fn simple(){
	let path = match pick_file(){
		Ok(path) => path,
		Err(e) => panic!("{}", e),
	};

	match patch_file(&path, (0.2, -9.81), false){
		Ok(_) => {println!("success")},
		Err(e) => panic!("{}", e),
	};
}

fn main(){
	// simple();
	gui();
}
