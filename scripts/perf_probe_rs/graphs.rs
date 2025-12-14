use std::collections::HashMap;
use std::sync::mpmc::TryRecvError;
use std::sync::mpsc::Receiver;
use eframe::egui;
use eframe::egui::{Color32, ViewportBuilder};
use egui_plot::{Bar, BarChart, Corner, Legend, Plot};

use crate::elf::{Bus, Mapping, MemoryDesc, MemoryKind, Symbol, Symbols};
use crate::perf_msg::{Entry, HeapStats, RawEntry, RawPerfMessage};
use crate::utils::{format_bytes, format_millis};

const CPUS: usize = 2;

const COLORS: [Color32; 16] = [
	Color32::from_rgb(0, 228, 54),
	Color32::from_rgb(41, 173, 255),
	Color32::from_rgb(255, 0, 77),
	Color32::from_rgb(255, 236, 39),
	Color32::from_rgb(255, 204, 170),
	Color32::from_rgb(171, 82, 54),
	Color32::from_rgb(131, 118, 156),
	Color32::from_rgb(255, 119, 168),
	Color32::from_rgb(194, 195, 199),
	Color32::from_rgb(29, 43, 83),
	Color32::from_rgb(126, 37, 83),
	Color32::from_rgb(0, 135, 81),
	Color32::from_rgb(255, 241, 232),
	Color32::from_rgb(41, 24, 20),
	Color32::from_rgb(17, 29, 53),
	Color32::from_rgb(66, 33, 54),
];

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum GraphsTab {
	Flamegraph,
	Memory,
}

pub struct Graphs {
	tab: GraphsTab,
	sram: HeapStats,
	psram: HeapStats,
	trace: Vec<Entry>,
	receiver: Receiver<RawPerfMessage>,
	legend: HashMap<String, Color32>,
	symbols: Symbols,
	max_x: f64,
	max_y: f64,
}

impl Graphs {
	pub fn wait_for_data(receiver: Receiver<RawPerfMessage>, get_symbols: impl FnOnce() -> Symbols) {
		let initial_data = match receiver.recv() {
			Ok(data) => data,
			Err(_) => return,
		};
		
		let symbols = get_symbols();
		
		let native_options = eframe::NativeOptions {
			viewport: ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
			..Default::default()
		};
		
		eframe::run_native("IEPass Perf", native_options, Box::new(|_cc| Ok(Box::new(Graphs::new(initial_data, symbols, receiver))))).unwrap();
	}
	
	fn new(data: RawPerfMessage, symbols: Symbols, receiver: Receiver<RawPerfMessage>) -> Self {
		let mut this = Self {
			tab: GraphsTab::Flamegraph,
			sram: HeapStats { used: 0, total: 0 },
			psram: HeapStats { used: 0, total: 0 },
			trace: Vec::with_capacity(data.trace.len()),
			receiver,
			legend: HashMap::new(),
			symbols,
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
	
	fn draw_flamegraph_tab(&self, ui: &mut egui::Ui) {
		let height = ui.available_height() / CPUS as f32;
		let cpu_group = ui.id().with("cpu_linkaxis");
		
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
							 .element_formatter(Box::new(|bar, _| format!("{}\n{}", bar.name, format_millis(bar.value))))
						)
					}
				});
		}
	}
	
	fn draw_memory_tab(&self, ui: &mut egui::Ui) {
		let memory_count = self.symbols.len();
		let height = (ui.available_height() - 16.0) / (2 + memory_count) as f32;
		let ram_group = ui.id().with("ram_linkaxis");
		let mem_group = ui.id().with("mem_linkaxis");
		
		self.draw_heap("SRAM - heap", &self.sram, height, true, ram_group, ui);
		self.draw_heap("PSRAM - heap", &self.psram, height, false, ram_group, ui);
		
		for (id, (kind, symbols)) in self.symbols.iter().enumerate() {
			let name = match kind {
				MemoryKind::ROM => "ROM",
				MemoryKind::SRAM => "SRAM",
				MemoryKind::RTCSlow => "RTCSlow",
				MemoryKind::RTCFast => "RTCFast",
				MemoryKind::FLASH => "Flash",
			};
			
			self.draw_memory(name, *kind, symbols, height, id != memory_count - 1, mem_group, ui);
		}
	}
	
	fn draw_heap(&self, name: &str, stats: &HeapStats, height: f32, show_axes: bool, group: egui::Id, ui: &mut egui::Ui) {
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
			.link_axis(group, [true, false])
			.link_cursor(group, [true, false])
			.label_formatter(|_, point| format!("{:.0}%", point.x))
			.y_grid_spacer(|_| vec![])
			.y_axis_label(name.to_string())
			.show_axes([show_axes, true])
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
					 .element_formatter(Box::new(move |bar, _| format!("{} ({:.0}%)", format_bytes(bar.value / 100.0 * total_bytes), bar.value)))
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
					 .element_formatter(Box::new(move |bar, _| format!("{} ({:.0}%)", format_bytes(bar.value / 100.0 * total_bytes), bar.value)))
				);
			});
	}
	
	fn draw_memory(&self, name: &str, kind: MemoryKind, symbols: &HashMap<Mapping, Vec<Symbol>>, height: f32, show_axes: bool, group: egui::Id, ui: &mut egui::Ui) {
		let desc = MemoryDesc::get(kind);
		let mem_size = desc.size as f64;
		
		Plot::new(name)
			.height(height)
			.include_x(0.0)
			.include_x(100.0)
			.include_y(-0.6)
			.include_y(1.0)
			.legend(Legend::default().position(Corner::RightBottom))
			.show_y(false)
			.show_grid([true, false])
			.allow_drag([true, false])
			.allow_scroll([true, false])
			.allow_zoom([true, false])
			.allow_drag([true, false])
			.allow_axis_zoom_drag([true, false])
			.allow_boxed_zoom(false)
			.link_axis(group, [true, false])
			.link_cursor(group, [true, false])
			.label_formatter(|_, point| format!("{:.0}%", point.x))
			.y_grid_spacer(|_| vec![])
			.y_axis_label(name.to_string())
			.show_axes([show_axes, true])
			.show(ui, |plot_ui| {
				for (mapping, symbols) in symbols.iter() {
					let (name, color) = match mapping.bus {
						Bus::Inst => ("Code", COLORS[0]),
						Bus::Data => ("Data", COLORS[1]),
						Bus::Both => ("Shared", COLORS[2]),
					};
					
					plot_ui.bar_chart(
						BarChart::new(
							name.to_string(),
							symbols.iter()
								.map(|symbol|
									Bar::new(0.0, symbol.size as f64 / mem_size * 100.0)
										.horizontal()
										.base_offset(symbol.offset as f64 / mem_size * 100.0)
										.width(1.0)
										.name(symbol.name.to_string()))
								.collect(),
						).color(color)
					);
				}
			});
	}
}

impl eframe::App for Graphs {
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
			ui.horizontal(|ui| {
				if ui.button("Framegraph").clicked() {
					self.tab = GraphsTab::Flamegraph;
				}
				if ui.button("Memory").clicked() {
					self.tab = GraphsTab::Memory;
				}
			});
			
			match self.tab {
				GraphsTab::Flamegraph => self.draw_flamegraph_tab(ui),
				GraphsTab::Memory => self.draw_memory_tab(ui),
			}
		});
	}
}
