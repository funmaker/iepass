use core::alloc::Allocator;
use core::ops::{Div, Not};
use core::any::Any;
use alloc::alloc::Global;
use alloc::boxed::Box;

use crate::pico8::callbacks::{Callbacks, DefaultCallbacks};
use crate::pico8::memory::Memory;
use crate::utils;

pub struct Runtime<A: Allocator = Global> {
	pub cart_memory: Box<[u8; 0x8000], A>,
	pub memory: Memory<A>,
	pub buttons: Buttons,
	pub target_fps: u16,
	pub callbacks: Box<dyn Callbacks>,
}

impl<A> Runtime<A>
where A: Allocator + Clone {
	pub fn new(alloc: A) -> Runtime<A> {
		Self {
			cart_memory: utils::new_zeroed_box_in(alloc.clone()),
			memory: Memory::new(alloc),
			buttons: Buttons::new(),
			target_fps: 30,
			callbacks: Box::new(DefaultCallbacks),
		}
	}
	
	pub fn finish_update_frame(&mut self) {
		self.buttons.finish_update_frame(self.target_fps);
	}
	
	pub fn update_buttons(&mut self, buttons: &[u8; 8]) {
		self.buttons.update_state(buttons);
		for i in 0..8 {
			self.memory[0x5f4c + i] = buttons[i] & 0x3f;
		}
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
	
	pub fn finish_update_frame(&mut self, fps: u16) {
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
	
	fn update_state(&mut self, state: &[u8; 8]) {
		self.buttons.copy_from_slice(state);
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
