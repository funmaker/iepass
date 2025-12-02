use std::ffi::{OsStr, OsString};
use std::process::{Child, Command, exit};
use std::io::{BufRead, BufReader};
use std::sync::mpsc::{self, Receiver, TryRecvError, Sender};
use std::thread;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::atomic::AtomicBool;
use eframe::egui::{self, Color32};
use egui_plot::{Plot, Legend, BarChart, Bar, Corner};
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
	
	if let Err(err) = ctrlc::set_handler(ctrlc_handler) {
		eprintln!("Failed to set up Ctrl-C handler. Backtrace might be missing: {err}");
	}
	
	let (sender, receiver) = mpsc::channel();
	let probe = spawn_probe(args, sender);
	
	thread::spawn(move || wait_for_exit(probe));
	
	let initial_data = match receiver.recv() {
		Ok(data) => data,
		Err(_) => return,
	};
	
	let native_options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
		..Default::default()
	};
	
	eframe::run_native("IEPass Perf", native_options, Box::new(|cc| Ok(Box::new(FlameGraph::new(cc, initial_data, receiver))))).unwrap();
}

static CTRLC_RECEIVED: AtomicBool = AtomicBool::new(false);

fn ctrlc_handler() {
	if CTRLC_RECEIVED.swap(true, Ordering::Relaxed) {
		eprintln!("Received second Ctrl+C, exiting.");
		exit(-1);
	} else {
		eprintln!("Received Ctrl+C, waiting for probe-rs to exit.");
	}
}

fn wait_for_exit(mut probe: Child) {
	loop {
		if let Some(status) = probe.try_wait().expect("Failed to wait for probe-rs process.") {
			exit(status.code().unwrap_or(0));
		} else {
			std::thread::yield_now();
		}
	}
}

#[cfg(target_os = "linux")]
fn spawn_probe(mut args: Vec<OsString>, sender: Sender<RawPerfMessage>) -> Child {
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
fn spawn_probe(args: Vec<OsString>, sender: Sender<RawPerfMessage>) -> Child {
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
struct RawPerfMessage {
	sram: [u64; 2],
	psram: [u64; 2],
	trace: Vec<RawEntry>,
}

#[derive(Deserialize, Debug, Clone)]
struct RawEntry(String, u64, u64, usize);

#[derive(Copy, Clone)]
struct RamStats {
	used: u64,
	total: u64,
}

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
	sram: RamStats,
	psram: RamStats,
	trace: Vec<Entry>,
	receiver: Receiver<RawPerfMessage>,
	legend: HashMap<String, Color32>,
	max_x: f64,
	max_y: f64,
}

impl FlameGraph {
	fn new(_cc: &eframe::CreationContext<'_>, data: RawPerfMessage, receiver: Receiver<RawPerfMessage>) -> Self {
		let mut this = Self {
			sram: RamStats { used: 0, total: 0 },
			psram: RamStats { used: 0, total: 0 },
			trace: Vec::with_capacity(data.trace.len()),
			receiver,
			legend: HashMap::new(),
			max_x: 0.0,
			max_y: 0.0,
		};
		this.update_data(data);
		this
	}
	
	fn update_data(&mut self, mut data: RawPerfMessage) {
		[self.sram.used,  self.sram.total]  = data.sram;
		[self.psram.used, self.psram.total] = data.psram;
		
		let mut colors_iter = COLORS.iter().copied().cycle();
		let mut stack = HashMap::new();
		
		data.trace.sort_by_key(|entry| entry.1);
		self.legend.clear();
		self.trace.clear();
		self.max_x = 0.0;
		self.max_y = 0.0;
		
		for entry in data.trace {
			let RawEntry(name, start, end, cpu) = entry;
			let cpu_stack = stack.entry(cpu).or_insert_with(|| vec![]);
			let start = start as f64 / 1000.0;
			let end = end as f64 / 1000.0;
			let color = *self.legend.entry(name.clone())
			                        .or_insert_with(|| colors_iter.next().unwrap());
			
			cpu_stack.retain(|&val| val > end);
			
			let level = cpu_stack.len() as f64;
			self.trace.push(Entry {
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
			let height = ui.available_height() / (CPUS + 2) as f32;
			let cpu_group = ui.id().with("cpu_linkaxis");
			let ram_group = ui.id().with("ram_linkaxis");
			
			for cpu in (0..CPUS).rev() {
				Plot::new(format!("Cpu {}", cpu))
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
					.link_axis(cpu_group, [true, false])
					.link_cursor(cpu_group, [true, false])
					.label_formatter(|_, point| format!("{:.3}ms", point.x))
					.y_grid_spacer(|_| vec![])
					.y_axis_label(format!("CPU{}", cpu))
					.show_axes(if cpu == 0 { [false, true] } else { [true, true] })
					.show(ui, |plot_ui| {
						for (name, &color) in self.legend.iter() {
							plot_ui.bar_chart(
								BarChart::new(
									name.clone(),
									self.trace
									    .iter()
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
			
			let mut ram_plot = |name: &str, stats: RamStats, axes: bool| {
				let used = stats.used as f64 / stats.total as f64 * 100.0;
				let free = 100.0 - used;
				let total_bytes = stats.total as f64;
				
				Plot::new(name)
					.height(height)
					.include_x(0.0)
					.include_x(100.0)
					.include_y(-0.6)
					.include_y(0.6)
					.legend(Legend::default().position(Corner::RightBottom))
					.show_y(false)
					.show_grid([true, false])
					.allow_drag([true, false])
					.allow_scroll([true, false])
					.allow_zoom([true, false])
					.allow_drag([true, false])
					.allow_axis_zoom_drag([true, false])
					.allow_boxed_zoom(false)
					.link_axis(ram_group, [true, false])
					.link_cursor(ram_group, [true, false])
					.label_formatter(|_, point| format!("{:.0}%", point.x))
					.y_grid_spacer(|_| vec![])
					.y_axis_label(name.to_string())
					.show_axes([axes, true])
					.show(ui, |plot_ui| {
						plot_ui.bar_chart(
							BarChart::new(
								"Used".to_string(),
								vec![
									Bar::new(0.0, used)
										.horizontal()
										.base_offset(0.0)
										.width(1.0)
										.stroke((1.0, Color32::CYAN))
										.fill(Color32::CYAN.gamma_multiply(0.5)),
								]
							).color(Color32::CYAN)
							 .element_formatter(Box::new(move |bar, _| {
								 let bytes = bar.value / 100.0 * total_bytes;
								 match bytes.log2() {
									 ..10.0 => format!("{} bytes ({:.0}%)", bytes, bar.value),
									 ..20.0 => format!("{:.1}KB ({:.0}%)", bytes / 1024.0, bar.value),
									 ..30.0 => format!("{:.1}MB ({:.0}%)", bytes / 1024.0 / 1024.0, bar.value),
									 _ => format!("{:.1}GB ({:.0}%)", bytes / 1024.0 / 1024.0 / 1024.0, bar.value),
								 }
							 }))
						);
						plot_ui.bar_chart(
							BarChart::new(
								"Free".to_string(),
								vec![
									Bar::new(0.0, free)
										.horizontal()
										.base_offset(used)
										.width(1.0)
										.stroke((1.0, Color32::GRAY))
										.fill(Color32::GRAY.gamma_multiply(0.5)),
								]
							).color(Color32::GRAY)
							 .element_formatter(Box::new(move |bar, _| {
								 let bytes = bar.value / 100.0 * total_bytes;
								 match bytes.log2() {
									 ..10.0 => format!("{} bytes ({:.0}%)", bytes, bar.value),
									 ..20.0 => format!("{:.1}KB ({:.0}%)", bytes / 1024.0, bar.value),
									 ..30.0 => format!("{:.1}MB ({:.0}%)", bytes / 1024.0 / 1024.0, bar.value),
									 _ => format!("{:.1}GB ({:.0}%)", bytes / 1024.0 / 1024.0 / 1024.0, bar.value),
								 }
							 }))
						);
					});
			};
			
			ram_plot("SRAM", self.sram, true);
			ram_plot("PSRAM", self.psram, false);
		});
	}
}
