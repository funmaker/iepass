//! ```cargo
//! [dependencies]
//! colored = "3.0.0"
//! eframe = "0.32.0"
//! egui_plot = "0.33.0"
//! serde_json = "1.0.0"
//! serde = { version = "1.0", features = ["derive"] }
//! 
//! [target.'cfg(target_os = "linux")'.dependencies]
//! ipipe = "0.11.7"
//! ```

use std::ffi::{OsStr, OsString};
use std::process::{Child, Command, exit};
use std::io::{BufRead, BufReader};
use std::sync::mpsc::{self, Receiver, TryRecvError, Sender};
use std::thread;
use std::collections::HashMap;
use eframe::egui::{self, Color32};
use egui_plot::{Plot, Legend, BarChart, Bar};
use serde::Deserialize;
use colored::Colorize;

const RUNNER: &'static str = "probe-rs";
const CPUS: usize = 2;

const COLORS: [Color32; 15] = [
	Color32::from_rgb(29, 43, 83),
	Color32::from_rgb(126, 37, 83),
	Color32::from_rgb(0, 135, 81),
	Color32::from_rgb(171, 82, 54),
	Color32::from_rgb(95, 87, 79),
	Color32::from_rgb(194, 195, 199),
	Color32::from_rgb(255, 241, 232),
	Color32::from_rgb(255, 0, 77),
	Color32::from_rgb(255, 163, 0),
	Color32::from_rgb(255, 236, 39),
	Color32::from_rgb(0, 228, 54),
	Color32::from_rgb(41, 173, 255),
	Color32::from_rgb(131, 118, 156),
	Color32::from_rgb(255, 119, 168),
	Color32::from_rgb(255, 204, 170),
];

fn main() {
	let args: Vec<_> = std::env::args_os().skip(1).collect();
	
	let (sender, receiver) = mpsc::channel();
	let mut probe = spawn_probe(args, sender);
	
	thread::spawn(move || {
		let status = probe.wait().unwrap();
		exit(status.code().unwrap_or(0));
	});
	
	let data = receiver.recv().unwrap();
	let native_options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
		..Default::default()
	};
	eframe::run_native("IEPass Perf", native_options, Box::new(|cc| Ok(Box::new(FlameGraph::new(cc, data, receiver))))).unwrap();
}

#[cfg(target_os = "linux")]
fn spawn_probe(mut args: Vec<OsString>, sender: Sender<Vec<RawEntry>>) -> Child {
	use ipipe::Pipe;
	
	let pipe = Pipe::with_name("iepass_perf").unwrap();
	args.push("--target-output-file".into());
	args.push(pipe.path().as_os_str().into());
	
	println!("     {} `{} {}`", "Running".green().bold(), RUNNER, args.join(OsStr::new(" ")).to_string_lossy());
	let probe = Command::new("probe-rs").args(args).spawn().unwrap();
	
	thread::spawn(move || {
		for line in BufReader::new(pipe).lines() {
			let line = line.unwrap();
			if let Some(line) = line.strip_prefix("[PERF ] ") {
				match serde_json::from_str(line) {
					Ok(entries) => sender.send(entries).unwrap(),
					Err(err) => eprintln!("Can't parse PERF message:\n{}", err),
				}
			}
		}
	});
	
	probe
}

#[cfg(not(target_os = "linux"))]
fn spawn_probe(args: Vec<OsString>, sender: Sender<Vec<RawEntry>>) -> Child {
	use std::process::Stdio;
	
	println!("     {} `{} {}`", "Running".green().bold(), RUNNER, args.join(OsStr::new(" ")).to_string_lossy());
	let mut probe = Command::new("probe-rs").args(args).env("CLICOLOR_FORCE", "true").stdout(Stdio::piped()).spawn().unwrap();
	let probe_out = probe.stdout.take().unwrap();
	
	thread::spawn(move || {
		for line in BufReader::new(probe_out).lines() {
			let line = line.unwrap();
			println!("{}", line);
			
			if let Some(line) = line.strip_prefix("[PERF ] ") {
				match serde_json::from_str(line) {
					Ok(entries) => sender.send(entries).unwrap(),
					Err(err) => eprintln!("Can't parse PERF message:\n{}", err),
				}
			}
		}
	});
	
	probe
}

#[derive(Deserialize, Debug, Clone)]
struct RawEntry(String, u64, u64, usize);

struct Entry {
	name: String,
	start: f64,
	end: f64,
	level: f64,
	cpu: usize,
	stroke: Color32,
	fill: Color32,
}

struct FlameGraph {
	data: Vec<Entry>,
	receiver: Receiver<Vec<RawEntry>>,
	legend: HashMap<String, Color32>,
	max_x: f64,
	max_y: f64,
}

impl FlameGraph {
	fn new(_cc: &eframe::CreationContext<'_>, data: Vec<RawEntry>, receiver: Receiver<Vec<RawEntry>>) -> Self {
		let mut this = Self {
			data: vec![],
			receiver,
			legend: HashMap::new(),
			max_x: 0.0,
			max_y: 0.0,
		};
		this.update_data(data);
		this
	}
	
	fn update_data(&mut self, mut data: Vec<RawEntry>) {
		let mut colors_iter = COLORS.iter().copied().cycle();
		let mut stack = HashMap::new();
		
		data.sort_by_key(|entry| entry.1);
		self.legend.clear();
		self.data.clear();
		self.max_x = 0.0;
		self.max_y = 0.0;
		
		for entry in data {
			let RawEntry(name, start, end, cpu) = entry;
			let cpu_stack = stack.entry(cpu).or_insert_with(|| vec![]);
			let start = start as f64 / 1000.0;
			let end = end as f64 / 1000.0;
			let color = *self.legend.entry(name.clone())
			                        .or_insert_with(|| colors_iter.next().unwrap());
			
			cpu_stack.retain(|&val| val > end);
			
			let level = cpu_stack.len() as f64;
			self.data.push(Entry {
				name,
				start,
				end,
				level,
				cpu,
				stroke: color,
				fill: color.gamma_multiply(0.5),
			});
			
			self.max_x = self.max_x.max(end);
			self.max_y = self.max_y.max(level + 1.0);
			cpu_stack.push(end);
		}
	}
}

impl eframe::App for FlameGraph {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		match self.receiver.try_recv() {
			Ok(data) => self.update_data(data),
			Err(TryRecvError::Empty) => {},
			Err(TryRecvError::Disconnected) => {
				eprintln!("Data channel hung up.");
				ctx.send_viewport_cmd(egui::ViewportCommand::Close);
				return
			}
		}
		
		egui::CentralPanel::default().show(ctx, |ui| {
			let height = ui.available_height() / CPUS as f32;
			let link_group = ui.id().with("linkaxis");
			
			for cpu in (0..CPUS).rev() {
				Plot::new("Time Graph")
					.height(height)
					.include_x(0.0)
					.include_x(self.max_x)
					.include_y(0.0)
					.include_y(self.max_y + 0.5)
					.legend(Legend::default())
					.show_y(false)
					.show_grid([true, false])
					.allow_drag([true, false])
					.allow_scroll([true, false])
					.allow_zoom([true, false])
					.allow_drag([true, false])
					.allow_axis_zoom_drag([true, false])
					.allow_boxed_zoom(false)
					.link_axis(link_group, [true, false])
					.link_cursor(link_group, [true, false])
					.label_formatter(|_, point| format!("{:.3}ms", point.x))
					.y_grid_spacer(|_| vec![])
					.y_axis_label(format!("CPU{}", cpu))
					.show_axes(if cpu == 0 { [false, true] } else { [true, true] })
					.show(ui, |plot_ui| {
						for (name, &color) in self.legend.iter() {
							plot_ui.bar_chart(
								BarChart::new(
									name.clone(),
									self.data.iter()
									    .filter(|entry| entry.cpu == cpu && &entry.name == name)
									    .map(|entry|
									        Bar::new(entry.level + 0.5, entry.end - entry.start)
										        .name(name.clone())
										        .horizontal()
										        .base_offset(entry.start)
										        .width(1.0)
										        .stroke((1.0, entry.stroke))
										        .fill(entry.fill)
									    )
									    .collect()
								).color(color)
								 .element_formatter(Box::new(|bar, _| match bar.value {
									 ..0.0001 => format!("{}\n<1us", bar.name),
									 ..1.0 => format!("{}\n{:.0}us", bar.name, bar.value * 1000.0),
									 ..10.0 => format!("{}\n{:.3}ms", bar.name, bar.value),
									 ..100.0 => format!("{}\n{:.2}ms", bar.name, bar.value),
									 ..1000.0 => format!("{}\n{:.1}ms", bar.name, bar.value),
									 _ => format!("{}\n{:.0}ms", bar.name, bar.value),
								 }))
							)
						}
					});
			}
		});
	}
}
