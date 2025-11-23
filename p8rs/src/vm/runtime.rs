use core::alloc::Allocator;
use core::ops::{Div, Not};
use core::any::Any;
use alloc::alloc::Global;
use alloc::boxed::Box;

use crate::utils;
use crate::vm::callbacks::{Callbacks, DefaultCallbacks};
use crate::vm::memory::{Memory, MemoryAccess};


pub struct Runtime<A: Allocator = Global> {
	pub cart_memory: Box<[u8; 0x8000], A>,
	pub memory: Box<Memory, A>,
	pub buttons: Buttons,
	pub target_fps: u16,
	pub callbacks: Box<dyn Callbacks>,
	cursor: [i16; 2],
	cursor_home: i16,
}

impl<A> Runtime<A>
where A: Allocator + Clone
{
	pub fn new(alloc: A) -> Runtime<A> {
		Self {
			cart_memory: utils::new_zeroed_box_in(alloc.clone()),
			memory: Memory::new_in(alloc),
			buttons: Buttons::new(),
			target_fps: 30,
			callbacks: Box::new(DefaultCallbacks),
			cursor: [0, 6],
			cursor_home: 0,
		}
	}
}

impl<A> Runtime<A>
where A: Allocator
{
	/// Should be called before every frame
	pub fn update(&mut self) {
		let buttons = self.callbacks.get_buttons();
		for i in 0..8 {
			self.memory[0x5f4c + i] = buttons[i] & 0x3f;
		}
		self.buttons.update(self.target_fps, &buttons);
	}
	
	/// Returns actual cursor position (memory only contains lower u8 of each coordinate)
	pub fn get_cursor_position(&self) -> [i16; 2] {
		self.cursor
	}
	
	pub fn set_cursor_position(&mut self, pos: [i16; 2]) {
		self.set_cursor_x(pos[0]);
		self.set_cursor_y(pos[1]);
	}
	
	pub fn set_cursor_x(&mut self, val: i16) {
		self.cursor[0] = val;
		self.memory.machine_state()._cursor_position()[0] = val as u8;
	}

	pub fn set_cursor_y(&mut self, val: i16) {
		self.cursor[1] = val;
		self.memory.machine_state()._cursor_position()[1] = val as u8;
	}
	
	
	pub fn get_cursor_home(&self) -> i16 {
		self.cursor_home
	}
	
	pub fn set_cursor_home(&mut self, x: i16) {
		self.cursor_home = x;
		*self.memory.machine_state()._cursor_home_x() = x as u8;
	}
}

impl<A> p8rs_piccolo::Runtime for Runtime<A>
where A: Allocator + 'static {
	fn as_any(&mut self) -> &mut dyn Any {
		self
	}
	
	fn peek(&mut self, addr: u16) -> u8 {
		self.memory.read(addr)
	}
	
	fn peek2(&mut self, addr: u16) -> u16 {
		self.memory.read(addr)
	}
	
	fn peek4(&mut self, addr: u16) -> u32 {
		self.memory.read(addr)
	}
}

pub struct Buttons {
	buttons: [u8; 8],
	buttons_repeat_state: [u8; 8],
	buttons_held_frames: [u8; 8*8],
}

#[allow(dead_code)]
impl Buttons {
	pub fn new() -> Buttons {
		Self {
			buttons: [0; 8],
			buttons_repeat_state: [0; 8],
			buttons_held_frames: [0; 8*8],
		}
	}
	
	pub fn update(&mut self, fps: u16, state: &[u8; 8]) {
		self.buttons.copy_from_slice(state);
		
		for player in 0..8 {
			let buttons = self.buttons[player];
			
			for i in 0..8 {
				let mask = 1 << i;
				let idx = player * 8 + i;
				
				if buttons & mask != 0 { // button is pressed
					self.buttons_held_frames[idx] += 1;
					
					let repeating = self.buttons_repeat_state[player] & mask != 0;
					let limit = fps.div(if repeating { 7 } else { 2 }).min(255) as u8;
					
					if self.buttons_held_frames[idx] >= limit {
						self.buttons_held_frames[idx] = 0;
						if !repeating {
							self.buttons_held_frames[player] |= mask;
						}
					}
				} else { // button is not pressed
					self.buttons_held_frames[idx] = 0;
					self.buttons_repeat_state[player] &= mask.not();
				}
			}
		}
	}
	
	pub fn get_bits_for_player(&self, player: usize) -> u8 {
		assert!(player < 8, "player idx > 8");
		self.buttons[player]
	}
	
	pub fn is_down(&self, player: usize, button: usize) -> bool {
		assert!(button < 8, "button idx > 8");
		(self.get_bits_for_player(player) & (1 << button)) != 0
	}
	
	pub fn is_just_pressed(&self, player: usize, button: usize) -> bool {
		self.is_down(player, button) && self.buttons_held_frames[player * 8 + button] == 1
	}
}
