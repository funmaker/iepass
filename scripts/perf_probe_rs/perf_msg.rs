use eframe::egui::Color32;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct RawPerfMessage {
	pub sram: [u64; 2],
	pub psram: [u64; 2],
	pub trace: Vec<RawEntry>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RawEntry(pub String, pub u64, pub u64, pub usize);

#[derive(Copy, Clone)]
pub struct HeapStats {
	pub used: u64,
	pub total: u64,
}

pub struct Entry {
	pub name: String,
	pub start: f64,
	pub end: f64,
	pub level: f64,
	pub cpu: usize,
	pub stroke: Color32,
	pub fill: Color32,
}